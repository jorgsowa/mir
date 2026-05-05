use super::helpers::infer_arithmetic;
use super::ExpressionAnalyzer;
use crate::context::Context;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Union};
use php_ast::ast::{BinaryExpr, BinaryOp, ExprKind};
use php_ast::Span;
use std::sync::Arc;

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_binary_expr<'arena, 'src>(
        &mut self,
        b: &BinaryExpr<'arena, 'src>,
        _span: Span,
        ctx: &mut Context,
    ) -> Union {
        use BinaryOp as B;
        if matches!(
            b.op,
            B::BooleanAnd | B::LogicalAnd | B::BooleanOr | B::LogicalOr
        ) {
            let _left_ty = self.analyze(b.left, ctx);
            let mut right_ctx = ctx.fork();
            let is_and = matches!(b.op, B::BooleanAnd | B::LogicalAnd);
            crate::narrowing::narrow_from_condition(
                b.left,
                &mut right_ctx,
                is_and,
                self.db,
                &self.file,
            );
            if !right_ctx.diverges {
                let _right_ty = self.analyze(b.right, &mut right_ctx);
            }
            for v in right_ctx.read_vars {
                ctx.read_vars.insert(v.clone());
            }
            for (name, ty) in &right_ctx.vars {
                if !ctx.vars.contains_key(name.as_str()) {
                    ctx.vars.insert(name.clone(), ty.clone());
                    ctx.possibly_assigned_vars.insert(name.clone());
                }
            }
            return Union::single(Atomic::TBool);
        }

        if b.op == B::Instanceof {
            let _left_ty = self.analyze(b.left, ctx);
            if let ExprKind::Identifier(name) = &b.right.kind {
                let resolved = crate::db::resolve_name_via_db(self.db, &self.file, name.as_ref());
                let fqcn: Arc<str> = Arc::from(resolved.as_str());
                if !matches!(resolved.as_str(), "self" | "static" | "parent")
                    && !crate::db::type_exists_via_db(self.db, &fqcn)
                {
                    self.emit(
                        IssueKind::UndefinedClass { name: resolved },
                        Severity::Error,
                        b.right.span,
                    );
                }
            }
            return Union::single(Atomic::TBool);
        }

        let left_ty = self.analyze(b.left, ctx);
        let right_ty = self.analyze(b.right, ctx);

        match b.op {
            BinaryOp::Add
            | BinaryOp::Sub
            | BinaryOp::Mul
            | BinaryOp::Div
            | BinaryOp::Mod
            | BinaryOp::Pow => infer_arithmetic(&left_ty, &right_ty),

            BinaryOp::Concat => Union::single(Atomic::TString),

            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Identical
            | BinaryOp::NotIdentical
            | BinaryOp::Less
            | BinaryOp::Greater
            | BinaryOp::LessOrEqual
            | BinaryOp::GreaterOrEqual => Union::single(Atomic::TBool),

            BinaryOp::Spaceship => Union::single(Atomic::TIntRange {
                min: Some(-1),
                max: Some(1),
            }),

            BinaryOp::BooleanAnd
            | BinaryOp::BooleanOr
            | BinaryOp::LogicalAnd
            | BinaryOp::LogicalOr
            | BinaryOp::LogicalXor => Union::single(Atomic::TBool),

            BinaryOp::BitwiseAnd
            | BinaryOp::BitwiseOr
            | BinaryOp::BitwiseXor
            | BinaryOp::ShiftLeft
            | BinaryOp::ShiftRight => Union::single(Atomic::TInt),

            BinaryOp::Pipe => right_ty,
            BinaryOp::Instanceof => Union::single(Atomic::TBool),
        }
    }
}
