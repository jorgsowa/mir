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
    // Class names must be strings or class-string types only.
    // Mixed is not allowed - must be explicit string or class-string.
    ty.contains(|t| {
        matches!(
            t,
            Atomic::TString | Atomic::TClassString(_) | Atomic::TLiteralString(_)
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
        &self,
        fqcn: &Arc<str>,
        ctor_params: Option<&[mir_codebase::storage::FnParam]>,
        arg_types: &[Type],
    ) -> Arc<[Type]> {
        let empty = mir_types::union::empty_type_params();
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
        let bindings =
            crate::generic::infer_arg_template_bindings(&class_tps, ctor_params, arg_types);

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
                Some(ty) if !ty.is_mixed() => {
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
        for a in n.args.iter() {
            let ty = self.analyze(&a.value, ctx);
            arg_types.push(if a.unpack {
                crate::call::spread_element_type(&ty)
            } else {
                ty
            });
        }
        let arg_spans: Vec<php_ast::Span> = n.args.iter().map(|a| a.span).collect();
        let arg_names: Vec<Option<String>> = n
            .args
            .iter()
            .map(|a| a.name.as_ref().map(crate::parser::name_to_string_owned))
            .collect();
        let arg_can_be_byref: Vec<bool> = n
            .args
            .iter()
            .map(|a| expr_can_be_passed_by_reference_owned(&a.value))
            .collect();

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
                    && !ctx.class_exists_guards.contains(fqcn.as_ref())
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
                        if class.is_class() && class.is_abstract() {
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
                        let written_short = name.rsplit('\\').next().unwrap_or(name.as_ref());
                        let canonical_short = class
                            .fqcn()
                            .rsplit('\\')
                            .next()
                            .unwrap_or(class.fqcn().as_ref());
                        if written_short != canonical_short
                            && written_short.eq_ignore_ascii_case(canonical_short)
                        {
                            self.emit(
                                IssueKind::WrongCaseClass {
                                    used: written_short.to_string(),
                                    canonical: canonical_short.to_string(),
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
                    .map(|(_, s)| (s.params.to_vec(), s.template_params.clone()));
                    if let Some((ctor_params, ctor_templates)) = &ctor_params_and_templates {
                        crate::call::check_constructor_args(
                            self,
                            &fqcn,
                            crate::call::CheckArgsParams {
                                fn_name: "__construct",
                                params: ctor_params,
                                arg_types: &arg_types,
                                arg_spans: &arg_spans,
                                arg_names: &arg_names,
                                arg_can_be_byref: &arg_can_be_byref,
                                call_span,
                                has_spread: n.args.iter().any(|a| a.unpack),
                                template_params: ctor_templates,
                            },
                        );
                    } else if !arg_types.is_empty()
                        && !n.args.iter().any(|a| a.unpack)
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
                            .map(|(p, _)| p.as_slice()),
                        &arg_types,
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
                self.record_ref(fqcn.clone(), n.class.span);
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
                // Check abstract for known TClassString
                for atom in ty.types.iter() {
                    if let Atomic::TClassString(Some(fqcn)) = atom {
                        let here = crate::db::Fqcn::new(self.db, *fqcn);
                        let is_abstract_class = crate::db::find_class_like(self.db, here)
                            .map(|c| c.is_class() && c.is_abstract())
                            .unwrap_or(false);
                        if is_abstract_class {
                            self.emit(
                                IssueKind::AbstractInstantiation {
                                    class: fqcn.to_string(),
                                },
                                Severity::Error,
                                n.class.span,
                            );
                        }
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
        let prop_name =
            extract_string_from_expr(&pa.property).unwrap_or_else(|| "<dynamic>".to_string());

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
            return Type::mixed();
        }
        let resolved = self.resolve_property_type(&obj_ty, &prop_name, pa.property.span);
        for atomic in &obj_ty.types {
            if let Atomic::TNamedObject { fqcn, .. } = atomic {
                self.record_symbol(
                    pa.property.span,
                    ReferenceKind::PropertyAccess {
                        class: Arc::from(fqcn.as_ref()),
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
        let prop_name =
            extract_string_from_expr(&pa.property).unwrap_or_else(|| "<dynamic>".to_string());
        if prop_name == "<dynamic>" {
            self.analyze(&pa.property, ctx);
            return Type::mixed();
        }
        let non_null_ty = obj_ty.remove_null();
        let mut prop_ty = self.resolve_property_type(&non_null_ty, &prop_name, pa.property.span);
        prop_ty.add_type(Atomic::TNull);
        for atomic in &non_null_ty.types {
            if let Atomic::TNamedObject { fqcn, .. } = atomic {
                self.record_symbol(
                    pa.property.span,
                    ReferenceKind::PropertyAccess {
                        class: Arc::from(fqcn.as_ref()),
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
        if let ExprKind::Identifier(id) = &spa.class.kind {
            let resolved = crate::db::resolve_name(self.db, &self.file, id.as_ref());
            if matches!(resolved.as_str(), "self" | "static" | "parent") {
                // Cannot resolve without FlowState; skip reference recording.
            } else if !crate::db::class_exists(self.db, &resolved)
                && !ctx.class_exists_guards.contains(resolved.as_str())
            {
                self.emit(
                    IssueKind::UndefinedClass { name: resolved },
                    Severity::Error,
                    spa.class.span,
                );
            } else {
                self.record_ref(Arc::from(resolved.as_str()), spa.class.span);
                if let Some(prop_name) = expr_name_str(&spa.member) {
                    self.record_ref(
                        Arc::from(format!("{}::{}", resolved, prop_name)),
                        spa.member.span,
                    );
                    // Check if the static property is deprecated
                    let here = crate::db::Fqcn::from_str(self.db, resolved.as_str());
                    if let Some(p) = crate::db::find_property_in_chain(self.db, here, prop_name) {
                        if let Some(msg) = &p.1.deprecated {
                            self.emit(
                                IssueKind::DeprecatedProperty {
                                    class: resolved.clone(),
                                    property: prop_name.to_string(),
                                    message: Some(msg.clone()).filter(|m| !m.is_empty()),
                                },
                                Severity::Info,
                                spa.member.span,
                            );
                        }
                    }
                }
            }
        }
        Type::mixed()
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
                if !matches!(resolved.as_str(), "self" | "static" | "parent") {
                    if !crate::db::class_exists(self.db, &resolved)
                        && !self.in_class_exists_arg
                        && !ctx.class_exists_guards.contains(resolved.as_str())
                    {
                        self.emit(
                            IssueKind::UndefinedClass {
                                name: resolved.clone(),
                            },
                            Severity::Error,
                            cca.class.span,
                        );
                    } else {
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
                    self.record_ref(Arc::from(resolved.as_str()), cca.class.span);
                }
                return Type::single(Atomic::TClassString(Some(mir_types::Name::from(
                    resolved.as_str(),
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
                        let exists = crate::db::class_constant_exists_in_chain(
                            self.db,
                            self_fqcn,
                            &const_name,
                        );
                        if !exists && !crate::db::has_unknown_ancestor(self.db, self_fqcn) {
                            self.emit(
                                IssueKind::UndefinedConstant {
                                    name: format!("{self_fqcn}::{const_name}"),
                                },
                                Severity::Error,
                                expr_span,
                            );
                        }
                        self.record_ref(
                            Arc::from(format!("{}::{}", self_fqcn, const_name)),
                            cca.member.span,
                        );
                        self.record_symbol(
                            cca.member.span,
                            ReferenceKind::ConstantAccess {
                                class: self_fqcn.clone(),
                                constant: Arc::from(const_name.as_str()),
                            },
                            Type::mixed(),
                        );
                        return Type::mixed();
                    }
                    "parent" => {
                        let Some(parent_fqcn) = &ctx.parent_fqcn else {
                            return Type::mixed();
                        };
                        let exists = crate::db::class_constant_exists_in_chain(
                            self.db,
                            parent_fqcn,
                            &const_name,
                        );
                        if !exists && !crate::db::has_unknown_ancestor(self.db, parent_fqcn) {
                            self.emit(
                                IssueKind::UndefinedConstant {
                                    name: format!("{parent_fqcn}::{const_name}"),
                                },
                                Severity::Error,
                                expr_span,
                            );
                        }
                        self.record_ref(
                            Arc::from(format!("{}::{}", parent_fqcn, const_name)),
                            cca.member.span,
                        );
                        self.record_symbol(
                            cca.member.span,
                            ReferenceKind::ConstantAccess {
                                class: parent_fqcn.clone(),
                                constant: Arc::from(const_name.as_str()),
                            },
                            Type::mixed(),
                        );
                        return Type::mixed();
                    }
                    _ => resolved,
                }
            }
            _ => return Type::mixed(),
        };

        if !crate::db::class_exists(self.db, &fqcn)
            && !ctx.class_exists_guards.contains(fqcn.as_str())
        {
            self.emit(
                IssueKind::UndefinedClass { name: fqcn },
                Severity::Error,
                cca.class.span,
            );
            return Type::mixed();
        }

        self.record_ref(Arc::from(fqcn.as_str()), cca.class.span);
        self.record_ref(
            Arc::from(format!("{}::{}", fqcn, const_name)),
            cca.member.span,
        );

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
        // Check if the constant is deprecated
        if let Some((_, ref c)) = found {
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
        }
        let const_ty = found
            .as_ref()
            .map(|(_, c)| c.ty.clone())
            .unwrap_or_else(Type::mixed);

        self.record_symbol(
            cca.member.span,
            ReferenceKind::ConstantAccess {
                class: Arc::from(fqcn.as_str()),
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

    pub(super) fn resolve_property_type(
        &mut self,
        obj_ty: &Type,
        prop_name: &str,
        span: php_ast::Span,
    ) -> Type {
        for atomic in &obj_ty.types {
            match atomic {
                Atomic::TNamedObject { fqcn, .. }
                    if crate::db::class_kind(self.db, fqcn.as_ref())
                        .is_some_and(|k| !k.is_interface && !k.is_trait && !k.is_enum) =>
                {
                    let prop_result = crate::db::find_property_in_chain(
                        self.db,
                        crate::db::Fqcn::new(self.db, *fqcn),
                        prop_name,
                    );
                    if let Some((_, ref p)) = prop_result {
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
                    }
                    let prop_found: Option<Type> = prop_result
                        .map(|(_, p)| p.ty.as_deref().cloned().unwrap_or_else(Type::mixed));
                    if let Some(ty) = prop_found {
                        self.record_ref(Arc::from(format!("{}::{}", fqcn, prop_name)), span);
                        return ty;
                    }
                    if !crate::db::has_unknown_ancestor(self.db, fqcn.as_ref())
                        && !crate::db::has_method_in_chain(self.db, fqcn.as_ref(), "__get")
                        && !self.in_existence_check
                    {
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
                Atomic::TMixed => return Type::mixed(),
                _ => {}
            }
        }
        Type::mixed()
    }
}
