use super::helpers::extract_simple_var;
use super::ExpressionAnalyzer;
use crate::context::Context;
use mir_types::{Atomic, Union};
use php_ast::ast::{UnaryPostfixOp, UnaryPrefixOp};
use php_ast::owned::{UnaryPostfixExpr, UnaryPrefixExpr};

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_unary_prefix(&mut self, u: &UnaryPrefixExpr, ctx: &mut Context) -> Union {
        let operand_ty = self.analyze(&u.operand, ctx);
        match u.op {
            UnaryPrefixOp::BooleanNot => Union::single(Atomic::TBool),
            UnaryPrefixOp::Negate => {
                if operand_ty.contains(|t| t.is_int()) {
                    Union::single(Atomic::TInt)
                } else {
                    Union::single(Atomic::TFloat)
                }
            }
            UnaryPrefixOp::Plus => operand_ty,
            UnaryPrefixOp::BitwiseNot => Union::single(Atomic::TInt),
            UnaryPrefixOp::PreIncrement | UnaryPrefixOp::PreDecrement => {
                if let Some(var_name) = extract_simple_var(&u.operand) {
                    let ty = ctx.get_var(&var_name);
                    let new_ty = if ty
                        .contains(|t| matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..)))
                    {
                        Union::single(Atomic::TFloat)
                    } else {
                        Union::single(Atomic::TInt)
                    };
                    ctx.set_var(&var_name, new_ty.clone());
                    let (line, col_start) = self.offset_to_line_col(u.operand.span.start);
                    let (line_end, col_end) = self.offset_to_line_col(u.operand.span.end);
                    ctx.record_var_location(&var_name, line, col_start, line_end, col_end);
                    new_ty
                } else {
                    Union::single(Atomic::TInt)
                }
            }
        }
    }

    pub(super) fn analyze_unary_postfix(
        &mut self,
        u: &UnaryPostfixExpr,
        ctx: &mut Context,
    ) -> Union {
        let operand_ty = self.analyze(&u.operand, ctx);
        match u.op {
            UnaryPostfixOp::PostIncrement | UnaryPostfixOp::PostDecrement => {
                if let Some(var_name) = extract_simple_var(&u.operand) {
                    let new_ty = if operand_ty
                        .contains(|t| matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..)))
                    {
                        Union::single(Atomic::TFloat)
                    } else {
                        Union::single(Atomic::TInt)
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
