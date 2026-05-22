use super::helpers::infer_arithmetic;
use super::ExpressionAnalyzer;
use crate::context::Context;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Union};
use php_ast::ast::BinaryOp;
use php_ast::owned::{BinaryExpr, ExprKind};
use php_ast::Span;
use std::sync::Arc;

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_binary_expr(
        &mut self,
        b: &BinaryExpr,
        _span: Span,
        ctx: &mut Context,
    ) -> Union {
        use BinaryOp as B;
        if matches!(
            b.op,
            B::BooleanAnd | B::LogicalAnd | B::BooleanOr | B::LogicalOr
        ) {
            let _left_ty = self.analyze(&b.left, ctx);
            let mut right_ctx = ctx.fork();
            let is_and = matches!(b.op, B::BooleanAnd | B::LogicalAnd);
            crate::narrowing::narrow_from_condition(
                &b.left,
                &mut right_ctx,
                is_and,
                self.db,
                &self.file,
            );
            if !right_ctx.diverges {
                let _right_ty = self.analyze(&b.right, &mut right_ctx);
            }
            for v in right_ctx.read_vars {
                ctx.read_vars.insert(v);
            }
            for (name, ty) in &right_ctx.vars {
                if !ctx.vars.contains_key(name) {
                    ctx.vars.insert(*name, ty.clone());
                    ctx.possibly_assigned_vars.insert(*name);
                }
            }
            return Union::single(Atomic::TBool);
        }

        if b.op == B::Instanceof {
            let _left_ty = self.analyze(&b.left, ctx);
            if let ExprKind::Identifier(name) = &b.right.kind {
                let resolved = crate::db::resolve_name_via_db(self.db, &self.file, name.as_ref());
                let fqcn: Arc<str> = Arc::from(resolved.as_str());
                if !matches!(resolved.as_str(), "self" | "static" | "parent") {
                    if !crate::db::type_exists_via_db(self.db, &fqcn) {
                        self.emit(
                            IssueKind::UndefinedClass { name: resolved },
                            Severity::Error,
                            b.right.span,
                        );
                    }
                    if !self.inference_only {
                        let (line, col_start, col_end) = self.span_to_ref_loc(b.right.span);
                        self.db.record_reference_location(crate::db::RefLoc {
                            symbol_key: fqcn,
                            file: self.file.clone(),
                            line,
                            col_start,
                            col_end,
                        });
                    }
                }
            }
            return Union::single(Atomic::TBool);
        }

        let left_ty = self.analyze(&b.left, ctx);
        let right_ty = self.analyze(&b.right, ctx);

        match b.op {
            BinaryOp::Add
            | BinaryOp::Sub
            | BinaryOp::Mul
            | BinaryOp::Div
            | BinaryOp::Mod
            | BinaryOp::Pow => infer_arithmetic(&left_ty, &right_ty),

            BinaryOp::Concat => {
                self.check_implicit_to_string_cast(&left_ty, b.left.span);
                self.check_implicit_to_string_cast(&right_ty, b.right.span);
                Union::single(Atomic::TString)
            }

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

    fn check_implicit_to_string_cast(&mut self, ty: &Union, span: Span) {
        for atomic in &ty.types {
            if let Atomic::TNamedObject { fqcn, .. } = atomic {
                let fqcn_str = fqcn.as_ref();
                if !crate::db::has_method_in_chain(self.db, fqcn_str, "__toString")
                    && !crate::db::extends_or_implements_via_db(self.db, fqcn_str, "Stringable")
                {
                    self.emit(
                        IssueKind::ImplicitToStringCast {
                            class: fqcn_str.to_string(),
                        },
                        Severity::Warning,
                        span,
                    );
                }
            }
        }
    }
}
