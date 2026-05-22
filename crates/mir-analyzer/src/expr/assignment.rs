use super::helpers::{
    extract_simple_var, extract_string_from_expr, infer_arithmetic, property_assign_compatible,
    widen_array_with_value,
};
use super::ExpressionAnalyzer;
use crate::context::Context;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Union};
use php_ast::ast::AssignOp;
use php_ast::owned::{AssignExpr, Expr, ExprKind};
use php_ast::Span;

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_assign(
        &mut self,
        a: &AssignExpr,
        expr_span: Span,
        ctx: &mut Context,
    ) -> Union {
        let rhs_tainted = crate::taint::is_expr_tainted(&a.value, ctx);
        let rhs_ty = self.analyze(&a.value, ctx);
        if rhs_ty.is_never() {
            return rhs_ty;
        }
        match a.op {
            AssignOp::Assign => {
                self.assign_to_target(&a.target, rhs_ty.clone(), ctx, expr_span);
                if rhs_tainted {
                    if let ExprKind::Variable(name) = &a.target.kind {
                        ctx.taint_var(name.as_ref());
                    }
                }
                rhs_ty
            }
            AssignOp::Concat => {
                if let Some(var_name) = extract_simple_var(&a.target) {
                    ctx.set_var(&var_name, Union::single(Atomic::TString));
                    let (line, col_start) = self.offset_to_line_col(a.target.span.start);
                    let (line_end, col_end) = self.offset_to_line_col(a.target.span.end);
                    ctx.record_var_location(&var_name, line, col_start, line_end, col_end);
                }
                Union::single(Atomic::TString)
            }
            AssignOp::Plus
            | AssignOp::Minus
            | AssignOp::Mul
            | AssignOp::Div
            | AssignOp::Mod
            | AssignOp::Pow => {
                let lhs_ty = self.analyze(&a.target, ctx);
                let result_ty = infer_arithmetic(&lhs_ty, &rhs_ty);
                if let Some(var_name) = extract_simple_var(&a.target) {
                    ctx.set_var(&var_name, result_ty.clone());
                    let (line, col_start) = self.offset_to_line_col(a.target.span.start);
                    let (line_end, col_end) = self.offset_to_line_col(a.target.span.end);
                    ctx.record_var_location(&var_name, line, col_start, line_end, col_end);
                }
                result_ty
            }
            AssignOp::Coalesce => {
                let lhs_ty = self.analyze(&a.target, ctx);
                let merged = Union::merge(&lhs_ty.remove_null(), &rhs_ty);
                if let Some(var_name) = extract_simple_var(&a.target) {
                    ctx.set_var(&var_name, merged.clone());
                    let (line, col_start) = self.offset_to_line_col(a.target.span.start);
                    let (line_end, col_end) = self.offset_to_line_col(a.target.span.end);
                    ctx.record_var_location(&var_name, line, col_start, line_end, col_end);
                }
                merged
            }
            _ => {
                if let Some(var_name) = extract_simple_var(&a.target) {
                    ctx.set_var(&var_name, Union::mixed());
                    let (line, col_start) = self.offset_to_line_col(a.target.span.start);
                    let (line_end, col_end) = self.offset_to_line_col(a.target.span.end);
                    ctx.record_var_location(&var_name, line, col_start, line_end, col_end);
                }
                Union::mixed()
            }
        }
    }

    pub(super) fn assign_to_target(
        &mut self,
        target: &Expr,
        ty: Union,
        ctx: &mut Context,
        span: Span,
    ) {
        match &target.kind {
            ExprKind::Variable(name) => {
                let name_str = name.trim_start_matches('$').to_string();
                let name_sym = mir_types::Symbol::from(name_str.as_str());
                if ctx.byref_param_names.contains(&name_sym) {
                    ctx.read_vars.insert(name_sym);
                }
                ctx.set_var(name_str.clone(), ty);
                let (line, col_start) = self.offset_to_line_col(target.span.start);
                let (line_end, col_end) = self.offset_to_line_col(target.span.end);
                ctx.record_var_location(&name_str, line, col_start, line_end, col_end);
            }
            ExprKind::Array(elements) => {
                let has_non_array = ty.contains(|a| matches!(a, Atomic::TFalse | Atomic::TNull));
                let has_array = ty.contains(|a| {
                    matches!(
                        a,
                        Atomic::TArray { .. }
                            | Atomic::TList { .. }
                            | Atomic::TNonEmptyArray { .. }
                            | Atomic::TNonEmptyList { .. }
                            | Atomic::TKeyedArray { .. }
                    )
                });
                if has_non_array && has_array {
                    self.emit(
                        IssueKind::PossiblyInvalidArrayOffset {
                            expected: "array".to_string(),
                            actual: format!("{ty}"),
                        },
                        Severity::Warning,
                        span,
                    );
                }
                let value_ty: Union = ty
                    .types
                    .iter()
                    .find_map(|a| match a {
                        Atomic::TArray { value, .. }
                        | Atomic::TList { value }
                        | Atomic::TNonEmptyArray { value, .. }
                        | Atomic::TNonEmptyList { value } => Some(*value.clone()),
                        _ => None,
                    })
                    .unwrap_or_else(Union::mixed);
                for elem in elements.iter() {
                    self.assign_to_target(&elem.value, value_ty.clone(), ctx, span);
                }
            }
            ExprKind::PropertyAccess(pa) => {
                let obj_ty = self.analyze(&pa.object, ctx);
                if let Some(prop_name) = extract_string_from_expr(&pa.property) {
                    for atomic in &obj_ty.types {
                        if let Atomic::TNamedObject { fqcn, .. } = atomic {
                            let db = self.db;
                            let here = crate::db::Fqcn::new(db, *fqcn);
                            let prop_info: Option<(bool, Option<Union>)> =
                                crate::db::find_property_in_class(db, here, &prop_name)
                                    .map(|p| (p.is_readonly, p.ty.clone()));
                            if let Some((is_readonly, prop_ty)) = prop_info {
                                if is_readonly && !ctx.inside_constructor {
                                    self.emit(
                                        IssueKind::ReadonlyPropertyAssignment {
                                            class: fqcn.to_string(),
                                            property: prop_name.clone(),
                                        },
                                        Severity::Error,
                                        span,
                                    );
                                }
                                if let Some(prop_ty) = &prop_ty {
                                    if !prop_ty.is_mixed()
                                        && !ty.is_mixed()
                                        && !property_assign_compatible(&ty, prop_ty, self.db)
                                    {
                                        self.emit(
                                            IssueKind::InvalidPropertyAssignment {
                                                property: prop_name.clone(),
                                                expected: format!("{prop_ty}"),
                                                actual: format!("{ty}"),
                                            },
                                            Severity::Warning,
                                            span,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            ExprKind::StaticPropertyAccess(_) => {}
            ExprKind::ArrayAccess(aa) => {
                if let Some(idx) = &aa.index {
                    self.analyze(idx, ctx);
                }
                let mut base: &Expr = &aa.array;
                loop {
                    match &base.kind {
                        ExprKind::Variable(name) => {
                            let name_str = name.trim_start_matches('$');
                            if !ctx.var_is_defined(name_str) {
                                let name_sym = mir_types::Symbol::from(name_str);
                                ctx.vars.insert(
                                    name_sym,
                                    Union::single(Atomic::TArray {
                                        key: Box::new(Union::mixed()),
                                        value: Box::new(ty.clone()),
                                    }),
                                );
                                ctx.assigned_vars.insert(name_sym);
                                let (line, col_start) = self.offset_to_line_col(base.span.start);
                                let (line_end, col_end) = self.offset_to_line_col(base.span.end);
                                ctx.record_var_location(
                                    name_str, line, col_start, line_end, col_end,
                                );
                            } else {
                                let current = ctx.get_var(name_str);
                                let updated = widen_array_with_value(&current, &ty);
                                ctx.set_var(name_str, updated);
                            }
                            break;
                        }
                        ExprKind::ArrayAccess(inner) => {
                            if let Some(idx) = &inner.index {
                                self.analyze(idx, ctx);
                            }
                            base = &inner.array;
                        }
                        _ => break,
                    }
                }
            }
            ExprKind::VariableVariable(inner) => {
                if let Some(var_name) = extract_simple_var(inner) {
                    ctx.read_vars
                        .insert(mir_types::Symbol::from(var_name.as_str()));
                    let var_ty = ctx.get_var(&var_name);
                    for atomic in &var_ty.types {
                        if let Atomic::TLiteralString(accessed_var_name) = atomic {
                            ctx.set_var(accessed_var_name.to_string(), ty.clone());
                            let (line, col_start) = self.offset_to_line_col(target.span.start);
                            let (line_end, col_end) = self.offset_to_line_col(target.span.end);
                            ctx.record_var_location(
                                accessed_var_name,
                                line,
                                col_start,
                                line_end,
                                col_end,
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
