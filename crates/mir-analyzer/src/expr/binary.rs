use super::helpers::{
    as_concat_str, infer_arithmetic, infer_div, infer_int_range_arithmetic,
    is_non_empty_when_concat,
};
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
        // PHP coerces bool→int and allows string bitwise ops ("a"&"b"), so these
        // are valid bitwise operands even though they aren't integers.
        Atomic::TBool | Atomic::TTrue | Atomic::TFalse | Atomic::TLiteralString(_) => false,
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
            // Propagate reads and consumed write locations from the short-circuit
            // RHS back to the parent. Without this, a variable consumed only in
            // the RHS (e.g. `A && $x['key']`) would not be marked as consumed in
            // the parent, causing its pending write to survive into a catch block
            // that resets last_write_locs to the pre-try state.
            ctx.absorb_branch_reads(&right_ctx);
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
            // When the RHS is not a static class name but a variable or other
            // expression (`$x instanceof $class`), analyze it so that the
            // variable is marked as consumed — otherwise it is falsely reported
            // as unused.
            if !matches!(b.right.kind, ExprKind::Identifier(_)) {
                let right_ty = self.analyze(&b.right, ctx);
                // `instanceof $cls` where `$cls` holds a known class-string
                // (`$cls = Foo::class;`) is a real reference to `Foo` — record it,
                // matching the plain `instanceof Foo` form below, or a class
                // checked only this way is falsely flagged unused with no
                // go-to-definition from the check site.
                for atomic in &right_ty.types {
                    if let Atomic::TClassString(Some(fqcn)) = atomic {
                        let fqcn: Arc<str> = Arc::from(fqcn.as_ref());
                        self.record_ref(Arc::from(format!("cls:{fqcn}")), b.right.span);
                        self.record_symbol(
                            b.right.span,
                            crate::symbol::ReferenceKind::ClassReference(fqcn),
                            mir_types::Type::single(mir_types::Atomic::TClassString(None)),
                        );
                    }
                }
            }
            if let ExprKind::Identifier(name) = &b.right.kind {
                let resolved = crate::db::resolve_name(self.db, &self.file, name.as_ref());
                let fqcn: Arc<str> = Arc::from(resolved.as_str());
                if !matches!(resolved.as_str(), "self" | "static" | "parent") {
                    if !crate::db::class_exists(self.db, &fqcn)
                        && !ctx.is_class_guarded(fqcn.as_ref())
                    {
                        self.emit(
                            IssueKind::UndefinedClass { name: resolved },
                            Severity::Error,
                            b.right.span,
                        );
                    } else {
                        let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
                        if let Some(class) = crate::db::find_class_like(self.db, here) {
                            if let Some((used, canonical_str)) =
                                crate::fqcn_case_mismatch(fqcn.as_ref(), class.fqcn().as_ref())
                            {
                                self.emit(
                                    IssueKind::WrongCaseClass {
                                        used,
                                        canonical: canonical_str,
                                    },
                                    Severity::Info,
                                    b.right.span,
                                );
                            }
                        }
                    }
                    self.record_ref(Arc::from(format!("cls:{fqcn}")), b.right.span);
                    self.record_symbol(
                        b.right.span,
                        crate::symbol::ReferenceKind::ClassReference(fqcn),
                        mir_types::Type::single(mir_types::Atomic::TClassString(None)),
                    );
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
                    } else if matches!(b.op, BinaryOp::Div | BinaryOp::Mod)
                        && operand_is_definitely_zero(&right_ty)
                    {
                        // The constant-folder below (infer_int_range_arithmetic)
                        // already detects a literal-zero divisor via its `r != 0`
                        // guards, but only to skip folding — it never reports
                        // this as the unconditional runtime DivisionByZeroError
                        // that it is.
                        self.emit(
                            IssueKind::DivisionByZero {
                                op: arithmetic_op_symbol(b.op).to_string(),
                            },
                            Severity::Error,
                            span,
                        );
                    }
                }
                infer_int_range_arithmetic(&left_ty, &right_ty, b.op).unwrap_or_else(|| {
                    if b.op == BinaryOp::Div {
                        infer_div(&left_ty, &right_ty)
                    } else {
                        infer_arithmetic(&left_ty, &right_ty)
                    }
                })
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
                // Literal string fold: "foo" . "bar" = "foobar".
                // Also folds single literal int operands to their string repr.
                // Cap at 1000 chars to avoid storing arbitrarily large strings.
                if let (Some(l), Some(r)) = (as_concat_str(&left_ty), as_concat_str(&right_ty)) {
                    let combined = format!("{l}{r}");
                    if combined.len() <= 1000 {
                        return Type::single(Atomic::TLiteralString(combined.into()));
                    }
                    return Type::single(Atomic::TNonEmptyString);
                }
                // If either operand is guaranteed non-empty when cast to string,
                // the result is also non-empty (concatenation can only add characters).
                if is_non_empty_when_concat(&left_ty) || is_non_empty_when_concat(&right_ty) {
                    return Type::single(Atomic::TNonEmptyString);
                }
                Type::single(Atomic::TString)
            }

            BinaryOp::Identical | BinaryOp::NotIdentical => {
                if !crate::contradiction::types_can_be_identical(&left_ty, &right_ty) {
                    let op = if b.op == BinaryOp::Identical {
                        "==="
                    } else {
                        "!=="
                    };
                    self.emit(
                        IssueKind::ImpossibleIdenticalComparison {
                            op: op.to_string(),
                            left: left_ty.to_string(),
                            right: right_ty.to_string(),
                        },
                        Severity::Warning,
                        span,
                    );
                }
                Type::single(Atomic::TBool)
            }

            BinaryOp::Equal | BinaryOp::NotEqual => {
                if !crate::contradiction::types_can_be_loose_equal(
                    &left_ty,
                    &right_ty,
                    self.php_version,
                ) {
                    let op = if b.op == BinaryOp::Equal { "==" } else { "!=" };
                    self.emit(
                        IssueKind::ImpossibleLooseComparison {
                            op: op.to_string(),
                            left: left_ty.to_string(),
                            right: right_ty.to_string(),
                        },
                        Severity::Warning,
                        span,
                    );
                }
                Type::single(Atomic::TBool)
            }

            BinaryOp::Less
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
                infer_bitwise_range(b.op, &left_ty, &right_ty)
                    .unwrap_or_else(|| Type::single(Atomic::TInt))
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

/// Whether `ty` is DEFINITELY the literal `0` (int or float) — every atomic in
/// the union reduces to zero, not just some of them. Used to catch an
/// unconditional division-by-zero, as opposed to `operand_contains_null`'s
/// weaker "might be" check.
pub(crate) fn operand_is_definitely_zero(ty: &Type) -> bool {
    !ty.types.is_empty()
        && ty
            .types
            .iter()
            .all(|a| matches!(a, Atomic::TLiteralInt(0) | Atomic::TLiteralFloat(0, 0)))
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

/// Infer a tighter return type for bitwise operations when possible:
///
/// - `$x & mask` / `mask & $x` where mask is a non-negative literal:
///   result is always in `[0, mask]` regardless of `$x`.
/// - `$x >> n` where n >= 0 and $x is non-negative:
///   result is non-negative (right shift of a non-negative value stays non-negative).
///
/// Returns `None` to fall through to the `TInt` default.
fn infer_bitwise_range(op: BinaryOp, left: &Type, right: &Type) -> Option<Type> {
    match op {
        BinaryOp::BitwiseAnd => {
            // If either side is a known non-negative literal, the result is in [0, mask].
            let mask = extract_non_negative_literal(right)
                .or_else(|| extract_non_negative_literal(left))?;
            let atom = if mask == 0 {
                Atomic::TLiteralInt(0)
            } else {
                Atomic::TIntRange {
                    min: Some(0),
                    max: Some(mask),
                }
            };
            Some(Type::single(atom))
        }
        BinaryOp::ShiftRight => {
            // `$x >> n` where n >= 0 and $x is non-negative stays non-negative.
            let _shift = extract_non_negative_literal(right)?;
            if is_non_negative_int(left) {
                Some(Type::single(Atomic::TNonNegativeInt))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// If `ty` is a single non-negative literal integer, return its value.
fn extract_non_negative_literal(ty: &Type) -> Option<i64> {
    if ty.types.len() == 1 {
        if let Atomic::TLiteralInt(n) = ty.types[0] {
            if n >= 0 {
                return Some(n);
            }
        }
    }
    None
}

/// True when all atoms of `ty` are non-negative integer types.
fn is_non_negative_int(ty: &Type) -> bool {
    !ty.types.is_empty()
        && ty.types.iter().all(|a| match a {
            Atomic::TNonNegativeInt | Atomic::TPositiveInt => true,
            Atomic::TLiteralInt(n) => *n >= 0,
            Atomic::TIntRange { min, .. } => min.is_some_and(|m| m >= 0),
            _ => false,
        })
}
