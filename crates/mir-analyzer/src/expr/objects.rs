use super::helpers::extract_string_from_expr;
use super::ExpressionAnalyzer;
use crate::context::Context;
use crate::symbol::SymbolKind;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Union};
use php_ast::owned::{Expr, ExprKind, NewExpr, PropertyAccessExpr, StaticAccessExpr};
use std::sync::Arc;

fn is_valid_class_name_type(ty: &Union) -> bool {
    // Class names must be strings or class-string types
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
    pub(super) fn analyze_new(
        &mut self,
        n: &NewExpr,
        call_span: php_ast::Span,
        ctx: &mut Context,
    ) -> Union {
        let arg_types: Vec<Union> = n
            .args
            .iter()
            .map(|a| {
                let ty = self.analyze(&a.value, ctx);
                if a.unpack {
                    crate::call::spread_element_type(&ty)
                } else {
                    ty
                }
            })
            .collect();
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

        let class_ty = match &n.class.kind {
            ExprKind::Identifier(name) => {
                let resolved = crate::db::resolve_name_via_db(self.db, &self.file, name.as_ref());
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
                let type_exists = crate::db::type_exists_via_db(self.db, fqcn.as_ref());
                if !matches!(resolved.as_str(), "self" | "static" | "parent") && !type_exists {
                    self.emit(
                        IssueKind::UndefinedClass {
                            name: resolved.clone(),
                        },
                        Severity::Error,
                        n.class.span,
                    );
                } else if type_exists {
                    let here = crate::db::Fqcn::new(self.db, fqcn.clone());
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
                    }
                    let fqcn_arc: Arc<str> = Arc::from(fqcn.as_ref());
                    let ctor_params = crate::db::find_method_in_chain(
                        self.db,
                        crate::db::Fqcn::new(self.db, fqcn_arc),
                        "__construct",
                    )
                    .map(|(_, s)| s.params.to_vec());
                    if let Some(ctor_params) = ctor_params {
                        crate::call::check_constructor_args(
                            self,
                            &fqcn,
                            crate::call::CheckArgsParams {
                                fn_name: "__construct",
                                params: &ctor_params,
                                arg_types: &arg_types,
                                arg_spans: &arg_spans,
                                arg_names: &arg_names,
                                arg_can_be_byref: &arg_can_be_byref,
                                call_span,
                                has_spread: n.args.iter().any(|a| a.unpack),
                            },
                        );
                    }
                }
                let ty = Union::single(Atomic::TNamedObject {
                    fqcn: mir_types::Symbol::from(fqcn.as_ref()),
                    type_params: vec![],
                });
                self.record_symbol(
                    n.class.span,
                    SymbolKind::ClassReference(fqcn.clone()),
                    ty.clone(),
                );
                if !self.inference_only {
                    let (line, col_start, col_end) = self.span_to_ref_loc(n.class.span);
                    self.db.record_reference_location(crate::db::RefLoc {
                        symbol_key: fqcn.clone(),
                        file: self.file.clone(),
                        line,
                        col_start,
                        col_end,
                    });
                }
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
                        IssueKind::UndefinedClass {
                            name: "<dynamic>".to_string(),
                        },
                        Severity::Error,
                        n.class.span,
                    );
                }
                // Check abstract for known TClassString
                for atom in ty.types.iter() {
                    if let Atomic::TClassString(Some(fqcn)) = atom {
                        let here = crate::db::Fqcn::new(self.db, Arc::from(fqcn.as_ref()));
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
                Union::single(Atomic::TObject)
            }
        };
        class_ty
    }

    pub(super) fn analyze_property_access(
        &mut self,
        pa: &PropertyAccessExpr,
        expr_span: php_ast::Span,
        ctx: &mut Context,
    ) -> Union {
        let obj_ty = self.analyze(&pa.object, ctx);
        let prop_name =
            extract_string_from_expr(&pa.property).unwrap_or_else(|| "<dynamic>".to_string());

        if obj_ty.contains(|t| matches!(t, Atomic::TNull)) && obj_ty.is_single() {
            self.emit(
                IssueKind::NullPropertyFetch {
                    property: prop_name.clone(),
                },
                Severity::Error,
                expr_span,
            );
            return Union::mixed();
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
            return Union::mixed();
        }
        let resolved = self.resolve_property_type(&obj_ty, &prop_name, pa.property.span);
        for atomic in &obj_ty.types {
            if let Atomic::TNamedObject { fqcn, .. } = atomic {
                self.record_symbol(
                    pa.property.span,
                    SymbolKind::PropertyAccess {
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
        ctx: &mut Context,
    ) -> Union {
        let obj_ty = self.analyze(&pa.object, ctx);
        let prop_name =
            extract_string_from_expr(&pa.property).unwrap_or_else(|| "<dynamic>".to_string());
        if prop_name == "<dynamic>" {
            return Union::mixed();
        }
        let non_null_ty = obj_ty.remove_null();
        let mut prop_ty = self.resolve_property_type(&non_null_ty, &prop_name, pa.property.span);
        prop_ty.add_type(Atomic::TNull);
        for atomic in &non_null_ty.types {
            if let Atomic::TNamedObject { fqcn, .. } = atomic {
                self.record_symbol(
                    pa.property.span,
                    SymbolKind::PropertyAccess {
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

    pub(super) fn analyze_static_property_access(&mut self, spa: &StaticAccessExpr) -> Union {
        if let ExprKind::Identifier(id) = &spa.class.kind {
            let resolved = crate::db::resolve_name_via_db(self.db, &self.file, id.as_ref());
            if !matches!(resolved.as_str(), "self" | "static" | "parent")
                && !crate::db::type_exists_via_db(self.db, &resolved)
            {
                self.emit(
                    IssueKind::UndefinedClass { name: resolved },
                    Severity::Error,
                    spa.class.span,
                );
            }
        }
        Union::mixed()
    }

    pub(super) fn analyze_class_const_access(
        &mut self,
        cca: &StaticAccessExpr,
        expr_span: php_ast::Span,
        ctx: &Context,
    ) -> Union {
        if expr_name_str(&cca.member) == Some("class") {
            let fqcn = if let ExprKind::Identifier(id) = &cca.class.kind {
                let resolved = crate::db::resolve_name_via_db(self.db, &self.file, id.as_ref());
                if !matches!(resolved.as_str(), "self" | "static" | "parent") {
                    if !crate::db::type_exists_via_db(self.db, &resolved) {
                        self.emit(
                            IssueKind::UndefinedClass {
                                name: resolved.clone(),
                            },
                            Severity::Error,
                            cca.class.span,
                        );
                    }
                    if !self.inference_only {
                        let (line, col_start, col_end) = self.span_to_ref_loc(cca.class.span);
                        self.db.record_reference_location(crate::db::RefLoc {
                            symbol_key: Arc::from(resolved.as_str()),
                            file: self.file.clone(),
                            line,
                            col_start,
                            col_end,
                        });
                    }
                }
                Some(mir_types::Symbol::from(resolved.as_str()))
            } else {
                None
            };
            return Union::single(Atomic::TClassString(fqcn));
        }

        let const_name = match expr_name_str(&cca.member) {
            Some(n) => n.to_string(),
            None => return Union::mixed(),
        };

        let fqcn = match &cca.class.kind {
            ExprKind::Identifier(id) => {
                let resolved = crate::db::resolve_name_via_db(self.db, &self.file, id.as_ref());
                match resolved.as_str() {
                    "self" | "static" => {
                        let Some(self_fqcn) = &ctx.self_fqcn else {
                            return Union::mixed();
                        };
                        let exists = crate::db::class_constant_exists_in_chain(
                            self.db,
                            self_fqcn,
                            &const_name,
                        );
                        if !exists && !crate::db::has_unknown_ancestor_via_db(self.db, self_fqcn) {
                            self.emit(
                                IssueKind::UndefinedConstant {
                                    name: format!("{self_fqcn}::{const_name}"),
                                },
                                Severity::Error,
                                expr_span,
                            );
                        }
                        return Union::mixed();
                    }
                    "parent" => {
                        let Some(parent_fqcn) = &ctx.parent_fqcn else {
                            return Union::mixed();
                        };
                        let exists = crate::db::class_constant_exists_in_chain(
                            self.db,
                            parent_fqcn,
                            &const_name,
                        );
                        if !exists && !crate::db::has_unknown_ancestor_via_db(self.db, parent_fqcn)
                        {
                            self.emit(
                                IssueKind::UndefinedConstant {
                                    name: format!("{parent_fqcn}::{const_name}"),
                                },
                                Severity::Error,
                                expr_span,
                            );
                        }
                        return Union::mixed();
                    }
                    _ => resolved,
                }
            }
            _ => return Union::mixed(),
        };

        if !crate::db::type_exists_via_db(self.db, &fqcn) {
            self.emit(
                IssueKind::UndefinedClass { name: fqcn },
                Severity::Error,
                cca.class.span,
            );
            return Union::mixed();
        }

        if !self.inference_only {
            let (line, col_start, col_end) = self.span_to_ref_loc(cca.class.span);
            self.db.record_reference_location(crate::db::RefLoc {
                symbol_key: Arc::from(fqcn.as_str()),
                file: self.file.clone(),
                line,
                col_start,
                col_end,
            });
        }

        let const_exists = crate::db::class_constant_exists_in_chain(self.db, &fqcn, &const_name);
        if !const_exists && !crate::db::has_unknown_ancestor_via_db(self.db, &fqcn) {
            self.emit(
                IssueKind::UndefinedConstant {
                    name: format!("{fqcn}::{const_name}"),
                },
                Severity::Error,
                expr_span,
            );
        }
        Union::mixed()
    }

    pub(super) fn resolve_property_type(
        &mut self,
        obj_ty: &Union,
        prop_name: &str,
        span: php_ast::Span,
    ) -> Union {
        for atomic in &obj_ty.types {
            match atomic {
                Atomic::TNamedObject { fqcn, .. }
                    if crate::db::class_kind_via_db(self.db, fqcn.as_ref())
                        .is_some_and(|k| !k.is_interface && !k.is_trait && !k.is_enum) =>
                {
                    let fqcn_arc: Arc<str> = Arc::from(fqcn.as_ref());
                    let prop_found: Option<Union> = crate::db::find_property_in_chain(
                        self.db,
                        crate::db::Fqcn::new(self.db, fqcn_arc),
                        prop_name,
                    )
                    .map(|(_, p)| p.ty.unwrap_or_else(Union::mixed));
                    if let Some(ty) = prop_found {
                        if !self.inference_only {
                            let (line, col_start, col_end) = self.span_to_ref_loc(span);
                            self.db.record_reference_location(crate::db::RefLoc {
                                symbol_key: Arc::from(format!("{}::{}", fqcn, prop_name)),
                                file: self.file.clone(),
                                line,
                                col_start,
                                col_end,
                            });
                        }
                        return ty;
                    }
                    if !crate::db::has_unknown_ancestor_via_db(self.db, fqcn.as_ref())
                        && !crate::db::has_method_in_chain(self.db, fqcn.as_ref(), "__get")
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
                    return Union::mixed();
                }
                Atomic::TNamedObject { fqcn, .. }
                    if crate::db::class_kind_via_db(self.db, fqcn.as_ref())
                        .is_some_and(|k| k.is_enum) =>
                {
                    match prop_name {
                        "name" => return Union::single(Atomic::TNonEmptyString),
                        "value" => {
                            let here = crate::db::Fqcn::new(self.db, Arc::from(fqcn.as_ref()));
                            if let Some(scalar_ty) = crate::db::find_class_like(self.db, here)
                                .and_then(|c| c.enum_scalar_type().cloned())
                            {
                                return scalar_ty;
                            }
                            self.emit(
                                IssueKind::UndefinedProperty {
                                    class: fqcn.to_string(),
                                    property: prop_name.to_string(),
                                },
                                Severity::Warning,
                                span,
                            );
                            return Union::mixed();
                        }
                        _ => {
                            self.emit(
                                IssueKind::UndefinedProperty {
                                    class: fqcn.to_string(),
                                    property: prop_name.to_string(),
                                },
                                Severity::Warning,
                                span,
                            );
                            return Union::mixed();
                        }
                    }
                }
                Atomic::TMixed => return Union::mixed(),
                _ => {}
            }
        }
        Union::mixed()
    }
}
