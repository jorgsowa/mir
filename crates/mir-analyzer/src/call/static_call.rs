use std::sync::Arc;

use php_ast::owned::{ExprKind, StaticDynMethodCallExpr, StaticMethodCallExpr};
use php_ast::Span;

use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Type};

use crate::expr::ExpressionAnalyzer;
use crate::flow_state::{self_is_trait, FlowState};
use crate::symbol::ReferenceKind;

use super::args::{
    check_args, expr_can_be_passed_by_reference_owned, spread_element_type,
    substitute_static_in_return, CheckArgsParams,
};
use super::method::resolve_method_from_db;
use super::CallAnalyzer;
use crate::generic::infer_template_bindings;

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
fn extract_object_fqcn(ty: &Type) -> Option<String> {
    let mut result: Option<String> = None;
    for atom in ty.types.iter() {
        let fqcn_str = match atom {
            Atomic::TNamedObject { fqcn, .. }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TSelf { fqcn }
            | Atomic::TParent { fqcn } => fqcn.to_string(),
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
        if fqcn.to_lowercase() == "parent"
            && ctx.parent_fqcn.is_none()
            && ctx.self_fqcn.is_some()
            && !self_is_trait(ea.db, ctx)
        {
            ea.emit(IssueKind::ParentNotFound, Severity::Error, call.class.span);
        }

        let fqcn = resolve_static_class(&fqcn, ctx);

        if matches!(&call.class.kind, ExprKind::Identifier(_)) {
            ea.record_ref(Arc::from(fqcn.as_str()), call.class.span);
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

        let arg_types: Vec<Type> = call
            .args
            .iter()
            .map(|arg| {
                let ty = ea.analyze(&arg.value, ctx);
                super::consume_arg_assignment(&arg.value, ctx);
                if arg.unpack {
                    spread_element_type(&ty)
                } else {
                    ty
                }
            })
            .collect();
        let arg_spans: Vec<Span> = call.args.iter().map(|a| a.span).collect();

        let fqcn_arc: Arc<str> = Arc::from(fqcn.as_str());
        let method_name_lower = method_name.to_lowercase();

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
                    if let mir_types::Atomic::TClosure {
                        params,
                        return_type,
                        ..
                    } = atomic
                    {
                        let new_this = arg_types.get(1).cloned().unwrap_or_else(Type::null);
                        let this_type = {
                            let non_null = new_this.remove_null();
                            if non_null.is_empty() {
                                None
                            } else {
                                Some(Box::new(non_null))
                            }
                        };
                        let mut result = Type::single(mir_types::Atomic::TClosure {
                            params: params.clone(),
                            return_type: return_type.clone(),
                            this_type,
                        });
                        result.add_type(mir_types::Atomic::TNull);
                        return result;
                    }
                }
            }
            // If we can't determine the closure type from the first arg, fall through to stub resolution
        }

        let resolved = resolve_method_from_db(ea, &fqcn_arc, &method_name_lower);

        if let Some(resolved) = resolved {
            ea.record_ref(
                Arc::from(format!(
                    "{}::{}",
                    resolved.owner_fqcn,
                    method_name.to_lowercase()
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
            // Skip self/static/parent callers: those resolve to a concrete subclass at runtime.
            let is_self_parent_call = if let ExprKind::Identifier(id) = &call.class.kind {
                matches!(id.as_ref(), "self" | "static" | "parent")
            } else {
                false
            };
            if resolved.is_abstract && !is_self_parent_call {
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
            let arg_names: Vec<Option<String>> = call
                .args
                .iter()
                .map(|a| a.name.as_ref().map(crate::parser::name_to_string_owned))
                .collect();
            let arg_can_be_byref: Vec<bool> = call
                .args
                .iter()
                .map(|a| expr_can_be_passed_by_reference_owned(&a.value))
                .collect();
            check_args(
                ea,
                CheckArgsParams {
                    fn_name: method_name,
                    params: &resolved.params,
                    arg_types: &arg_types,
                    arg_spans: &arg_spans,
                    arg_names: &arg_names,
                    arg_can_be_byref: &arg_can_be_byref,
                    call_span: span,
                    has_spread: call.args.iter().any(|a| a.unpack),
                    template_params: &resolved.template_params,
                    no_named_arguments: resolved.no_named_arguments,
                },
            );
            let owner_fqcn = resolved.owner_fqcn.clone();
            let ret_raw = resolved.return_ty_raw;
            let ret_substituted = substitute_static_in_return(ret_raw, &fqcn_arc);
            let ret = if !resolved.template_params.is_empty() {
                let bindings = infer_template_bindings(
                    &resolved.template_params,
                    &resolved.params,
                    &arg_types,
                );
                ret_substituted.substitute_templates(&bindings)
            } else {
                ret_substituted
            };
            let ret = ret.resolve_conditional_returns(|param_name| {
                resolved
                    .params
                    .iter()
                    .position(|p| p.name.as_ref() == param_name)
                    .and_then(|idx| arg_types.get(idx))
                    .cloned()
            });
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
            // In a trait body, self::/static:: resolve to the consuming class,
            // which may provide the method — not undefined.
            if is_abstract || is_trait || has_callstatic_magic {
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
            && !ctx.class_exists_guards.contains(fqcn.as_str())
        {
            ea.emit(
                IssueKind::UndefinedClass { name: fqcn },
                Severity::Error,
                call.class.span,
            );
            Type::mixed()
        } else {
            Type::mixed()
        }
    }

    pub fn analyze_static_dyn_method_call<'a>(
        ea: &mut ExpressionAnalyzer<'a>,
        call: &StaticDynMethodCallExpr,
        ctx: &mut FlowState,
    ) -> Type {
        for arg in call.args.iter() {
            ea.analyze(&arg.value, ctx);
            super::consume_arg_assignment(&arg.value, ctx);
        }
        Type::mixed()
    }
}

fn resolve_static_class(name: &str, ctx: &FlowState) -> String {
    match name.to_lowercase().as_str() {
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
