use std::sync::Arc;

use php_ast::owned::{ExprKind, StaticDynMethodCallExpr, StaticMethodCallExpr};
use php_ast::Span;

use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Type};

use mir_codebase::definitions::DeclaredParam;
use mir_types::Name;
use rustc_hash::FxHashMap;

use crate::expr::ExpressionAnalyzer;
use crate::flow_state::{self_is_trait, FlowState};
use crate::narrowing::extract_expr_guard_key;
use crate::symbol::ReferenceKind;

use super::args::{
    check_args, check_method_visibility_with_magic, distinct_spans_for_expansion,
    expand_sole_spread_arg, expr_can_be_passed_by_reference_owned, spread_element_type,
    substitute_static_in_return, CheckArgsParams,
};
use super::method::resolve_method_from_db;
use super::CallAnalyzer;
use crate::generic::{
    build_class_bindings, check_template_bounds_with_inheritance, infer_arg_template_bindings,
    infer_template_bindings,
};

/// Widen scalar literal atomics to their base type before attaching an
/// inferred binding as a receiver's generic type param — mirrors
/// `expr::objects::widen_type_param` (kept separate since that module is
/// private to `expr`). Carrying a literal type param into the receiver
/// (e.g. `Box<42>` from `Box::make(42)`) is over-narrow and risks false
/// positives downstream (e.g. `$box->set(43)` where `set(T)` and `T=42`
/// would wrongly reject `43`).
fn widen_own_type_param(ty: &Type) -> Type {
    let mut out = Type::empty();
    out.from_docblock = ty.from_docblock;
    out.possibly_undefined = ty.possibly_undefined;
    for atomic in &ty.types {
        let widened = match atomic {
            Atomic::TLiteralInt(_) | Atomic::TIntRange { .. } => Atomic::TInt,
            Atomic::TLiteralString(_) => Atomic::TString,
            Atomic::TLiteralFloat(_, _) => Atomic::TFloat,
            Atomic::TTrue | Atomic::TFalse => Atomic::TBool,
            other => other.clone(),
        };
        out.add_type(widened);
    }
    out
}

fn extract_namespace(fqcn: &str) -> Option<&str> {
    if let Some(pos) = fqcn.rfind('\\') {
        Some(&fqcn[..pos])
    } else {
        None
    }
}

/// First namespace segment ("root namespace"), or `None` for the global namespace.
/// `@internal` (no argument) scopes a symbol to its root namespace, so any
/// sub-namespace under the same root may use it (Psalm semantics).
fn namespace_root(ns: Option<&str>) -> Option<&str> {
    ns.map(|n| n.trim_start_matches('\\'))
        .and_then(|n| n.split('\\').next())
        .filter(|seg| !seg.is_empty())
}

fn is_valid_class_name_type(ty: &Type) -> bool {
    // Class names must be strings or class-string types.
    // Mixed is allowed since it's already imprecise. Template params are
    // allowed because their bound may be a class-string.
    ty.contains(|t| {
        matches!(
            t,
            Atomic::TString
                | Atomic::TClassString(_)
                | Atomic::TLiteralString(_)
                | Atomic::TMixed
                | Atomic::TTemplateParam { .. }
        )
    })
}

fn is_object_atomic(t: &Atomic) -> bool {
    matches!(
        t,
        Atomic::TObject
            | Atomic::TNamedObject { .. }
            | Atomic::TStaticObject { .. }
            | Atomic::TSelf { .. }
            | Atomic::TParent { .. }
            | Atomic::TIntersection { .. }
            | Atomic::TNull
    )
}

/// If `ty` is a uniform single-class object type (possibly nullable), return
/// its FQCN so the static call can be resolved against it.  Returns `None`
/// for `object`, multi-class unions, or any non-object/non-null type component.
///
/// `$this::method()` and `$obj::method()` use LSB semantics at runtime; we
/// approximate here with the declared class, which is correct in the common
/// case and never produces a false positive.  Null is skipped — null safety
/// on `::` is a separate concern from class-string validity.
///
/// Also covers `$cls::method()` where `$cls` holds a known class-string
/// (`TClassString(Some(fqcn))`, e.g. `$cls = Foo::class;`) — otherwise this
/// call form skips method resolution/reference-recording entirely, unlike
/// its already-handled `$cls::$prop` / `$cls::CONST` siblings.
fn extract_object_fqcn(ty: &Type) -> Option<String> {
    let mut result: Option<String> = None;
    for atom in ty.types.iter() {
        let fqcn_str = match atom {
            Atomic::TNamedObject { fqcn, .. }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TSelf { fqcn }
            | Atomic::TParent { fqcn } => fqcn.to_string(),
            Atomic::TClassString(Some(fqcn)) => fqcn.to_string(),
            Atomic::TNull => continue, // nullable object: skip null, resolve against class
            _ => return None,
        };
        match &result {
            None => result = Some(fqcn_str),
            Some(existing) if *existing == fqcn_str => {}
            _ => return None,
        }
    }
    result
}

/// The concrete type args a `$var::method()`/`$this::method()` receiver
/// carries for its own class-level `@template` (e.g. `int` from a `Box<int>
/// $b` receiver in `$b::peek()`). Mirrors what `method.rs`'s instance-call
/// path reads straight off the `TNamedObject` atom — without this, a
/// receiver's own type args never reach the static-call template binding,
/// so a `@return T` leaks the raw template atom instead of resolving it.
fn extract_receiver_type_params(ty: &Type, fqcn: &str) -> Vec<Type> {
    ty.types
        .iter()
        .find_map(|atom| match atom {
            Atomic::TNamedObject {
                fqcn: f,
                type_params,
            } if f.as_ref() == fqcn => Some(type_params.to_vec()),
            _ => None,
        })
        .unwrap_or_default()
}

impl CallAnalyzer {
    pub fn analyze_static_method_call<'a>(
        ea: &mut ExpressionAnalyzer<'a>,
        call: &StaticMethodCallExpr,
        ctx: &mut FlowState,
        span: Span,
    ) -> Type {
        let method_name = match &call.method.kind {
            ExprKind::Identifier(name) => name.as_ref(),
            _ => return Type::mixed(),
        };

        let mut receiver_type_params: Vec<Type> = Vec::new();
        let fqcn = match &call.class.kind {
            ExprKind::Identifier(name) => crate::db::resolve_name(ea.db, &ea.file, name.as_ref()),
            _ => {
                let ty = ea.analyze(&call.class, ctx);
                // $obj::method() / $this::method(): resolve against the object's class
                if let Some(fqcn) = extract_object_fqcn(&ty) {
                    if ty.is_nullable() && !ty.is_mixed() {
                        ea.emit(
                            IssueKind::PossiblyNullMethodCall {
                                method: method_name.to_string(),
                            },
                            Severity::Info,
                            call.class.span,
                        );
                    }
                    receiver_type_params = extract_receiver_type_params(&ty, &fqcn);
                    fqcn
                } else {
                    // All-object unions (Foo|Bar, object) are valid PHP — skip error
                    if !is_valid_class_name_type(&ty) && !ty.types.iter().all(is_object_atomic) {
                        ea.emit(
                            IssueKind::InvalidStringClass {
                                actual: ty.to_string(),
                            },
                            Severity::Warning,
                            call.class.span,
                        );
                    }
                    return Type::mixed();
                }
            }
        };

        // Detect `parent::` used in a class that has no parent. Skip inside a
        // trait: `parent::` there resolves against the using class at runtime,
        // not the trait (which never has a parent).
        if fqcn.eq_ignore_ascii_case("parent")
            && ctx.parent_fqcn.is_none()
            && ctx.self_fqcn.is_some()
            && !self_is_trait(ea.db, ctx)
        {
            ea.emit(IssueKind::ParentNotFound, Severity::Error, call.class.span);
        }

        let fqcn = resolve_static_class(&fqcn, ctx);

        if matches!(&call.class.kind, ExprKind::Identifier(_)) {
            ea.record_ref(Arc::from(format!("cls:{fqcn}")), call.class.span);
            // Record a symbol on the class token itself so hover / go-to-definition
            // works when the cursor sits on the class name — including the
            // `self`/`parent`/`static` keywords, which `resolve_static_class`
            // has already mapped to a concrete FQCN.  Mirrors `new Foo` and
            // `instanceof Foo`.  Skip the literal keywords that failed to
            // resolve (e.g. `parent::` with no parent), which carry no class.
            if !matches!(fqcn.as_str(), "self" | "static" | "parent") {
                ea.record_symbol(
                    call.class.span,
                    ReferenceKind::ClassReference(Arc::from(fqcn.as_str())),
                    Type::single(Atomic::TClassString(None)),
                );
            }
            // Check if the class is deprecated (skip self/static/parent)
            if !matches!(fqcn.as_str(), "self" | "static" | "parent") {
                let here = crate::db::Fqcn::from_str(ea.db, fqcn.as_str());
                if let Some(class) = crate::db::find_class_like(ea.db, here) {
                    if let Some(msg) = class.deprecated() {
                        ea.emit(
                            IssueKind::DeprecatedClass {
                                name: fqcn.clone(),
                                message: Some(msg.clone()).filter(|m| !m.is_empty()),
                            },
                            Severity::Info,
                            call.class.span,
                        );
                    }
                    // Check for case mismatch between the written class name and canonical
                    if let Some((used, canonical_str)) =
                        crate::fqcn_case_mismatch(fqcn.as_str(), class.fqcn().as_ref())
                    {
                        ea.emit(
                            IssueKind::WrongCaseClass {
                                used,
                                canonical: canonical_str,
                            },
                            Severity::Info,
                            call.class.span,
                        );
                    }
                }
            }
        }

        let fqcn_arc: Arc<str> = Arc::from(fqcn.as_str());
        let method_name_lower = crate::util::php_ident_lowercase(method_name);

        // Pre-mark by-reference argument variables as defined before evaluating
        // the arguments, so passing an undefined variable to an out-param (e.g.
        // `Registry::build($items, $out)` where `$out` is `@param-out`) does
        // not produce a false UndefinedVariable.
        if let Some(pre_resolved) = resolve_method_from_db(ea, &fqcn_arc, &method_name_lower) {
            super::premark_byref_arg_vars(&pre_resolved.params, &call.args, ctx);
        }

        let mut sole_spread_ty: Option<Type> = None;
        let mut arg_types: Vec<Type> = call
            .args
            .iter()
            .map(|arg| {
                let ty = ea.analyze(&arg.value, ctx);
                super::consume_arg_assignment(&arg.value, ctx);
                if arg.unpack {
                    if call.args.len() == 1 {
                        sole_spread_ty = Some(ty.clone());
                    }
                    spread_element_type(&ty)
                } else {
                    ty
                }
            })
            .collect();
        let mut arg_spans: Vec<Span> = call.args.iter().map(|a| a.span).collect();

        // Check if trying to call static method on an interface (not allowed)
        if crate::db::class_exists(ea.db, &fqcn) {
            let here = crate::db::Fqcn::from_str(ea.db, fqcn_arc.as_ref());
            let is_interface = crate::db::find_class_like(ea.db, here)
                .map(|c| c.is_interface())
                .unwrap_or(false);
            if is_interface {
                ea.emit(
                    IssueKind::UndefinedClass { name: fqcn.clone() },
                    Severity::Error,
                    call.class.span,
                );
                return Type::mixed();
            }
        }

        // Closure::bind($closure, $newThis, $newScope = 'static'): ?Closure
        // Preserve the closure's params and return_type, update this_type
        if fqcn_arc.as_ref() == "Closure" && method_name_lower == "bind" {
            if let Some(closure_arg) = arg_types.first() {
                for atomic in &closure_arg.types {
                    if let mir_types::Atomic::TClosure { data } = atomic {
                        let new_this = arg_types.get(1).cloned().unwrap_or_else(Type::null);
                        let this_type = {
                            let non_null = new_this.remove_null();
                            if non_null.is_empty() {
                                None
                            } else {
                                Some(non_null)
                            }
                        };
                        let mut result = Type::single(mir_types::Atomic::TClosure {
                            data: Box::new(mir_types::atomic::ClosureData {
                                params: data.params.clone(),
                                return_type: data.return_type.clone(),
                                this_type,
                            }),
                        });
                        result.add_type(mir_types::Atomic::TNull);
                        return result;
                    }
                }
            }
            // If we can't determine the closure type from the first arg, fall through to stub resolution
        }

        // Closure::fromCallable('helper') / Closure::fromCallable('Foo::bar'):
        // a bare string callable argument is a real runtime reference, same as
        // call_user_func('name') — record it, or a function/method reachable
        // only this way is falsely flagged dead code.
        if fqcn_arc.as_ref() == "Closure" && method_name_lower == "fromcallable" {
            if let (Some(callback_ty), Some(&callback_span)) =
                (arg_types.first(), arg_spans.first())
            {
                super::callable::record_callable_string_ref(ea, callback_ty, callback_span);
            }
        }

        let resolved = resolve_method_from_db(ea, &fqcn_arc, &method_name_lower);

        if let Some(resolved) = resolved {
            ea.record_ref(
                Arc::from(format!(
                    "meth:{}::{}",
                    resolved.owner_fqcn,
                    crate::util::php_ident_lowercase(method_name)
                )),
                call.method.span,
            );
            if let Some(msg) = resolved.deprecated.clone() {
                ea.emit(
                    IssueKind::DeprecatedMethodCall {
                        class: fqcn.clone(),
                        method: method_name.to_string(),
                        message: Some(msg).filter(|m| !m.is_empty()),
                    },
                    Severity::Info,
                    span,
                );
            }
            // Purity check: a static call has no receiver to scope the
            // "only externally-visible mutations matter" exception to (unlike
            // instance calls on a local object) — any non-pure static/self::/
            // parent:: call inside a @pure function can touch static state,
            // so it's flagged unconditionally, mirroring the plain
            // function-call check in call/function.rs.
            if ctx.is_in_pure_fn && !resolved.is_pure {
                ea.emit(
                    IssueKind::ImpureMethodCall {
                        method: method_name.to_string(),
                    },
                    Severity::Warning,
                    span,
                );
            }
            if method_name != resolved.name.as_ref()
                && method_name.eq_ignore_ascii_case(resolved.name.as_ref())
            {
                ea.emit(
                    IssueKind::WrongCaseMethod {
                        class: fqcn.clone(),
                        used: method_name.to_string(),
                        canonical: resolved.name.to_string(),
                    },
                    Severity::Info,
                    call.method.span,
                );
            }
            // Detect call to an abstract method via an explicit class name.
            // `$this::method()` is self-referential too (LSB against the
            // current instance), same as the `self`/`static`/`parent`
            // keywords — otherwise it falls into the "explicit class name"
            // path below and produces a false `InvalidStaticInvocation` on
            // a non-static method, and drops `@psalm-self-out` narrowing.
            let is_self_parent_call = match &call.class.kind {
                ExprKind::Identifier(id) => matches!(id.as_ref(), "self" | "static" | "parent"),
                ExprKind::Variable(name) => name.trim_start_matches('$') == "this",
                _ => false,
            };
            // Only static:: uses LSB and resolves to the concrete subclass at runtime.
            // self:: resolves to the declaring class (abstract → no body to call).
            // parent:: resolves to the parent class (abstract → no body to call).
            let is_static_keyword = if let ExprKind::Identifier(id) = &call.class.kind {
                id.as_ref() == "static"
            } else {
                false
            };
            if resolved.is_abstract && !is_static_keyword {
                ea.emit(
                    IssueKind::AbstractMethodCall {
                        class: fqcn.clone(),
                        method: method_name.to_string(),
                    },
                    Severity::Error,
                    span,
                );
            }
            if !resolved.is_static
                && !method_name.starts_with("__")
                && !is_self_parent_call
                && !crate::db::has_method_in_chain(ea.db, fqcn.as_str(), "__callStatic")
            {
                ea.emit(
                    IssueKind::InvalidStaticInvocation {
                        class: fqcn.clone(),
                        method: method_name.to_string(),
                    },
                    Severity::Error,
                    span,
                );
            }
            // Detect non-static method called via self::/static:: from a static context.
            // Note: __callStatic only intercepts UNDEFINED methods, so we don't suppress here
            // when the method is explicitly defined as non-static.
            if !resolved.is_static
                && !method_name.starts_with("__")
                && is_self_parent_call
                && ctx.inside_static_method
            {
                ea.emit(
                    IssueKind::NonStaticSelfCall {
                        class: fqcn.clone(),
                        method: method_name.to_string(),
                    },
                    Severity::Error,
                    span,
                );
            }
            if resolved.is_internal {
                let calling_ns = ea.db.file_namespace(&ea.file);
                let calling_root = namespace_root(calling_ns.as_deref());
                let owner_root = namespace_root(extract_namespace(&resolved.owner_fqcn));
                // self::/static::/parent:: calls are self-calls; also allow when calling
                // on a class that is the current self (trait @internal methods included).
                let is_self_call = is_self_parent_call
                    || ctx
                        .self_fqcn
                        .as_deref()
                        .map(|s| s.eq_ignore_ascii_case(fqcn.as_str()))
                        .unwrap_or(false);
                if calling_root != owner_root && !is_self_call {
                    ea.emit(
                        IssueKind::InternalMethod {
                            class: fqcn.clone(),
                            method: method_name.to_string(),
                        },
                        Severity::Warning,
                        span,
                    );
                }
            }
            // Only checked for genuinely static methods: a non-static method
            // called via `Foo::bar()`/self::/static:: already gets a more
            // precise `InvalidStaticInvocation`/`NonStaticSelfCall` above —
            // piling on a visibility error for the same call site is noise.
            if resolved.is_static {
                check_method_visibility_with_magic(
                    ea,
                    resolved.visibility,
                    &resolved.owner_fqcn,
                    &resolved.name,
                    ctx,
                    span,
                    "__callStatic",
                );
            }
            let mut arg_names: Vec<Option<String>> = call
                .args
                .iter()
                .map(|a| a.name.as_ref().map(crate::parser::name_to_string_owned))
                .collect();
            let mut arg_can_be_byref: Vec<bool> = call
                .args
                .iter()
                .map(|a| expr_can_be_passed_by_reference_owned(&a.value))
                .collect();
            let mut has_spread = call.args.iter().any(|a| a.unpack);
            let mut arity_unknown = has_spread;
            // A sole spread arg over a literal, sequentially-keyed shape can be
            // expanded into one binding per element so each parameter (and
            // template-binding inference below) is checked individually instead
            // of only the first (see expand_sole_spread_arg). `arity_unknown`
            // stays true even after expansion — PHP allows extra/spread
            // positional args, so a concretely-known count still shouldn't
            // trigger TooFew/TooManyArguments.
            if let Some(expanded) = sole_spread_ty
                .take()
                .and_then(|t| expand_sole_spread_arg(&t))
            {
                arg_spans = distinct_spans_for_expansion(arg_spans[0], expanded.len());
                arg_names = vec![None; expanded.len()];
                arg_can_be_byref = vec![false; expanded.len()];
                arg_types = expanded;
                has_spread = false;
                arity_unknown = true;
            }
            // `Foo::bar()` has no receiver value to carry type params, so the
            // seed is empty and class-level bindings come solely from `fqcn`'s
            // own `@extends`/`@implements` chain. `$var::bar()`/`$this::bar()`
            // seeds `receiver_type_params` from the receiver's own concrete
            // type args (e.g. `int` from a `Box<int>` receiver) the same way
            // method.rs's instance-call path does, so a class-level `@template`
            // bound on the receiver resolves instead of leaking through raw.
            let class_tps = crate::db::class_template_params(ea.db, &fqcn).unwrap_or_default();
            let mut class_bindings = build_class_bindings(&class_tps, &receiver_type_params);
            let inherited_class_bindings =
                crate::db::inherited_template_bindings(ea.db, &fqcn, &class_bindings);
            // The resolved method's params/return/out-types are declared on
            // `resolved.owner_fqcn` — a bare template name in its signature
            // is the RECEIVER's own template only when the receiver itself
            // declares the method; otherwise it's scoped to the ancestor
            // that actually declares it (same collision the property-access/
            // instance-method-call/foreach/array-access sites already guard
            // against).
            if resolved.owner_fqcn.as_ref() == fqcn.as_str() {
                for (k, v) in inherited_class_bindings {
                    class_bindings.entry(k).or_insert(v);
                }
            } else {
                class_bindings.extend(inherited_class_bindings);
            }
            // A class-level `@template T of Bound` was previously only ever
            // checked at `new Box(...)` construction sites — a receiver typed
            // `Box<NotAnimal>` via a docblock/param annotation instead sailed
            // through every static/self/parent call unchecked, regardless of
            // whether the called method itself declares its own template params.
            for (name, inferred, bound) in check_template_bounds_with_inheritance(
                ea.db,
                &class_bindings,
                &class_tps,
                &Default::default(),
                Some(fqcn_arc.as_ref()),
            ) {
                ea.emit(
                    IssueKind::InvalidTemplateParam {
                        name: name.to_string(),
                        expected_bound: format!("{bound}"),
                        actual: format!("{inferred}"),
                    },
                    Severity::Error,
                    span,
                );
            }
            let mut param_bindings = class_bindings.clone();
            for tp in resolved.template_params.iter() {
                param_bindings.remove(&Name::from(tp.name.as_ref()));
            }
            let substituted_params: Vec<DeclaredParam>;
            let effective_params: &[DeclaredParam] = if param_bindings.is_empty() {
                &resolved.params
            } else {
                substituted_params = resolved
                    .params
                    .iter()
                    .map(|p| DeclaredParam {
                        ty: mir_codebase::wrap_param_type(
                            p.ty.as_ref()
                                .map(|t| t.substitute_templates(&param_bindings)),
                        ),
                        ..p.clone()
                    })
                    .collect();
                &substituted_params
            };
            check_args(
                ea,
                CheckArgsParams {
                    fn_name: method_name,
                    params: effective_params,
                    arg_types: &arg_types,
                    arg_spans: &arg_spans,
                    arg_names: &arg_names,
                    arg_can_be_byref: &arg_can_be_byref,
                    call_span: span,
                    has_spread,
                    arity_unknown,
                    template_params: &resolved.template_params,
                    no_named_arguments: resolved.no_named_arguments,
                },
            );
            let owner_fqcn = resolved.owner_fqcn.clone();
            let ret_raw = resolved.return_ty_raw;

            let method_bindings = if !resolved.template_params.is_empty() {
                let (bindings, unchecked) = infer_template_bindings(
                    ea.db,
                    &resolved.template_params,
                    effective_params,
                    &arg_types,
                    &arg_names,
                );
                // Static calls (`Foo::bar()`, `self::bar()`, `parent::bar()`)
                // previously never checked the method's own `@template ... of
                // Bound` at all — only instance-method and function calls did.
                for (name, inferred, bound) in check_template_bounds_with_inheritance(
                    ea.db,
                    &bindings,
                    &resolved.template_params,
                    &unchecked,
                    Some(fqcn_arc.as_ref()),
                ) {
                    ea.emit(
                        IssueKind::InvalidTemplateParam {
                            name: name.to_string(),
                            expected_bound: format!("{bound}"),
                            actual: format!("{inferred}"),
                        },
                        Severity::Error,
                        span,
                    );
                }
                // Only warn about template shadowing when the declaring class lives
                // in the file under analysis — mirrors method.rs's instance-call
                // check, which a static call (Foo::bar()) previously never got at
                // all despite computing an equivalent class_bindings/method
                // bindings pair right here.
                let declared_here = crate::db::class_like_decl_file(
                    ea.db,
                    crate::db::Fqcn::from_str(ea.db, resolved.owner_fqcn.as_ref()),
                )
                .is_some_and(|f| f.as_ref() == ea.file.as_ref());
                if declared_here {
                    for key in bindings.keys() {
                        if class_bindings.contains_key(key) {
                            ea.emit(
                                IssueKind::ShadowedTemplateParam {
                                    name: key.to_string(),
                                },
                                Severity::Info,
                                span,
                            );
                        }
                    }
                }
                Some(bindings)
            } else {
                None
            };

            // The CLASS's own template (as opposed to `resolved.template_params`,
            // the METHOD's own separately-declared templates) isn't otherwise
            // bound for a bare `Foo::make(...)` call with no concretizing
            // subclass — infer it from the call's arguments the same way `new
            // Foo(...)` does for constructors (`infer_new_type_params`), so a
            // `@return static`/`@return T` on a static factory resolves to
            // the concrete type instead of leaking the bare class template. A
            // declared binding (from `@extends`/`@implements`, e.g. a
            // concretizing subclass) still takes precedence over one merely
            // inferred from this call's arguments.
            // A plain subclass that doesn't redeclare `@template` (`class
            // IntBox extends Box {}`) still shares Box's template slot, so
            // `IntBox::make(42)` must infer against Box's declared params —
            // walk up to the nearest ancestor that actually declares them.
            // (`class_tps` is already computed above, alongside `class_bindings`.)
            let class_arg_bindings: FxHashMap<Name, Type> = if class_tps.is_empty() {
                FxHashMap::default()
            } else {
                infer_arg_template_bindings(
                    ea.db,
                    &class_tps,
                    &resolved.params,
                    &arg_types,
                    &arg_names,
                )
                .0
                .into_iter()
                .map(|(name, ty)| (name, widen_own_type_param(&ty)))
                .collect()
            };
            let return_class_bindings: FxHashMap<Name, Type> = if class_arg_bindings.is_empty() {
                class_bindings.clone()
            } else {
                let mut merged = class_arg_bindings;
                for (name, ty) in class_bindings.iter() {
                    merged.insert(*name, ty.clone());
                }
                merged
            };

            // `static`'s receiver type params come from the CLASS's own
            // template params, not the method's — attach them before
            // substituting `static`/`self` in the return type, or a bare
            // `@return static` resolves to an unparameterized `Box` and
            // erases them entirely.
            let own_type_params: Vec<Type> = class_tps
                .iter()
                .map(|tp| {
                    return_class_bindings
                        .get(&Name::from(tp.name.as_ref()))
                        .cloned()
                        .unwrap_or_else(Type::mixed)
                })
                .collect();

            let ret_substituted = substitute_static_in_return(ret_raw, &fqcn_arc, &own_type_params);
            let ret_substituted = if return_class_bindings.is_empty() {
                ret_substituted
            } else {
                ret_substituted.substitute_templates(&return_class_bindings)
            };
            let ret = match &method_bindings {
                Some(bindings) => ret_substituted.substitute_templates(bindings),
                None => ret_substituted,
            };
            let ret = ret.resolve_conditional_returns(|param_name| {
                resolved
                    .params
                    .iter()
                    .position(|p| p.name.as_ref() == param_name)
                    .and_then(|idx| arg_types.get(idx))
                    .cloned()
            });
            // Write @param-out types back to caller variables for by-ref params.
            // Substitute the same bindings the return type uses (class template
            // from `@extends`/inferred-from-args, then the method's own), so a
            // generic method's out-type resolves the class's bound type param
            // instead of leaking the bare template name to the caller.
            let mut out_bindings = return_class_bindings.clone();
            if let Some(mb) = &method_bindings {
                for (k, v) in mb.iter() {
                    out_bindings.insert(*k, v.clone());
                }
            }
            for (i, param) in resolved.params.iter().enumerate() {
                let Some(out_ty) = param.out_ty.as_ref() else {
                    continue;
                };
                // `@param-out self`/`@param-out static` must resolve to the receiver's
                // concrete class, the same way `@return static` already does.
                let out_ty =
                    substitute_static_in_return((**out_ty).clone(), &fqcn_arc, &own_type_params);
                let out_ty = if out_bindings.is_empty() {
                    out_ty
                } else {
                    out_ty.substitute_templates(&out_bindings)
                };
                if param.is_variadic {
                    for arg in call.args.iter().skip(i) {
                        if let ExprKind::Variable(name) = &arg.value.kind {
                            ctx.set_var(name.as_ref().trim_start_matches('$'), out_ty.clone());
                        }
                    }
                } else if let Some(arg) = call.args.get(i) {
                    if let ExprKind::Variable(name) = &arg.value.kind {
                        ctx.set_var(name.as_ref().trim_start_matches('$'), out_ty);
                    }
                }
            }
            // `@if-this-is X<Y>` on a method reached through self::/static::/
            // parent:: — mirrors the instance-call-syntax handling in
            // `resolve_method_return`, which `analyze_static_method_call` never
            // invoked at all, so this idiom silently never fired for the
            // (more common) self::/static:: call syntax, only `$this->method()`.
            if is_self_parent_call {
                if let Some(constraint) = resolved.if_this_is.clone() {
                    let receiver_type_params =
                        extract_receiver_type_params(&ctx.get_var("this"), &fqcn_arc);
                    let constraint_has_params = constraint.types.iter().any(|a| {
                        matches!(a, Atomic::TNamedObject { type_params, .. } if !type_params.is_empty())
                    });
                    let receiver_has_unresolved_template = receiver_type_params.iter().any(|t| {
                        t.types
                            .iter()
                            .any(|a| matches!(a, Atomic::TTemplateParam { .. }))
                    });
                    if !receiver_has_unresolved_template
                        && (!receiver_type_params.is_empty() || !constraint_has_params)
                    {
                        let receiver = Type::single(Atomic::TNamedObject {
                            fqcn: Name::new(fqcn_arc.as_ref()),
                            type_params: receiver_type_params.to_vec().into(),
                        });
                        if !crate::subtype::is_subtype(ea.db, &receiver, &constraint) {
                            ea.emit(
                                IssueKind::IfThisIsMismatch {
                                    class: fqcn.clone(),
                                    method: method_name.to_string(),
                                    expected: format!("{constraint}"),
                                    actual: format!("{receiver}"),
                                },
                                Severity::Info,
                                span,
                            );
                        }
                    }
                }
            }
            // `@psalm-self-out Type` on a method reached through self::/static::/
            // parent:: retypes the implicit `$this` receiver, mirroring the
            // instance-call-syntax handling in `analyze_method_call`. A plain
            // `Foo::bar()` (not a self-referential call) has no `$this` receiver
            // for this call to have narrowed, so it's left alone.
            if is_self_parent_call {
                if let Some(self_out_raw) = resolved.self_out.clone() {
                    let self_out_ty =
                        substitute_static_in_return((*self_out_raw).clone(), &fqcn_arc, &[]);
                    let self_out_ty = if class_bindings.is_empty() {
                        self_out_ty
                    } else {
                        self_out_ty.substitute_templates(&class_bindings)
                    };
                    let self_out_ty = if !resolved.template_params.is_empty() {
                        let (mut method_bindings, _unchecked) = infer_template_bindings(
                            ea.db,
                            &resolved.template_params,
                            effective_params,
                            &arg_types,
                            &arg_names,
                        );
                        for v in method_bindings.values_mut() {
                            *v = crate::stmt::widen_for_check(v.clone());
                        }
                        self_out_ty.substitute_templates(&method_bindings)
                    } else {
                        self_out_ty
                    };
                    ctx.set_var("this", self_out_ty);
                }
            }
            ea.record_symbol(
                call.method.span,
                ReferenceKind::StaticCall {
                    class: owner_fqcn,
                    method: Arc::from(method_name),
                },
                ret.clone(),
            );
            ret
        } else if crate::db::class_exists(ea.db, &fqcn)
            && !crate::db::has_unknown_ancestor(ea.db, &fqcn)
        {
            let (is_abstract, is_trait) = crate::db::class_kind(ea.db, &fqcn)
                .map(|k| (k.is_abstract, k.is_trait))
                .unwrap_or((false, false));
            // Check for __callStatic in the full inheritance chain (not just direct methods)
            let has_callstatic_magic = crate::db::has_method_in_chain(ea.db, &fqcn, "__callstatic");
            // Suppress when caller guarded with `method_exists(Foo::class, 'method')`
            // (literal class name — keyed by the already-resolved `fqcn`) or
            // `method_exists($cls, 'method')` (dynamic class-string variable).
            let guard_key: Option<Arc<str>> = match &call.class.kind {
                ExprKind::Identifier(_) => Some(Arc::from(format!("cls:{fqcn}").as_str())),
                _ => extract_expr_guard_key(&call.class, ea.db, &ea.file),
            };
            let guarded_by_method_exists = guard_key
                .map(|key| {
                    ctx.method_exists_guards.contains(&(
                        key,
                        Arc::from(crate::util::php_ident_lowercase(method_name).as_str()),
                    ))
                })
                .unwrap_or(false);
            // The method didn't resolve on an otherwise-known class — keep a
            // name-only fallback so find-references on any `X::name` can
            // still surface this call, mirroring the instance-call path in
            // call/method.rs.
            ea.record_ref(
                Arc::from(format!(
                    "methname:{}",
                    crate::util::php_ident_lowercase(method_name)
                )),
                call.method.span,
            );
            if is_trait {
                // The call may be satisfied by whichever class ends up consuming
                // this trait — record a per-trait marker so DeadCodeAnalyzer can
                // credit any composing class's own private method of this name
                // as used, instead of only ever seeing the trait's own (failed)
                // resolution attempt. Mirrors the instance-call path in method.rs.
                ea.record_ref(
                    Arc::from(format!("traituse:{fqcn}::{method_name_lower}")),
                    call.method.span,
                );
            }
            // In a trait body, self::/static:: resolve to the consuming class,
            // which may provide the method — not undefined.
            if is_abstract || is_trait || has_callstatic_magic || guarded_by_method_exists {
                Type::mixed()
            } else {
                ea.emit(
                    IssueKind::UndefinedMethod {
                        class: fqcn,
                        method: method_name.to_string(),
                    },
                    Severity::Error,
                    span,
                );
                Type::mixed()
            }
        } else if !crate::db::class_exists(ea.db, &fqcn)
            && !matches!(fqcn.as_str(), "self" | "static" | "parent")
            && !ctx.is_class_guarded(fqcn.as_str())
        {
            // The class itself couldn't be resolved — same name-only fallback
            // as the known-class-unresolved-method branch above.
            ea.record_ref(
                Arc::from(format!(
                    "methname:{}",
                    crate::util::php_ident_lowercase(method_name)
                )),
                call.method.span,
            );
            ea.emit(
                IssueKind::UndefinedClass { name: fqcn },
                Severity::Error,
                call.class.span,
            );
            Type::mixed()
        } else {
            ea.record_ref(
                Arc::from(format!(
                    "methname:{}",
                    crate::util::php_ident_lowercase(method_name)
                )),
                call.method.span,
            );
            Type::mixed()
        }
    }

    pub fn analyze_static_dyn_method_call<'a>(
        ea: &mut ExpressionAnalyzer<'a>,
        call: &StaticDynMethodCallExpr,
        ctx: &mut FlowState,
    ) -> Type {
        // The called method's name isn't statically known — mark the target
        // class as dynamically accessed so DeadCodeAnalyzer doesn't flag its
        // private members as unused, mirroring the instance-call fallback.
        if let ExprKind::Identifier(name) = &call.class.kind {
            let resolved = crate::db::resolve_name(ea.db, &ea.file, name.as_ref());
            let fqcn = resolve_static_class(&resolved, ctx);
            if !matches!(fqcn.as_str(), "self" | "static" | "parent") {
                ea.record_ref(Arc::from(format!("dyn:{fqcn}")), call.method.span);
            }
        } else {
            let class_ty = ea.analyze(&call.class, ctx);
            ea.record_dynamic_member_access(&class_ty, call.method.span);
        }
        ea.analyze(&call.method, ctx);
        for arg in call.args.iter() {
            ea.analyze(&arg.value, ctx);
            super::consume_arg_assignment(&arg.value, ctx);
        }
        Type::mixed()
    }
}

fn resolve_static_class(name: &str, ctx: &FlowState) -> String {
    match crate::util::php_ident_lowercase(name).as_str() {
        "self" => ctx.self_fqcn.as_deref().unwrap_or("self").to_string(),
        "parent" => ctx.parent_fqcn.as_deref().unwrap_or("parent").to_string(),
        "static" => ctx
            .static_fqcn
            .as_deref()
            .unwrap_or(ctx.self_fqcn.as_deref().unwrap_or("static"))
            .to_string(),
        _ => name.to_string(),
    }
}
