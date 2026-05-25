use super::helpers::infer_arithmetic;
use super::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Type};
use php_ast::ast::BinaryOp;
use php_ast::owned::{BinaryExpr, ExprKind};
use php_ast::Span;
use std::sync::Arc;

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_binary_expr(
        &mut self,
        b: &BinaryExpr,
        _span: Span,
        ctx: &mut FlowState,
    ) -> Type {
        use BinaryOp as B;
        if matches!(
            b.op,
            B::BooleanAnd | B::LogicalAnd | B::BooleanOr | B::LogicalOr
        ) {
            let _left_ty = self.analyze(&b.left, ctx);
            let mut right_ctx = ctx.branch();
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
            for (name, ty) in right_ctx.vars.iter() {
                if !ctx.vars.contains_key(name) {
                    std::sync::Arc::make_mut(&mut ctx.vars).insert(*name, ty.clone());
                    std::sync::Arc::make_mut(&mut ctx.possibly_assigned_vars).insert(*name);
                }
            }
            return Type::single(Atomic::TBool);
        }

        if b.op == B::Instanceof {
            let _left_ty = self.analyze(&b.left, ctx);
            if let ExprKind::Identifier(name) = &b.right.kind {
                let resolved = crate::db::resolve_name(self.db, &self.file, name.as_ref());
                let fqcn: Arc<str> = Arc::from(resolved.as_str());
                if !matches!(resolved.as_str(), "self" | "static" | "parent") {
                    if !crate::db::class_exists(self.db, &fqcn) {
                        self.emit(
                            IssueKind::UndefinedClass { name: resolved },
                            Severity::Error,
                            b.right.span,
                        );
                    }
                    self.record_ref(fqcn, b.right.span);
                }
            }
            return Type::single(Atomic::TBool);
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
                Type::single(Atomic::TString)
            }

            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Identical
            | BinaryOp::NotIdentical
            | BinaryOp::Less
            | BinaryOp::Greater
            | BinaryOp::LessOrEqual
            | BinaryOp::GreaterOrEqual => Type::single(Atomic::TBool),

            BinaryOp::Spaceship => Type::single(Atomic::TIntRange {
                min: Some(-1),
                max: Some(1),
            }),

            BinaryOp::BooleanAnd
            | BinaryOp::BooleanOr
            | BinaryOp::LogicalAnd
            | BinaryOp::LogicalOr
            | BinaryOp::LogicalXor => Type::single(Atomic::TBool),

            BinaryOp::BitwiseAnd
            | BinaryOp::BitwiseOr
            | BinaryOp::BitwiseXor
            | BinaryOp::ShiftLeft
            | BinaryOp::ShiftRight => Type::single(Atomic::TInt),

            BinaryOp::Pipe => right_ty,
            BinaryOp::Instanceof => Type::single(Atomic::TBool),
        }
    }

    fn check_implicit_to_string_cast(&mut self, ty: &Type, span: Span) {
        for atomic in &ty.types {
            if let Atomic::TNamedObject { fqcn, .. } = atomic {
                let fqcn_str = fqcn.as_ref();
                if !crate::db::has_method_in_chain(self.db, fqcn_str, "__toString")
                    && !crate::db::extends_or_implements(self.db, fqcn_str, "Stringable")
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
