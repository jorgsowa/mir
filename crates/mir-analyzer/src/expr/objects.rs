use super::helpers::extract_string_from_expr;
use super::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use crate::symbol::ReferenceKind;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Type};
use php_ast::owned::{Expr, ExprKind, NewExpr, PropertyAccessExpr, StaticAccessExpr};
use std::sync::Arc;

/// Widen scalar literal atomics to their base type when forming a `new`
/// receiver's generic `type_params`. Carrying a literal type param into the
/// receiver (e.g. `Box<5>` from `new Box(5)`) is over-narrow and risks false
/// positives downstream (e.g. `$box->set(6)` where `set(T)` and `T=5` would
/// wrongly reject `6`). Mirrors `stmt::widen_for_check`, plus `true`/`false`
/// → `bool`.
fn widen_type_param(ty: &Type) -> Type {
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

fn is_valid_class_name_type(ty: &Type) -> bool {
    // Class names must be strings or class-string types. Mixed is allowed
    // since it's already imprecise (matches the static-call path — a `mixed`
    // receiver is a Mixed* concern, not InvalidStringClass). Template params
    // are allowed because their bound may be a class-string.
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

/// Owned equivalent of `expr_can_be_passed_by_reference` for owned `Expr`.
fn expr_can_be_passed_by_reference_owned(expr: &Expr) -> bool {
    matches!(
        expr.kind,
        ExprKind::Variable(_)
            | ExprKind::ArrayAccess(_)
            | ExprKind::PropertyAccess(_)
            | ExprKind::NullsafePropertyAccess(_)
            | ExprKind::StaticPropertyAccess(_)
            | ExprKind::StaticPropertyAccessDynamic { .. }
    )
}

/// Get the name string from an owned `Expr` for Variable/Identifier nodes.
fn expr_name_str(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Variable(s) | ExprKind::Identifier(s) => Some(s.as_ref()),
        _ => None,
    }
}

impl<'a> ExpressionAnalyzer<'a> {
    /// Infer the class-level generic type params for `new Class(...)` from the
    /// constructor argument types.
    ///
    /// Given `@template T` on the class and a constructor parameter typed `T`,
    /// `new Box(5)` binds `T → int` and returns `[int]` (in the class's declared
    /// template-param order) so the result is `Box<int>`. Returns the cached
    /// empty params when the class is non-generic or nothing could be inferred,
    /// preserving the previous behaviour for ordinary instantiations.
    fn infer_new_type_params(
        &mut self,
        fqcn: &Arc<str>,
        ctor_params: Option<&[mir_codebase::definitions::DeclaredParam]>,
        arg_types: &[Type],
        arg_names: &[Option<String>],
        call_span: php_ast::Span,
    ) -> Arc<[Type]> {
        let empty = mir_types::union::empty_type_params();
        // A plain subclass that doesn't redeclare `@template` (`class IntBox
        // extends Box {}`) is still implicitly parameterized the same way
        // `Box` is — walk up to the nearest ancestor that actually declares
        // templates instead of bailing out just because `fqcn` itself has none.
        let class_tps = match crate::db::class_template_params(self.db, fqcn) {
            Some(tps) if !tps.is_empty() => tps,
            _ => return empty,
        };
        let Some(ctor_params) = ctor_params else {
            return empty;
        };

        // Bind class templates ONLY from constructor ARGUMENTS (no bound/mixed
        // fallback). A template the constructor never binds is absent from the
        // map, so it stays `mixed` (bare) in the emitted `type_params` — it must
        // NOT be fabricated to its declared bound, or a later `T`-typed method
        // call would falsely substitute the param to the bound and reject valid
        // args (e.g. `@template T of Base`, `__construct(int $id)` → `new Repo(5)`
        // must be bare `Repo`, never `Repo<Base>`).
        let (mut bindings, unchecked) = crate::generic::infer_arg_template_bindings(
            self.db,
            &class_tps,
            ctor_params,
            arg_types,
            arg_names,
        );
        // A subclass that fixes a generic ancestor via `@extends Box<int>`
        // already determines T=int for every instance of this class,
        // regardless of what this particular constructor call's arguments
        // would otherwise infer — an inherited fixed binding takes priority
        // so a mismatched constructor argument shows up as an arg-type
        // mismatch (check_constructor_args, substituted the same way) rather
        // than silently rebinding T to the bad argument's type and
        // corrupting every later `@return T` on this receiver. This applies
        // even when `fqcn` ALSO declares its own, separate `@template`
        // (`class Mid<U> extends Base<int>` — U is still freshly inferred
        // below, T is independently fixed) — only skipped per-entry when the
        // inherited binding's value is itself self-referential, pointing
        // back at one of `fqcn`'s OWN template params (e.g. `class
        // TypedList { @template T; @implements Collection<T> }`, where T is
        // exactly what THIS constructor call is inferring — merging it here
        // would corrupt it into a self-referential `TypedList<T>`-shaped
        // type before inference even runs).
        let own_template_names: std::collections::HashSet<mir_types::Name> = class_tps
            .iter()
            .map(|tp| mir_types::Name::from(tp.name.as_ref()))
            .collect();
        for (name, ty) in crate::db::inherited_template_bindings(self.db, fqcn, &Default::default())
        {
            let self_referential = ty.contains(
                |a| matches!(a, Atomic::TTemplateParam { name, .. } if own_template_names.contains(name)),
            );
            if !self_referential {
                bindings.insert(name, ty);
            }
        }

        // A class-level `@template T of Bound` was previously enforced only at
        // method-call sites (against a method's OWN template params), never
        // here — the dominant real-world path (constructor-arg inference on
        // `new`) bypassed bound checking entirely. Check it the same way
        // method/function calls do.
        //
        // Restricted to classes declared outside the bundled stubs: a stub
        // constructor with multiple declared PHP overloads (e.g. DatePeriod's
        // DateTimeInterface-pair vs. ISO-8601-string forms) is collapsed by
        // `merge_method_overloads` into a single permissive signature that
        // keeps only the longest overload's param *types* — the other
        // overload's own param types are discarded entirely, not unioned in.
        // Binding a template from an argument that actually satisfies a
        // *different, now-invisible* overload would be a false positive that
        // this approximation can't distinguish from a real violation.
        let violations = crate::generic::check_template_bounds_with_inheritance(
            self.db,
            &bindings,
            &class_tps,
            &unchecked,
            Some(fqcn.as_ref()),
        );
        let is_stub_class = !violations.is_empty()
            && crate::db::class_like_decl_file(self.db, crate::db::Fqcn::from_str(self.db, fqcn))
                .is_some_and(|f| crate::stubs::StubVfs::new().is_stub_file(f.as_ref()));
        for (name, inferred, bound) in violations {
            if is_stub_class {
                continue;
            }
            self.emit(
                IssueKind::InvalidTemplateParam {
                    name: name.to_string(),
                    expected_bound: format!("{bound}"),
                    actual: format!("{inferred}"),
                },
                Severity::Error,
                call_span,
            );
        }

        // Emit the inferred bindings in the class's declared template-param order
        // so `build_class_bindings` zips them back correctly at the call site.
        // A template bound from an argument → its (widened) inferred type;
        // otherwise → `mixed`. Only consider the receiver "concrete" when at
        // least one template was bound from an argument — otherwise keep the
        // bare (un-parameterised) type, preserving prior behaviour for ordinary
        // / non-generic / uninferable instantiations.
        let mut params: Vec<Type> = Vec::with_capacity(class_tps.len());
        let mut any_concrete = false;
        for tp in class_tps.iter() {
            match bindings.get(&mir_types::Name::from(tp.name.as_ref())) {
                Some(ty) if !ty.is_mixed_not_template() => {
                    // Widen scalar literals to their base type so the receiver
                    // does not carry an over-narrow type param (e.g. `new Box(5)`
                    // → `Box<int>`, not `Box<5>`, so a later `$box->set(6)` for
                    // `set(T)` is not wrongly rejected).
                    any_concrete = true;
                    params.push(widen_type_param(ty));
                }
                _ => params.push(Type::mixed()),
            }
        }
        if any_concrete {
            mir_types::union::vec_to_type_params(params)
        } else {
            empty
        }
    }

    pub(super) fn analyze_new(
        &mut self,
        n: &NewExpr,
        call_span: php_ast::Span,
        ctx: &mut FlowState,
    ) -> Type {
        let mut arg_types = crate::call::ARG_TYPES_BUF
            .with(|b| b.borrow_mut().take())
            .unwrap_or_default();
        arg_types.clear();
        let mut sole_spread_ty: Option<Type> = None;
        for a in n.args.iter() {
            let ty = self.analyze(&a.value, ctx);
            crate::call::consume_arg_assignment(&a.value, ctx);
            if a.unpack {
                if n.args.len() == 1 {
                    sole_spread_ty = Some(ty.clone());
                }
                arg_types.push(crate::call::spread_element_type(&ty));
            } else {
                arg_types.push(ty);
            }
        }
        let mut arg_spans: Vec<php_ast::Span> = n.args.iter().map(|a| a.span).collect();
        let mut arg_names: Vec<Option<String>> = n
            .args
            .iter()
            .map(|a| a.name.as_ref().map(crate::parser::name_to_string_owned))
            .collect();
        let mut arg_can_be_byref: Vec<bool> = n
            .args
            .iter()
            .map(|a| expr_can_be_passed_by_reference_owned(&a.value))
            .collect();
        let mut ctor_has_spread = n.args.iter().any(|a| a.unpack);
        let mut ctor_arity_unknown = ctor_has_spread;
        // A sole spread arg over a literal, sequentially-keyed shape can be
        // expanded into one binding per element so each constructor
        // parameter is checked individually instead of only the first (see
        // expand_sole_spread_arg). `ctor_arity_unknown` stays true even
        // after expansion — PHP allows extra/spread positional args, so a
        // concretely-known count still shouldn't trigger
        // TooFew/TooManyArguments.
        if let Some(expanded) = sole_spread_ty.and_then(|t| crate::call::expand_sole_spread_arg(&t))
        {
            arg_spans = crate::call::distinct_spans_for_expansion(arg_spans[0], expanded.len());
            arg_names = vec![None; expanded.len()];
            arg_can_be_byref = vec![false; expanded.len()];
            arg_types = expanded;
            ctor_has_spread = false;
            ctor_arity_unknown = true;
        }

        // Generic type params inferred from constructor arguments, attached to
        // the resulting `TNamedObject` so member-access substitution works.
        let mut inferred_type_params: Arc<[Type]> = mir_types::union::empty_type_params();
        let class_ty = match &n.class.kind {
            ExprKind::Identifier(name) => {
                let resolved = crate::db::resolve_name(self.db, &self.file, name.as_ref());
                let fqcn: Arc<str> = match resolved.as_str() {
                    "self" | "static" => ctx
                        .self_fqcn
                        .clone()
                        .or_else(|| ctx.static_fqcn.clone())
                        .unwrap_or_else(|| Arc::from(resolved.as_str())),
                    "parent" => ctx
                        .parent_fqcn
                        .clone()
                        .unwrap_or_else(|| Arc::from(resolved.as_str())),
                    _ => Arc::from(resolved.as_str()),
                };
                let type_exists = crate::db::class_exists(self.db, fqcn.as_ref());
                if !matches!(resolved.as_str(), "self" | "static" | "parent")
                    && !type_exists
                    && !ctx.is_class_guarded(fqcn.as_ref())
                {
                    self.emit(
                        IssueKind::UndefinedClass {
                            name: resolved.clone(),
                        },
                        Severity::Error,
                        n.class.span,
                    );
                } else if type_exists {
                    let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
                    if let Some(class) = crate::db::find_class_like(self.db, here) {
                        // `new static()` is valid even in an abstract class:
                        // late static binding resolves `static` to the concrete
                        // subclass at runtime, never the abstract class itself.
                        // `new self()` / `new AbstractName()` remain errors.
                        if class.is_class() && class.is_abstract() && resolved.as_str() != "static"
                        {
                            self.emit(
                                IssueKind::AbstractInstantiation {
                                    class: fqcn.to_string(),
                                },
                                Severity::Error,
                                n.class.span,
                            );
                        }
                        if class.is_interface() {
                            self.emit(
                                IssueKind::InterfaceInstantiation {
                                    class: fqcn.to_string(),
                                },
                                Severity::Error,
                                n.class.span,
                            );
                        }
                        if let Some(msg) = class.deprecated() {
                            self.emit(
                                IssueKind::DeprecatedClass {
                                    name: fqcn.to_string(),
                                    message: Some(msg.clone()).filter(|m| !m.is_empty()),
                                },
                                Severity::Info,
                                n.class.span,
                            );
                        }
                        // Check for case mismatch between written name and canonical
                        if let Some((used, canonical_str)) =
                            crate::fqcn_case_mismatch(fqcn.as_ref(), class.fqcn().as_ref())
                        {
                            self.emit(
                                IssueKind::WrongCaseClass {
                                    used,
                                    canonical: canonical_str,
                                },
                                Severity::Info,
                                n.class.span,
                            );
                        }
                    }
                    let ctor_params_and_templates = crate::db::find_method_in_chain(
                        self.db,
                        crate::db::Fqcn::from_str(self.db, fqcn.as_ref()),
                        "__construct",
                    )
                    .map(|(_, s)| {
                        (
                            s.params.to_vec(),
                            s.template_params.clone(),
                            s.no_named_arguments,
                        )
                    });
                    // `new static`/`new self`/`new parent` inside a trait binds
                    // to the using class's constructor (late static binding),
                    // not the trait's — which has none. Skip constructor-arg
                    // validation so passing args isn't flagged against the
                    // trait's (absent) zero-arg implicit constructor.
                    let trait_relative_new =
                        matches!(resolved.as_str(), "self" | "static" | "parent")
                            && crate::flow_state::self_is_trait(self.db, ctx);
                    if let Some((ctor_params, ctor_templates, ctor_no_named_args)) =
                        &ctor_params_and_templates
                    {
                        if !trait_relative_new {
                            // A bare subclass of a generic ancestor fixed via
                            // `@extends Box<int>` already determines T=int for
                            // this exact `new` — substitute it into the
                            // constructor's own param types before arg-checking,
                            // mirroring how every other template-consuming call
                            // site (method/static calls, property read/write,
                            // array-access, foreach) substitutes
                            // `inherited_template_bindings` before using a
                            // class's template params. A constructor-level
                            // `@template T` shadowing the class-level one keeps
                            // its own occurrences unbound so arg inference still
                            // runs for it.
                            let class_tps = crate::db::class_template_params(self.db, &fqcn)
                                .map(|tps| tps.to_vec())
                                .unwrap_or_default();
                            let mut bindings: rustc_hash::FxHashMap<mir_types::Name, Type> =
                                Default::default();
                            // Merged even when `fqcn` ALSO declares its own,
                            // separate `@template` (`class Mid<U> extends
                            // Base<int>` — U is still freshly inferred below,
                            // T is independently fixed) — only skipped
                            // per-entry when the inherited binding's value is
                            // itself self-referential, pointing back at one of
                            // `fqcn`'s OWN template params (e.g. a class
                            // implementing a covariant interface with its own
                            // template forwarded to it, `@implements
                            // Collection<T>`) — there, T is exactly what THIS
                            // constructor-arg inference is meant to bind, and
                            // merging it here would corrupt it into a
                            // self-referential type before inference runs.
                            let own_template_names: std::collections::HashSet<mir_types::Name> =
                                class_tps
                                    .iter()
                                    .map(|tp| mir_types::Name::from(tp.name.as_ref()))
                                    .collect();
                            for (k, v) in crate::db::inherited_template_bindings(
                                self.db,
                                &fqcn,
                                &Default::default(),
                            ) {
                                let self_referential = v.contains(|a| {
                                    matches!(a, Atomic::TTemplateParam { name, .. } if own_template_names.contains(name))
                                });
                                if !self_referential {
                                    bindings.insert(k, v);
                                }
                            }
                            for tp in ctor_templates.iter() {
                                bindings.remove(&mir_types::Name::from(tp.name.as_ref()));
                            }
                            let substituted_ctor_params: Vec<
                                mir_codebase::definitions::DeclaredParam,
                            >;
                            let effective_ctor_params: &[mir_codebase::definitions::DeclaredParam] =
                                if bindings.is_empty() || class_tps.is_empty() {
                                    ctor_params
                                } else {
                                    substituted_ctor_params = ctor_params
                                        .iter()
                                        .map(|p| mir_codebase::definitions::DeclaredParam {
                                            ty: mir_codebase::wrap_param_type(
                                                p.ty.as_ref()
                                                    .map(|t| t.substitute_templates(&bindings)),
                                            ),
                                            out_ty: mir_codebase::wrap_param_type(
                                                p.out_ty
                                                    .as_ref()
                                                    .map(|t| t.substitute_templates(&bindings)),
                                            ),
                                            ..p.clone()
                                        })
                                        .collect();
                                    &substituted_ctor_params
                                };
                            crate::call::check_constructor_args(
                                self,
                                &fqcn,
                                crate::call::CheckArgsParams {
                                    fn_name: "__construct",
                                    params: effective_ctor_params,
                                    arg_types: &arg_types,
                                    arg_spans: &arg_spans,
                                    arg_names: &arg_names,
                                    arg_can_be_byref: &arg_can_be_byref,
                                    call_span,
                                    has_spread: ctor_has_spread,
                                    arity_unknown: ctor_arity_unknown,
                                    template_params: ctor_templates,
                                    no_named_arguments: *ctor_no_named_args,
                                },
                            );
                        }
                    } else if !arg_types.is_empty()
                        && !n.args.iter().any(|a| a.unpack)
                        && !trait_relative_new
                        // `new static(args)` may construct a subclass that
                        // declares a constructor, even when the current class
                        // has none — don't flag it against the implicit 0-arg
                        // constructor (`new self` / a named class still are).
                        && resolved.as_str() != "static"
                        && crate::db::class_exists(self.db, fqcn.as_ref())
                    {
                        // Class has no constructor but arguments were passed —
                        // PHP's implicit constructor accepts zero arguments.
                        self.emit(
                            mir_issues::IssueKind::TooManyArguments {
                                fn_name: format!("{fqcn}::__construct"),
                                expected: 0,
                                actual: arg_types.len(),
                            },
                            mir_issues::Severity::Error,
                            call_span,
                        );
                    }
                    // Infer class-level generic type params from the constructor
                    // arguments (e.g. `new Box(5)` → `Box<int>` for `@template T`
                    // with `__construct(T $value)`). This lets later member access
                    // (`$b->get()`) substitute the receiver's bindings into return
                    // types, including UNANNOTATED inferred returns.
                    inferred_type_params = self.infer_new_type_params(
                        &fqcn,
                        ctor_params_and_templates
                            .as_ref()
                            .map(|(p, _, _)| p.as_slice()),
                        &arg_types,
                        &arg_names,
                        call_span,
                    );
                }
                crate::call::ARG_TYPES_BUF.with(|b| {
                    let mut g = b.borrow_mut();
                    if g.as_ref().map_or(0, |v| v.capacity()) < arg_types.capacity() {
                        *g = Some(arg_types);
                    }
                });
                let ty = Type::single(Atomic::TNamedObject {
                    fqcn: mir_types::Name::from(fqcn.as_ref()),
                    type_params: std::mem::replace(
                        &mut inferred_type_params,
                        mir_types::union::empty_type_params(),
                    ),
                });
                self.record_symbol(
                    n.class.span,
                    ReferenceKind::ClassReference(fqcn.clone()),
                    ty.clone(),
                );
                self.record_ref(Arc::from(format!("cls:{fqcn}")), n.class.span);
                // A `new X(...)` site is also a constructor call: record it
                // under the method key so find-references on `__construct`
                // resolves instantiation sites without an AST re-walk.
                self.record_ref(Arc::from(format!("meth:{fqcn}::__construct")), n.class.span);
                ty
            }
            _ => {
                let ty = self.analyze(&n.class, ctx);
                // Check if the expression could evaluate to a valid class name
                // (but skip anonymous class definitions, which are valid)
                if !matches!(n.class.kind, ExprKind::AnonymousClass(_))
                    && !is_valid_class_name_type(&ty)
                {
                    self.emit(
                        IssueKind::InvalidStringClass {
                            actual: ty.to_string(),
                        },
                        Severity::Warning,
                        n.class.span,
                    );
                }
                // Note: TClassString<AbstractClass> is valid for `new $var` because the
                // class-string constraint means the held class-name IS-A AbstractClass, not
                // that it IS AbstractClass itself. The concrete runtime class may be any
                // non-abstract subclass, so no AbstractInstantiation check here.

                // `new $class()` where `$class` holds a known class-string
                // (`$class = Foo::class;`) is a real reference to `Foo` — record it,
                // or a class instantiated only this way is falsely flagged unused
                // with no go-to-definition from this call site.
                for atomic in &ty.types {
                    if let Atomic::TClassString(Some(fqcn)) = atomic {
                        self.record_ref(Arc::from(format!("cls:{fqcn}")), n.class.span);
                        self.record_symbol(
                            n.class.span,
                            ReferenceKind::ClassReference(Arc::from(fqcn.as_ref())),
                            Type::single(Atomic::TClassString(None)),
                        );
                    }
                }
                Type::single(Atomic::TObject)
            }
        };
        class_ty
    }

    pub(super) fn analyze_property_access(
        &mut self,
        pa: &PropertyAccessExpr,
        expr_span: php_ast::Span,
        ctx: &mut FlowState,
    ) -> Type {
        let obj_ty = self.analyze(&pa.object, ctx);
        self.record_receiver_type(pa.object.span, pa.property.span, obj_ty.clone());
        let prop_name =
            extract_string_from_expr(&pa.property).unwrap_or_else(|| "<dynamic>".to_string());

        if obj_ty.is_mixed() {
            let is_only_template_params = obj_ty
                .types
                .iter()
                .all(|t| matches!(t, Atomic::TTemplateParam { .. }));
            if !is_only_template_params {
                self.emit(
                    IssueKind::MixedPropertyFetch {
                        property: prop_name.clone(),
                    },
                    Severity::Info,
                    expr_span,
                );
            }
            // Unknowable receiver — record a name-only fallback so
            // find-references on any `X::$name` can surface this access.
            if prop_name != "<dynamic>" {
                self.record_ref(Arc::from(format!("propname:{prop_name}")), pa.property.span);
            }
            return Type::mixed();
        }

        // InvalidPropertyFetch: all types are scalar/non-object
        if !obj_ty.is_mixed()
            && !obj_ty.types.is_empty()
            && obj_ty.types.iter().all(|a| {
                matches!(
                    a,
                    Atomic::TInt
                        | Atomic::TLiteralInt(_)
                        | Atomic::TIntRange { .. }
                        | Atomic::TPositiveInt
                        | Atomic::TFloat
                        | Atomic::TIntegralFloat
                        | Atomic::TLiteralFloat(_, _)
                        | Atomic::TString
                        | Atomic::TNonEmptyString
                        | Atomic::TNumericString
                        | Atomic::TLiteralString(_)
                        | Atomic::TBool
                        | Atomic::TTrue
                        | Atomic::TFalse
                        | Atomic::TArray { .. }
                        | Atomic::TNonEmptyArray { .. }
                        | Atomic::TList { .. }
                        | Atomic::TNonEmptyList { .. }
                        | Atomic::TKeyedArray { .. }
                )
            })
        {
            self.emit(
                IssueKind::InvalidPropertyFetch {
                    ty: obj_ty.to_string(),
                },
                Severity::Error,
                expr_span,
            );
            return Type::mixed();
        }

        if obj_ty.contains(|t| matches!(t, Atomic::TNull)) && obj_ty.is_single() {
            self.emit(
                IssueKind::NullPropertyFetch {
                    property: prop_name.clone(),
                },
                Severity::Error,
                expr_span,
            );
            return Type::mixed();
        }
        if obj_ty.is_nullable() {
            self.emit(
                IssueKind::PossiblyNullPropertyFetch {
                    property: prop_name.clone(),
                },
                Severity::Info,
                expr_span,
            );
        }

        if prop_name == "<dynamic>" {
            self.analyze(&pa.property, ctx);
            self.record_dynamic_member_access(&obj_ty, pa.property.span);
            return Type::mixed();
        }
        let non_null_ty = obj_ty.remove_null();
        let mut declaring = None;
        let resolved =
            self.resolve_property_type(&non_null_ty, &prop_name, pa.property.span, &mut declaring);

        // If we have a narrowed type for this property access ($var->prop),
        // return it instead of the declared type.
        let mut resolved = if let ExprKind::Variable(obj_var) = &pa.object.kind {
            ctx.get_prop_refined(obj_var.as_ref(), &prop_name)
                .cloned()
                .unwrap_or(resolved)
        } else {
            resolved
        };
        // PHP 8 reads a plain `->` access on a null receiver as a warning
        // (not fatal), still evaluating to null — same observable value as
        // `?->`'s short-circuit (see analyze_nullsafe_property_access, which
        // this mirrors). So the expression's type must include null too
        // whenever the receiver itself could be null. Guarded on `obj_ty`
        // (not `non_null_ty`) so a receiver already narrowed non-null (e.g.
        // inside `if ($obj->prop !== null)`, which also narrows `$obj`)
        // doesn't get re-widened.
        if obj_ty.is_nullable() {
            resolved.add_type(Atomic::TNull);
        }

        for atomic in &non_null_ty.types {
            if let Atomic::TNamedObject { fqcn, .. } = atomic {
                let declaring_class = declaring.take().unwrap_or_else(|| Arc::from(fqcn.as_ref()));
                self.record_symbol(
                    pa.property.span,
                    ReferenceKind::PropertyAccess {
                        class: declaring_class,
                        property: Arc::from(prop_name.as_str()),
                    },
                    resolved.clone(),
                );
                break;
            }
        }
        resolved
    }

    pub(super) fn analyze_nullsafe_property_access(
        &mut self,
        pa: &PropertyAccessExpr,
        ctx: &mut FlowState,
    ) -> Type {
        let obj_ty = self.analyze(&pa.object, ctx);
        self.record_receiver_type(pa.object.span, pa.property.span, obj_ty.clone());
        let prop_name =
            extract_string_from_expr(&pa.property).unwrap_or_else(|| "<dynamic>".to_string());
        if prop_name == "<dynamic>" {
            self.analyze(&pa.property, ctx);
            self.record_dynamic_member_access(&obj_ty, pa.property.span);
            return Type::mixed();
        }
        let non_null_ty = obj_ty.remove_null();
        let mut declaring = None;
        let resolved =
            self.resolve_property_type(&non_null_ty, &prop_name, pa.property.span, &mut declaring);

        // If we have a narrowed type for this property access ($var?->prop),
        // return it instead of the declared type — matching the plain `->`
        // path in analyze_property_access above.
        let mut prop_ty = if let ExprKind::Variable(obj_var) = &pa.object.kind {
            ctx.get_prop_refined(obj_var.as_ref(), &prop_name)
                .cloned()
                .unwrap_or(resolved)
        } else {
            resolved
        };
        // Only the receiver's own nullability can make `$obj?->prop` evaluate to
        // null — if `$obj` can never be null, this is exactly `$obj->prop`'s type,
        // narrowed-or-not. Adding TNull unconditionally clobbered a narrowed
        // non-null property type back into a nullable one.
        if obj_ty.is_nullable() {
            prop_ty.add_type(Atomic::TNull);
        }
        for atomic in &non_null_ty.types {
            if let Atomic::TNamedObject { fqcn, .. } = atomic {
                let declaring_class = declaring.take().unwrap_or_else(|| Arc::from(fqcn.as_ref()));
                self.record_symbol(
                    pa.property.span,
                    ReferenceKind::PropertyAccess {
                        class: declaring_class,
                        property: Arc::from(prop_name.as_str()),
                    },
                    prop_ty.clone(),
                );
                break;
            }
        }
        prop_ty
    }

    pub(super) fn analyze_static_property_access(
        &mut self,
        spa: &StaticAccessExpr,
        ctx: &FlowState,
    ) -> Type {
        let mut result_ty = Type::mixed();
        if let ExprKind::Identifier(id) = &spa.class.kind {
            let resolved = crate::db::resolve_name(self.db, &self.file, id.as_ref());
            if matches!(resolved.as_str(), "self" | "static" | "parent") {
                // Resolve the relative keyword through the FlowState and record
                // the property read so a `self::$prop` access counts as a use
                // (otherwise a private static property is wrongly reported
                // UnusedProperty).
                let fqcn_opt = match resolved.as_str() {
                    "self" | "static" => ctx.self_fqcn.clone().or_else(|| ctx.static_fqcn.clone()),
                    "parent" => ctx.parent_fqcn.clone(),
                    _ => None,
                };
                if let Some(fqcn) = fqcn_opt {
                    self.record_symbol(
                        spa.class.span,
                        ReferenceKind::ClassReference(fqcn.clone()),
                        Type::single(Atomic::TClassString(None)),
                    );
                    if let Some(prop_name) = expr_name_str(&spa.member) {
                        let prop_name = prop_name.trim_start_matches('$');
                        // Key by the declaring owner, not `self`/`static`'s own
                        // class — a `self::$prop` access inside a subclass for a
                        // `$prop` declared on the parent must record
                        // `prop:Parent::prop`, matching `record_static_prop_access`.
                        let mut owner = fqcn.clone();
                        if let Some(refined) = ctx.get_prop_refined(fqcn.as_ref(), prop_name) {
                            result_ty = refined.clone();
                        } else {
                            let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
                            if let Some((prop_owner, p)) =
                                crate::db::find_property_in_chain(self.db, here, prop_name)
                            {
                                owner = prop_owner;
                                result_ty = p.ty.as_deref().cloned().unwrap_or_else(Type::mixed);
                            }
                        }
                        self.record_ref(
                            Arc::from(format!("prop:{}::{}", owner, prop_name)),
                            spa.member.span,
                        );
                        self.record_symbol(
                            spa.member.span,
                            ReferenceKind::PropertyAccess {
                                class: owner,
                                property: Arc::from(prop_name),
                            },
                            result_ty.clone(),
                        );
                        self.record_receiver_type(
                            spa.class.span,
                            spa.member.span,
                            Type::single(Atomic::TClassString(Some(mir_types::Name::from(
                                fqcn.as_ref(),
                            )))),
                        );
                    }
                }
            } else if !crate::db::class_exists(self.db, &resolved)
                && !ctx.is_class_guarded(resolved.as_str())
            {
                self.emit(
                    IssueKind::UndefinedClass { name: resolved },
                    Severity::Error,
                    spa.class.span,
                );
            } else {
                result_ty = self.record_static_prop_access(
                    Arc::from(resolved.as_str()),
                    &spa.class,
                    &spa.member,
                    ctx,
                );
            }
        } else if let ExprKind::Variable(var_name) = &spa.class.kind {
            // `$cls::$prop` — derive the FQCN(s) from the variable's inferred
            // type, mirroring the `$obj::CONST` handling in
            // `analyze_class_const_access`. Covers both an object instance
            // (`$obj::$prop`) and a class-string (`$cls = self::class;`).
            // Without this, a property read only through a variable holding
            // the class was never checked (existence/visibility/deprecation)
            // and never recorded as a usage — falsely flagging the property
            // (and, if otherwise unreferenced, the class) as unused.
            let var_ty = ctx.get_var(var_name.as_ref());
            let mut result = Type::empty();
            let mut any = false;
            for atomic in &var_ty.types {
                let fqcn = atomic.named_object_fqcn().or_else(|| match atomic {
                    Atomic::TClassString(Some(fqcn)) => Some(fqcn.as_ref()),
                    _ => None,
                });
                if let Some(fqcn) = fqcn {
                    any = true;
                    let ty = self.record_static_prop_access(
                        Arc::from(fqcn),
                        &spa.class,
                        &spa.member,
                        ctx,
                    );
                    result.merge_with(&ty);
                }
            }
            if any {
                result_ty = result;
            }
        }
        result_ty
    }

    /// Record and resolve a static property access (`Class::$prop`) once the
    /// concrete class FQCN is known — shared by the plain-identifier class
    /// name path and the object-instance/class-string variable path.
    fn record_static_prop_access(
        &mut self,
        resolved: Arc<str>,
        class_expr: &Expr,
        member_expr: &Expr,
        ctx: &FlowState,
    ) -> Type {
        let mut result_ty = Type::mixed();
        self.record_ref(Arc::from(format!("cls:{resolved}")), class_expr.span);
        self.record_symbol(
            class_expr.span,
            ReferenceKind::ClassReference(resolved.clone()),
            Type::single(Atomic::TClassString(None)),
        );
        self.record_receiver_type(
            class_expr.span,
            member_expr.span,
            Type::single(Atomic::TClassString(Some(mir_types::Name::from(
                resolved.as_ref(),
            )))),
        );
        if let Some(prop_name) = expr_name_str(member_expr) {
            // Key the reference/symbol by the property's declaring owner, not
            // the accessed-through class — `Child::$prop` for a `$prop`
            // declared on `Parent` must record `prop:Parent::prop`, or
            // find-references from the declaring property never sees usages
            // reached only through a subclass name (the same fix already
            // applied to constant/instance-property access above).
            let mut owner = resolved.clone();
            if let Some(refined) = ctx.get_prop_refined(resolved.as_ref(), prop_name) {
                result_ty = refined.clone();
            } else {
                let here = crate::db::Fqcn::from_str(self.db, resolved.as_ref());
                if let Some(p) = crate::db::find_property_in_chain(self.db, here, prop_name) {
                    owner = p.0.clone();
                    if let Some(msg) = &p.1.deprecated {
                        self.emit(
                            IssueKind::DeprecatedProperty {
                                class: resolved.to_string(),
                                property: prop_name.to_string(),
                                message: Some(msg.clone()).filter(|m| !m.is_empty()),
                            },
                            Severity::Info,
                            member_expr.span,
                        );
                    }
                    result_ty = p.1.ty.as_deref().cloned().unwrap_or_else(Type::mixed);
                }
            }
            self.record_ref(
                Arc::from(format!("prop:{}::{}", owner, prop_name)),
                member_expr.span,
            );
            self.record_symbol(
                member_expr.span,
                ReferenceKind::PropertyAccess {
                    class: owner,
                    property: Arc::from(prop_name),
                },
                result_ty.clone(),
            );
        }
        result_ty
    }

    pub(super) fn analyze_class_const_access(
        &mut self,
        cca: &StaticAccessExpr,
        expr_span: php_ast::Span,
        ctx: &FlowState,
    ) -> Type {
        if expr_name_str(&cca.member) == Some("class") {
            if let ExprKind::Identifier(id) = &cca.class.kind {
                let resolved = crate::db::resolve_name(self.db, &self.file, id.as_ref());
                if resolved.as_str() == "parent"
                    && ctx.parent_fqcn.is_none()
                    && ctx.self_fqcn.is_some()
                {
                    self.emit(IssueKind::ParentNotFound, Severity::Error, cca.class.span);
                }
                if !matches!(resolved.as_str(), "self" | "static" | "parent") {
                    // `Foo::class` is a PHP compile-time string constant — the class
                    // need not be loaded or even defined.  Never emit UndefinedClass
                    // for `::class` expressions.
                    if crate::db::class_exists(self.db, &resolved) {
                        // Check if the class is deprecated
                        let here = crate::db::Fqcn::from_str(self.db, resolved.as_str());
                        if let Some(class) = crate::db::find_class_like(self.db, here) {
                            if let Some(msg) = class.deprecated() {
                                self.emit(
                                    IssueKind::DeprecatedClass {
                                        name: resolved.clone(),
                                        message: Some(msg.clone()).filter(|m| !m.is_empty()),
                                    },
                                    Severity::Info,
                                    cca.class.span,
                                );
                            }
                        }
                    }
                    self.record_ref(Arc::from(format!("cls:{resolved}")), cca.class.span);
                    // Without this, go-to-definition/hover on the class name inside
                    // `Foo::class` resolved nothing, unlike every other class-name
                    // position (`new Foo`, `instanceof Foo`, `Foo::method()`, …).
                    self.record_symbol(
                        cca.class.span,
                        ReferenceKind::ClassReference(Arc::from(resolved.as_str())),
                        Type::single(Atomic::TClassString(None)),
                    );
                }
                // `self`/`static`/`parent::class` must carry the actual enclosing
                // class, not the literal pseudo-name — otherwise a variable holding
                // `self::class` types as the unresolvable `class-string<self>`
                // instead of e.g. `class-string<Foo>`, breaking every downstream
                // consumer of that variable (including `$var::CONST`/`$var::$prop`).
                let concrete = match resolved.as_str() {
                    "self" | "static" => ctx
                        .self_fqcn
                        .clone()
                        .or_else(|| ctx.static_fqcn.clone())
                        .unwrap_or_else(|| Arc::from(resolved.as_str())),
                    "parent" => ctx
                        .parent_fqcn
                        .clone()
                        .unwrap_or_else(|| Arc::from(resolved.as_str())),
                    _ => Arc::from(resolved.as_str()),
                };
                return Type::single(Atomic::TClassString(Some(mir_types::Name::from(
                    concrete.as_ref(),
                ))));
            }

            // For $obj::class, derive class-string<T> from the object's declared type.
            if let ExprKind::Variable(var_name) = &cca.class.kind {
                let obj_ty = ctx.get_var(var_name.as_ref());
                let mut result = Type::empty();
                for atomic in &obj_ty.types {
                    match atomic {
                        Atomic::TNamedObject { fqcn, .. }
                        | Atomic::TSelf { fqcn }
                        | Atomic::TStaticObject { fqcn } => {
                            result.add_type(Atomic::TClassString(Some(*fqcn)));
                        }
                        _ => {}
                    }
                }
                if !result.types.is_empty() {
                    return result;
                }
            }

            return Type::single(Atomic::TClassString(None));
        }

        let const_name = match expr_name_str(&cca.member) {
            Some(n) => n.to_string(),
            None => return Type::mixed(),
        };

        let fqcn = match &cca.class.kind {
            ExprKind::Identifier(id) => {
                let resolved = crate::db::resolve_name(self.db, &self.file, id.as_ref());
                match resolved.as_str() {
                    "self" | "static" => {
                        let Some(self_fqcn) = &ctx.self_fqcn else {
                            return Type::mixed();
                        };
                        let here = crate::db::Fqcn::from_str(self.db, self_fqcn);
                        let found =
                            crate::db::find_class_constant_in_chain(self.db, here, &const_name);
                        // Inside a trait, `self::`/`static::CONST` may be
                        // defined on the using class via late static binding,
                        // not the trait itself — skip the undefined check.
                        if found.is_none()
                            && !crate::db::has_unknown_ancestor(self.db, self_fqcn)
                            && !crate::flow_state::self_is_trait(self.db, ctx)
                        {
                            self.emit(
                                IssueKind::UndefinedConstant {
                                    name: format!("{self_fqcn}::{const_name}"),
                                },
                                Severity::Error,
                                expr_span,
                            );
                        }
                        if let Some((_, ref c)) = found {
                            if let Some(msg) = &c.deprecated {
                                self.emit(
                                    IssueKind::DeprecatedConstant {
                                        class: self_fqcn.to_string(),
                                        constant: const_name.clone(),
                                        message: Some(msg.clone()).filter(|m| !m.is_empty()),
                                    },
                                    Severity::Info,
                                    cca.member.span,
                                );
                            }
                        }
                        let const_ty = found
                            .as_ref()
                            .map(|(_, c)| c.ty.clone())
                            .unwrap_or_else(Type::mixed);
                        // Key against the resolved *declaring* class (may be a
                        // trait providing the constant), not self_fqcn — see
                        // record_object_const_access for why.
                        let owner: Arc<str> = found
                            .as_ref()
                            .map(|(owner_fqcn, _)| owner_fqcn.clone())
                            .unwrap_or_else(|| self_fqcn.clone());
                        self.record_ref(
                            Arc::from(format!("cnst:{owner}::{const_name}")),
                            cca.member.span,
                        );
                        self.record_symbol(
                            cca.member.span,
                            ReferenceKind::ConstantAccess {
                                class: owner,
                                constant: Arc::from(const_name.as_str()),
                            },
                            const_ty.clone(),
                        );
                        return const_ty;
                    }
                    "parent" => {
                        let Some(parent_fqcn) = &ctx.parent_fqcn else {
                            if ctx.self_fqcn.is_some() {
                                self.emit(
                                    IssueKind::ParentNotFound,
                                    Severity::Error,
                                    cca.class.span,
                                );
                            }
                            return Type::mixed();
                        };
                        let here = crate::db::Fqcn::from_str(self.db, parent_fqcn);
                        let found =
                            crate::db::find_class_constant_in_chain(self.db, here, &const_name);
                        if found.is_none() && !crate::db::has_unknown_ancestor(self.db, parent_fqcn)
                        {
                            self.emit(
                                IssueKind::UndefinedConstant {
                                    name: format!("{parent_fqcn}::{const_name}"),
                                },
                                Severity::Error,
                                expr_span,
                            );
                        }
                        if let Some((_, ref c)) = found {
                            if let Some(msg) = &c.deprecated {
                                self.emit(
                                    IssueKind::DeprecatedConstant {
                                        class: parent_fqcn.to_string(),
                                        constant: const_name.clone(),
                                        message: Some(msg.clone()).filter(|m| !m.is_empty()),
                                    },
                                    Severity::Info,
                                    cca.member.span,
                                );
                            }
                        }
                        let const_ty = found
                            .as_ref()
                            .map(|(_, c)| c.ty.clone())
                            .unwrap_or_else(Type::mixed);
                        // Key against the resolved *declaring* class (may be a
                        // trait providing the constant), not parent_fqcn — see
                        // record_object_const_access for why.
                        let owner: Arc<str> = found
                            .as_ref()
                            .map(|(owner_fqcn, _)| owner_fqcn.clone())
                            .unwrap_or_else(|| parent_fqcn.clone());
                        self.record_ref(
                            Arc::from(format!("cnst:{owner}::{const_name}")),
                            cca.member.span,
                        );
                        self.record_symbol(
                            cca.member.span,
                            ReferenceKind::ConstantAccess {
                                class: owner,
                                constant: Arc::from(const_name.as_str()),
                            },
                            const_ty.clone(),
                        );
                        return const_ty;
                    }
                    _ => resolved,
                }
            }
            // `$obj::CONST` — derive the FQCN(s) from the object's inferred
            // type, mirroring the `$obj::class` handling a few lines above.
            // Without this, a constant read only through an object-instance
            // variable was never checked (existence/visibility/deprecation)
            // and never recorded as a usage. Also handles `$cls::CONST` where
            // `$cls` holds a class-string (e.g. `$cls = self::class;`) — the
            // same access form, just via a string rather than an instance.
            ExprKind::Variable(var_name) => {
                let obj_ty = ctx.get_var(var_name.as_ref());
                let mut result = Type::empty();
                let mut any = false;
                for atomic in &obj_ty.types {
                    let fqcn = atomic.named_object_fqcn().or_else(|| match atomic {
                        Atomic::TClassString(Some(fqcn)) => Some(fqcn.as_ref()),
                        _ => None,
                    });
                    if let Some(fqcn) = fqcn {
                        any = true;
                        let const_ty = self.record_object_const_access(
                            fqcn,
                            &const_name,
                            cca.class.span,
                            cca.member.span,
                            expr_span,
                            ctx,
                        );
                        result.merge_with(&const_ty);
                    }
                }
                return if any { result } else { Type::mixed() };
            }
            _ => return Type::mixed(),
        };

        if !crate::db::class_exists(self.db, &fqcn) && !ctx.is_class_guarded(fqcn.as_str()) {
            self.emit(
                IssueKind::UndefinedClass { name: fqcn },
                Severity::Error,
                cca.class.span,
            );
            return Type::mixed();
        }

        // A trait can declare constants for the classes that `use` it (PHP
        // 8.2+), but the trait itself is never a valid constant-access
        // target — `HasFoo::FOO` is a hard fatal regardless of whether FOO
        // exists, so this must short-circuit before the existence lookup
        // below, which would otherwise treat the trait like any other class.
        if crate::db::class_kind(self.db, &fqcn).is_some_and(|k| k.is_trait) {
            self.emit(
                IssueKind::TraitConstantAccessedDirectly {
                    trait_name: fqcn,
                    constant: const_name,
                },
                Severity::Error,
                expr_span,
            );
            return Type::mixed();
        }

        self.record_ref(Arc::from(format!("cls:{fqcn}")), cca.class.span);

        let here = crate::db::Fqcn::from_str(self.db, &fqcn);
        // Check if the class is deprecated
        if let Some(class) = crate::db::find_class_like(self.db, here) {
            if let Some(msg) = class.deprecated() {
                self.emit(
                    IssueKind::DeprecatedClass {
                        name: fqcn.clone(),
                        message: Some(msg.clone()).filter(|m| !m.is_empty()),
                    },
                    Severity::Info,
                    cca.class.span,
                );
            }
        }
        let here = crate::db::Fqcn::from_str(self.db, &fqcn);
        let found = crate::db::find_class_constant_in_chain(self.db, here, &const_name);
        // Key the `cnst:` reference against the resolved *declaring* class, not
        // the literal receiver — see record_object_const_access for why.
        let owner: Arc<str> = found
            .as_ref()
            .map(|(owner_fqcn, _)| owner_fqcn.clone())
            .unwrap_or_else(|| Arc::from(fqcn.as_str()));
        self.record_ref(
            Arc::from(format!("cnst:{owner}::{const_name}")),
            cca.member.span,
        );
        // Check if the constant is deprecated
        if let Some((ref owner_fqcn, ref c)) = found {
            if let Some(msg) = &c.deprecated {
                self.emit(
                    IssueKind::DeprecatedConstant {
                        class: fqcn.clone(),
                        constant: const_name.clone(),
                        message: Some(msg.clone()).filter(|m| !m.is_empty()),
                    },
                    Severity::Info,
                    cca.member.span,
                );
            }
            // Visibility check: private constants are only accessible from the
            // declaring class; protected constants only from the declaring class
            // or its subclasses.
            use mir_codebase::definitions::Visibility;
            let inaccessible = match c.visibility {
                Some(Visibility::Private) => {
                    // Accessible only from the exact declaring class
                    ctx.self_fqcn
                        .as_deref()
                        .map(|s| !s.eq_ignore_ascii_case(owner_fqcn))
                        .unwrap_or(true)
                }
                Some(Visibility::Protected) => {
                    // Accessible from the declaring class or a subclass
                    let caller = ctx.self_fqcn.as_deref().unwrap_or("");
                    if caller.is_empty() {
                        true
                    } else {
                        !crate::db::extends_or_implements(self.db, caller, owner_fqcn)
                            && !caller.eq_ignore_ascii_case(owner_fqcn)
                    }
                }
                _ => false,
            };
            if inaccessible {
                self.emit(
                    IssueKind::InaccessibleClassConstant {
                        class: fqcn.clone(),
                        constant: const_name.clone(),
                    },
                    Severity::Error,
                    cca.member.span,
                );
            }
        }
        let const_ty = found
            .as_ref()
            .map(|(_, c)| c.ty.clone())
            .unwrap_or_else(Type::mixed);

        self.record_symbol(
            cca.member.span,
            ReferenceKind::ConstantAccess {
                class: owner,
                constant: Arc::from(const_name.as_str()),
            },
            const_ty.clone(),
        );

        if found.is_none() && !crate::db::has_unknown_ancestor(self.db, &fqcn) {
            self.emit(
                IssueKind::UndefinedConstant {
                    name: format!("{fqcn}::{const_name}"),
                },
                Severity::Error,
                expr_span,
            );
        }
        const_ty
    }

    /// Constant access through an object-typed receiver (`$obj::CONST`).
    /// `fqcn` comes from the receiver's already-resolved type, so unlike the
    /// class-name-token path in `analyze_class_const_access` there's no
    /// separate `UndefinedClass` check here.
    fn record_object_const_access(
        &mut self,
        fqcn: &str,
        const_name: &str,
        class_span: php_ast::Span,
        member_span: php_ast::Span,
        expr_span: php_ast::Span,
        ctx: &FlowState,
    ) -> Type {
        if crate::db::class_kind(self.db, fqcn).is_some_and(|k| k.is_trait) {
            self.emit(
                IssueKind::TraitConstantAccessedDirectly {
                    trait_name: fqcn.to_string(),
                    constant: const_name.to_string(),
                },
                Severity::Error,
                expr_span,
            );
            return Type::mixed();
        }

        self.record_ref(Arc::from(format!("cls:{fqcn}")), class_span);

        let here = crate::db::Fqcn::from_str(self.db, fqcn);
        let found = crate::db::find_class_constant_in_chain(self.db, here, const_name);
        // Key the `cnst:` reference against the resolved *declaring* class, not
        // the literal receiver — a `Trait::CONST` access is never legal PHP, so
        // a constant provided by a used trait is only ever read through the
        // consuming class. Without this, find-references from the trait's own
        // constant declaration never sees any of its external usages.
        let owner: Arc<str> = found
            .as_ref()
            .map(|(owner_fqcn, _)| owner_fqcn.clone())
            .unwrap_or_else(|| Arc::from(fqcn));
        self.record_ref(
            Arc::from(format!("cnst:{owner}::{const_name}")),
            member_span,
        );
        if let Some((ref owner_fqcn, ref c)) = found {
            if let Some(msg) = &c.deprecated {
                self.emit(
                    IssueKind::DeprecatedConstant {
                        class: fqcn.to_string(),
                        constant: const_name.to_string(),
                        message: Some(msg.clone()).filter(|m| !m.is_empty()),
                    },
                    Severity::Info,
                    member_span,
                );
            }
            use mir_codebase::definitions::Visibility;
            let inaccessible = match c.visibility {
                Some(Visibility::Private) => ctx
                    .self_fqcn
                    .as_deref()
                    .map(|s| !s.eq_ignore_ascii_case(owner_fqcn))
                    .unwrap_or(true),
                Some(Visibility::Protected) => {
                    let caller = ctx.self_fqcn.as_deref().unwrap_or("");
                    if caller.is_empty() {
                        true
                    } else {
                        !crate::db::extends_or_implements(self.db, caller, owner_fqcn)
                            && !caller.eq_ignore_ascii_case(owner_fqcn)
                    }
                }
                _ => false,
            };
            if inaccessible {
                self.emit(
                    IssueKind::InaccessibleClassConstant {
                        class: fqcn.to_string(),
                        constant: const_name.to_string(),
                    },
                    Severity::Error,
                    member_span,
                );
            }
        }

        let const_ty = found
            .as_ref()
            .map(|(_, c)| c.ty.clone())
            .unwrap_or_else(Type::mixed);

        self.record_symbol(
            member_span,
            ReferenceKind::ConstantAccess {
                class: owner,
                constant: Arc::from(const_name),
            },
            const_ty.clone(),
        );

        if found.is_none() && !crate::db::has_unknown_ancestor(self.db, fqcn) {
            self.emit(
                IssueKind::UndefinedConstant {
                    name: format!("{fqcn}::{const_name}"),
                },
                Severity::Error,
                expr_span,
            );
        }

        const_ty
    }

    /// `declaring_class` is set to the FQCN of the class that declares the
    /// property when the inheritance-chain lookup resolves it — reused by the
    /// callers for symbol recording so the chain is only walked once.
    pub(super) fn resolve_property_type(
        &mut self,
        obj_ty: &Type,
        prop_name: &str,
        span: php_ast::Span,
        declaring_class: &mut Option<Arc<str>>,
    ) -> Type {
        for atomic in &obj_ty.types {
            match atomic {
                Atomic::TNamedObject { fqcn, type_params }
                    if crate::db::class_kind(self.db, fqcn.as_ref())
                        .is_some_and(|k| !k.is_interface && !k.is_trait && !k.is_enum) =>
                {
                    let prop_result = crate::db::find_property_in_chain(
                        self.db,
                        crate::db::Fqcn::new(self.db, *fqcn),
                        prop_name,
                    );
                    if let Some((owner, p)) = prop_result {
                        if let Some(msg) = &p.deprecated {
                            self.emit(
                                IssueKind::DeprecatedProperty {
                                    class: fqcn.to_string(),
                                    property: prop_name.to_string(),
                                    message: Some(msg.clone()).filter(|m| !m.is_empty()),
                                },
                                Severity::Info,
                                span,
                            );
                        }
                        let ty = p.ty.as_deref().cloned().unwrap_or_else(Type::mixed);
                        // Substitute the receiver's own concrete type params (e.g.
                        // `Box<int>`'s `T → int`) into a property declared with a
                        // bare `@var T` — plus, when the property is inherited from
                        // an ancestor with its own separate template, resolve THAT
                        // ancestor's params through the same `@extends`/`@implements`
                        // chain walk used everywhere else for inherited bindings.
                        // Run this even when the receiver itself has no own type
                        // params (a bare subclass) — `inherited_template_bindings`
                        // still needs to resolve a `@extends Box<int>`-fixed
                        // ancestor template with no receiver-supplied args at all.
                        let class_tps = crate::db::class_template_params(self.db, fqcn.as_ref())
                            .unwrap_or_default();
                        let own_bindings: rustc_hash::FxHashMap<mir_types::Name, Type> = class_tps
                            .iter()
                            .zip(type_params.iter())
                            .map(|(tp, t)| (tp.name, t.clone()))
                            .collect();
                        let inherited = crate::db::inherited_template_bindings(
                            self.db,
                            fqcn.as_ref(),
                            &own_bindings,
                        );
                        let mut substitution = own_bindings.clone();
                        if owner.as_ref() == fqcn.as_ref() {
                            // `prop_name` is declared directly on the receiver's own
                            // class — a bare template name in its docblock is the
                            // receiver's OWN template, so it must win over a
                            // same-named but unrelated ancestor template (only fill
                            // in names own_bindings doesn't have).
                            for (k, v) in inherited {
                                substitution.entry(k).or_insert(v);
                            }
                        } else {
                            // `prop_name` is inherited from `owner` — a bare template
                            // name in ITS docblock is scoped to `owner`'s own
                            // declaration, which the ancestor-chain walk resolves; it
                            // must win over a same-named receiver-own template.
                            substitution.extend(inherited);
                        }
                        let ty = ty.substitute_templates(&substitution);
                        self.record_ref(Arc::from(format!("prop:{}::{}", owner, prop_name)), span);
                        *declaring_class = Some(owner);
                        return ty;
                    }
                    let get_method = crate::db::find_method_in_chain(
                        self.db,
                        crate::db::Fqcn::from_str(self.db, fqcn.as_ref()),
                        "__get",
                    );
                    // `stdClass` permits arbitrary dynamic properties at
                    // runtime (json_decode results, DB rows, casts), so any
                    // `->prop` access is valid — never an UndefinedProperty.
                    let allows_dynamic = fqcn.as_ref().eq_ignore_ascii_case("stdClass");
                    if get_method.is_none()
                        && !allows_dynamic
                        && !crate::db::has_unknown_ancestor(self.db, fqcn.as_ref())
                    {
                        // Give a plugin (e.g. Eloquent `$casts`) a chance to
                        // supply the type before flagging the property undefined.
                        if let Some(ty) = self.class_property_from_plugin(fqcn.as_ref(), prop_name)
                        {
                            self.record_ref(
                                Arc::from(format!("prop:{}::{}", fqcn, prop_name)),
                                span,
                            );
                            *declaring_class = Some((*fqcn).into());
                            return ty;
                        }
                        if !self.in_existence_check {
                            self.emit(
                                IssueKind::UndefinedProperty {
                                    class: fqcn.to_string(),
                                    property: prop_name.to_string(),
                                },
                                Severity::Warning,
                                span,
                            );
                        }
                    }
                    return get_method
                        .and_then(|(_, m)| m.effective_return_type().cloned())
                        .unwrap_or_else(Type::mixed);
                }
                Atomic::TNamedObject { fqcn, .. }
                    if crate::db::class_kind(self.db, fqcn.as_ref())
                        .is_some_and(|k| k.is_trait) =>
                {
                    // Inside a trait body $this is typed as the trait itself.
                    // Properties can be declared on the trait or inherited from
                    // trait ancestors. If not found, return mixed silently —
                    // the using class might supply the property.
                    let prop_result = crate::db::find_property_in_chain(
                        self.db,
                        crate::db::Fqcn::new(self.db, *fqcn),
                        prop_name,
                    );
                    if let Some((owner, p)) = prop_result {
                        let ty = p.ty.as_deref().cloned().unwrap_or_else(Type::mixed);
                        self.record_ref(Arc::from(format!("prop:{}::{}", owner, prop_name)), span);
                        *declaring_class = Some(owner);
                        return ty;
                    }
                    // The property may be supplied by whichever class ends up
                    // consuming this trait — record a per-trait marker so
                    // DeadCodeAnalyzer can credit any composing class's own
                    // private property of this name as used.
                    self.record_ref(Arc::from(format!("traituse:{fqcn}::{prop_name}")), span);
                    return Type::mixed();
                }
                Atomic::TNamedObject { fqcn, .. }
                    if crate::db::class_kind(self.db, fqcn.as_ref())
                        .is_some_and(|k| k.is_interface) =>
                {
                    if let Some(crate::db::ClassLike::Interface(iface)) = crate::db::find_class_like(
                        self.db,
                        crate::db::Fqcn::from_str(self.db, fqcn.as_ref()),
                    ) {
                        if iface.seal_properties
                            && !self.in_existence_check
                            && !iface.own_properties.contains_key(prop_name)
                        {
                            self.emit(
                                IssueKind::NoInterfaceProperties {
                                    property: prop_name.to_string(),
                                },
                                Severity::Info,
                                span,
                            );
                        }
                        if let Some(p) = iface.own_properties.get(prop_name) {
                            // Unlike the class/trait branches above, this never ran
                            // record_ref/set declaring_class — a `$x->prop` access
                            // through an interface-typed `$x` was invisible to
                            // find-references/hover and any dead-code exemption
                            // that keys off the property reference.
                            self.record_ref(
                                Arc::from(format!("prop:{}::{}", fqcn, prop_name)),
                                span,
                            );
                            *declaring_class = Some((*fqcn).into());
                            return p.ty.as_deref().cloned().unwrap_or_else(Type::mixed);
                        }
                    }
                    return Type::mixed();
                }
                Atomic::TNamedObject { fqcn, .. }
                    if crate::db::class_kind(self.db, fqcn.as_ref()).is_some_and(|k| k.is_enum) =>
                {
                    match prop_name {
                        "name" => return Type::single(Atomic::TNonEmptyString),
                        "value" => {
                            let here = crate::db::Fqcn::new(self.db, *fqcn);
                            if let Some(scalar_ty) = crate::db::find_class_like(self.db, here)
                                .and_then(|c| c.enum_scalar_type().cloned())
                            {
                                return scalar_ty;
                            }
                            if !self.in_existence_check {
                                self.emit(
                                    IssueKind::UndefinedProperty {
                                        class: fqcn.to_string(),
                                        property: prop_name.to_string(),
                                    },
                                    Severity::Warning,
                                    span,
                                );
                            }
                            return Type::mixed();
                        }
                        _ => {
                            let here = crate::db::Fqcn::new(self.db, *fqcn);
                            if let Some(crate::db::ClassLike::Enum(e)) =
                                crate::db::find_class_like(self.db, here)
                            {
                                if let Some(p) = e.own_properties.get(prop_name) {
                                    self.record_ref(
                                        Arc::from(format!("prop:{}::{}", fqcn, prop_name)),
                                        span,
                                    );
                                    *declaring_class = Some((*fqcn).into());
                                    return p.ty.as_deref().cloned().unwrap_or_else(Type::mixed);
                                }
                            }
                            if !self.in_existence_check {
                                self.emit(
                                    IssueKind::UndefinedProperty {
                                        class: fqcn.to_string(),
                                        property: prop_name.to_string(),
                                    },
                                    Severity::Warning,
                                    span,
                                );
                            }
                            return Type::mixed();
                        }
                    }
                }
                Atomic::TSelf { fqcn }
                | Atomic::TStaticObject { fqcn }
                | Atomic::TParent { fqcn } => {
                    // A variable/param holding a self/static/parent-typed value
                    // (not just `$this`, which is injected as a plain
                    // TNamedObject) — resolve the property the same way, minus
                    // template substitution (these atoms carry no type params
                    // of their own). Falls through to the loop's final
                    // `Type::mixed()` on a miss, matching prior silent behavior.
                    let prop_result = crate::db::find_property_in_chain(
                        self.db,
                        crate::db::Fqcn::new(self.db, *fqcn),
                        prop_name,
                    );
                    if let Some((owner, p)) = prop_result {
                        if let Some(msg) = &p.deprecated {
                            self.emit(
                                IssueKind::DeprecatedProperty {
                                    class: fqcn.to_string(),
                                    property: prop_name.to_string(),
                                    message: Some(msg.clone()).filter(|m| !m.is_empty()),
                                },
                                Severity::Info,
                                span,
                            );
                        }
                        let ty = p.ty.as_deref().cloned().unwrap_or_else(Type::mixed);
                        self.record_ref(Arc::from(format!("prop:{}::{}", owner, prop_name)), span);
                        *declaring_class = Some(owner);
                        return ty;
                    }
                }
                Atomic::TIntersection { parts } => {
                    for part in parts.iter() {
                        for inner_atomic in &part.types {
                            if let Atomic::TNamedObject { fqcn, .. } = inner_atomic {
                                let prop_result = crate::db::find_property_in_chain(
                                    self.db,
                                    crate::db::Fqcn::new(self.db, *fqcn),
                                    prop_name,
                                );
                                if let Some((owner, p)) = prop_result {
                                    self.record_ref(
                                        Arc::from(format!("prop:{}::{}", owner, prop_name)),
                                        span,
                                    );
                                    *declaring_class = Some(owner);
                                    return p.ty.as_deref().cloned().unwrap_or_else(Type::mixed);
                                }
                            }
                        }
                    }
                    if !self.in_existence_check {
                        self.emit(
                            IssueKind::UndefinedProperty {
                                class: atomic.to_string(),
                                property: prop_name.to_string(),
                            },
                            Severity::Warning,
                            span,
                        );
                    }
                    return Type::mixed();
                }
                Atomic::TMixed => return Type::mixed(),
                _ => {}
            }
        }
        Type::mixed()
    }
}
