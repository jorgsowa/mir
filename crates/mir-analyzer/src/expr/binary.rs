use super::helpers::infer_arithmetic;
use super::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Type};
use php_ast::ast::BinaryOp;
use php_ast::owned::{BinaryExpr, ExprKind};
use php_ast::Span;
use std::sync::Arc;

pub(super) fn operand_is_non_bitwise(ty: &Type) -> bool {
    if ty.types.is_empty() || ty.is_mixed() {
        return false;
    }
    ty.types.iter().all(|a| match a {
        Atomic::TLiteralString(s) => !is_numeric_string(s),
        Atomic::TArray { .. }
        | Atomic::TList { .. }
        | Atomic::TNonEmptyArray { .. }
        | Atomic::TNonEmptyList { .. }
        | Atomic::TKeyedArray { .. }
        | Atomic::TObject
        | Atomic::TNamedObject { .. }
        | Atomic::TStaticObject { .. }
        | Atomic::TSelf { .. }
        | Atomic::TParent { .. }
        | Atomic::TIntersection { .. }
        | Atomic::TClosure { .. }
        | Atomic::TLiteralEnumCase { .. }
        | Atomic::TBool
        | Atomic::TTrue
        | Atomic::TFalse => true,
        _ => false,
    })
}

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_binary_expr(
        &mut self,
        b: &BinaryExpr,
        span: Span,
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
                    if !crate::db::class_exists(self.db, &fqcn)
                        && !ctx.class_exists_guards.contains(fqcn.as_ref())
                    {
                        self.emit(
                            IssueKind::UndefinedClass { name: resolved },
                            Severity::Error,
                            b.right.span,
                        );
                    } else {
                        let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
                        if let Some(class) = crate::db::find_class_like(self.db, here) {
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
                                    b.right.span,
                                );
                            }
                        }
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
            | BinaryOp::Pow => {
                // `array + array` is the valid array-union operation; everything
                // else requires numeric operands. Flag an operand only when it is
                // *definitely* non-numeric (a non-numeric literal string, array,
                // object, or enum case) so unions / general strings never FP.
                let both_arrays = b.op == BinaryOp::Add
                    && !left_ty.types.is_empty()
                    && left_ty.types.iter().all(Atomic::is_array)
                    && !right_ty.types.is_empty()
                    && right_ty.types.iter().all(Atomic::is_array);
                if !both_arrays {
                    if operand_is_non_numeric(&left_ty) || operand_is_non_numeric(&right_ty) {
                        self.emit(
                            IssueKind::InvalidOperand {
                                op: arithmetic_op_symbol(b.op).to_string(),
                                left: left_ty.to_string(),
                                right: right_ty.to_string(),
                            },
                            Severity::Warning,
                            span,
                        );
                    } else if operand_has_any_non_numeric_member(&left_ty)
                        || operand_has_any_non_numeric_member(&right_ty)
                    {
                        self.emit(
                            IssueKind::PossiblyInvalidOperand {
                                op: arithmetic_op_symbol(b.op).to_string(),
                                left: left_ty.to_string(),
                                right: right_ty.to_string(),
                            },
                            Severity::Info,
                            span,
                        );
                    } else if matches!(b.op, BinaryOp::Div | BinaryOp::Mod)
                        && operand_contains_null(&right_ty)
                    {
                        self.emit(
                            IssueKind::PossiblyNullOperand {
                                op: arithmetic_op_symbol(b.op).to_string(),
                                ty: right_ty.to_string(),
                            },
                            Severity::Info,
                            span,
                        );
                    }
                }
                infer_arithmetic(&left_ty, &right_ty)
            }

            BinaryOp::Concat => {
                self.check_implicit_to_string_cast(&left_ty, b.left.span);
                self.check_implicit_to_string_cast(&right_ty, b.right.span);
                // Flag when a union member is an array (not stringifiable).
                if operand_has_any_array_member(&left_ty) || operand_has_any_array_member(&right_ty)
                {
                    self.emit(
                        IssueKind::PossiblyInvalidOperand {
                            op: ".".to_string(),
                            left: left_ty.to_string(),
                            right: right_ty.to_string(),
                        },
                        Severity::Info,
                        span,
                    );
                } else if operand_contains_null(&left_ty) {
                    self.emit(
                        IssueKind::PossiblyNullOperand {
                            op: ".".to_string(),
                            ty: left_ty.to_string(),
                        },
                        Severity::Info,
                        b.left.span,
                    );
                } else if operand_contains_null(&right_ty) {
                    self.emit(
                        IssueKind::PossiblyNullOperand {
                            op: ".".to_string(),
                            ty: right_ty.to_string(),
                        },
                        Severity::Info,
                        b.right.span,
                    );
                }
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
            | BinaryOp::ShiftRight => {
                if operand_is_non_bitwise(&left_ty) || operand_is_non_bitwise(&right_ty) {
                    self.emit(
                        IssueKind::InvalidOperand {
                            op: bitwise_op_symbol(b.op).to_string(),
                            left: left_ty.to_string(),
                            right: right_ty.to_string(),
                        },
                        Severity::Warning,
                        span,
                    );
                }
                Type::single(Atomic::TInt)
            }

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

/// The source symbol for an arithmetic operator, for `InvalidOperand` messages.
fn arithmetic_op_symbol(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Mod => "%",
        BinaryOp::Pow => "**",
        _ => "?",
    }
}

/// The source symbol for a bitwise operator, for `InvalidOperand` messages.
fn bitwise_op_symbol(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::BitwiseAnd => "&",
        BinaryOp::BitwiseOr => "|",
        BinaryOp::BitwiseXor => "^",
        BinaryOp::ShiftLeft => "<<",
        BinaryOp::ShiftRight => ">>",
        _ => "?",
    }
}

/// Whether `s` is a PHP numeric string (so `"5" + 3` is valid). Conservative:
/// anything that doesn't cleanly parse is treated as numeric to avoid false
/// positives.
fn is_numeric_string(s: &str) -> bool {
    let t = s.trim();
    !t.is_empty() && (t.parse::<i64>().is_ok() || t.parse::<f64>().is_ok())
}

/// Whether every member of `ty` is *definitely* a non-numeric value for
/// arithmetic. Returns `false` for empty/`mixed`/unions that include any
/// possibly-numeric member, so only clear errors are flagged.
fn operand_is_non_numeric(ty: &Type) -> bool {
    if ty.types.is_empty() || ty.is_mixed() {
        return false;
    }
    ty.types.iter().all(|a| match a {
        Atomic::TLiteralString(s) => !is_numeric_string(s),
        Atomic::TArray { .. }
        | Atomic::TList { .. }
        | Atomic::TNonEmptyArray { .. }
        | Atomic::TNonEmptyList { .. }
        | Atomic::TKeyedArray { .. }
        | Atomic::TObject
        | Atomic::TNamedObject { .. }
        | Atomic::TStaticObject { .. }
        | Atomic::TSelf { .. }
        | Atomic::TParent { .. }
        | Atomic::TIntersection { .. }
        | Atomic::TClosure { .. }
        | Atomic::TLiteralEnumCase { .. } => true,
        // int/float/numeric/bool/null and general/class strings are left alone.
        _ => false,
    })
}

/// Whether `ty` has *any* member that is definitely non-numeric but the type
/// as a whole is not all-non-numeric. Used for `PossiblyInvalidOperand`.
fn operand_has_any_non_numeric_member(ty: &Type) -> bool {
    if ty.types.is_empty() || ty.is_mixed() {
        return false;
    }
    ty.types.iter().any(|a| match a {
        Atomic::TLiteralString(s) => !is_numeric_string(s),
        Atomic::TArray { .. }
        | Atomic::TList { .. }
        | Atomic::TNonEmptyArray { .. }
        | Atomic::TNonEmptyList { .. }
        | Atomic::TKeyedArray { .. }
        | Atomic::TObject
        | Atomic::TNamedObject { .. }
        | Atomic::TStaticObject { .. }
        | Atomic::TSelf { .. }
        | Atomic::TParent { .. }
        | Atomic::TIntersection { .. }
        | Atomic::TClosure { .. }
        | Atomic::TLiteralEnumCase { .. } => true,
        _ => false,
    }) && !operand_is_non_numeric(ty)
}

/// Whether `ty` contains `null` (potential division-by-zero when used as divisor).
pub(super) fn operand_contains_null(ty: &Type) -> bool {
    ty.types.iter().any(|a| matches!(a, Atomic::TNull))
}

/// Whether `ty` has any array member (arrays cannot be concatenated).
fn operand_has_any_array_member(ty: &Type) -> bool {
    ty.types.iter().any(|a| {
        matches!(
            a,
            Atomic::TArray { .. }
                | Atomic::TList { .. }
                | Atomic::TNonEmptyArray { .. }
                | Atomic::TNonEmptyList { .. }
                | Atomic::TKeyedArray { .. }
        )
    })
}
