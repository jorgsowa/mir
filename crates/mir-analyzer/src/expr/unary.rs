use super::binary::operand_is_non_bitwise;
use super::helpers::extract_simple_var;
use super::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Type};
use php_ast::ast::{UnaryPostfixOp, UnaryPrefixOp};
use php_ast::owned::{UnaryPostfixExpr, UnaryPrefixExpr};

/// Returns true when every member of `ty` is definitively `true` or `bool`
/// (not `false` — that case is reserved for a future FalseOperand kind).
fn operand_is_definitely_bool(ty: &Type) -> bool {
    !ty.types.is_empty()
        && !ty.is_mixed()
        && ty
            .types
            .iter()
            .all(|a| matches!(a, Atomic::TTrue | Atomic::TBool))
}

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_unary_prefix(
        &mut self,
        u: &UnaryPrefixExpr,
        ctx: &mut FlowState,
    ) -> Type {
        let operand_ty = self.analyze(&u.operand, ctx);
        match u.op {
            UnaryPrefixOp::BooleanNot => Type::single(Atomic::TBool),
            UnaryPrefixOp::Negate => {
                if operand_ty.contains(|t| t.is_int()) {
                    Type::single(Atomic::TInt)
                } else {
                    Type::single(Atomic::TFloat)
                }
            }
            UnaryPrefixOp::Plus => operand_ty,
            UnaryPrefixOp::BitwiseNot => {
                if operand_is_non_bitwise(&operand_ty) {
                    self.emit(
                        IssueKind::InvalidOperand {
                            op: "~".to_string(),
                            left: operand_ty.to_string(),
                            right: String::new(),
                        },
                        Severity::Warning,
                        u.operand.span,
                    );
                }
                Type::single(Atomic::TInt)
            }
            UnaryPrefixOp::PreIncrement | UnaryPrefixOp::PreDecrement => {
                if let Some(var_name) = extract_simple_var(&u.operand) {
                    let ty = ctx.get_var(&var_name);
                    let new_ty = if ty
                        .contains(|t| matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..)))
                    {
                        Type::single(Atomic::TFloat)
                    } else {
                        Type::single(Atomic::TInt)
                    };
                    ctx.set_var(&var_name, new_ty.clone());
                    let (line, col_start) = self.offset_to_line_col(u.operand.span.start);
                    let (line_end, col_end) = self.offset_to_line_col(u.operand.span.end);
                    ctx.record_var_location(&var_name, line, col_start, line_end, col_end);
                    new_ty
                } else {
                    Type::single(Atomic::TInt)
                }
            }
        }
    }

    pub(super) fn analyze_unary_postfix(
        &mut self,
        u: &UnaryPostfixExpr,
        ctx: &mut FlowState,
    ) -> Type {
        let operand_ty = self.analyze(&u.operand, ctx);
        match u.op {
            UnaryPostfixOp::PostIncrement | UnaryPostfixOp::PostDecrement => {
                // Flag increment/decrement on a definitely-bool operand.
                // PHP silently no-ops on bool but this is almost always a bug.
                // Only fires when ALL union members are TTrue/TBool (never on
                // int|bool). TFalse is reserved for the future FalseOperand kind.
                if operand_is_definitely_bool(&operand_ty) {
                    let op = if u.op == UnaryPostfixOp::PostIncrement {
                        "++"
                    } else {
                        "--"
                    };
                    self.emit(
                        IssueKind::InvalidOperand {
                            op: op.to_string(),
                            left: operand_ty.to_string(),
                            right: String::new(),
                        },
                        Severity::Warning,
                        u.operand.span,
                    );
                }
                if let Some(var_name) = extract_simple_var(&u.operand) {
                    let new_ty = if operand_ty
                        .contains(|t| matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..)))
                    {
                        Type::single(Atomic::TFloat)
                    } else {
                        Type::single(Atomic::TInt)
                    };
                    ctx.set_var(&var_name, new_ty);
                    let (line, col_start) = self.offset_to_line_col(u.operand.span.start);
                    let (line_end, col_end) = self.offset_to_line_col(u.operand.span.end);
                    ctx.record_var_location(&var_name, line, col_start, line_end, col_end);
                }
                operand_ty
            }
        }
    }
}
