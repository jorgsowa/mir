use super::binary::{operand_contains_null, operand_is_non_bitwise, operand_is_non_numeric};
use super::helpers::{extract_simple_var, infer_int_range_arithmetic};
use super::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Type};
use php_ast::ast::{BinaryOp, UnaryPostfixOp, UnaryPrefixOp};
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

/// Returns true when `ty` is a non-empty literal string (PHP's string++ is deprecated/invalid).
fn operand_is_non_empty_literal_string(ty: &Type) -> bool {
    !ty.types.is_empty()
        && !ty.is_mixed()
        && ty.types.iter().all(|a| match a {
            Atomic::TLiteralString(s) => !s.is_empty(),
            _ => false,
        })
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
            UnaryPrefixOp::Negate | UnaryPrefixOp::Plus => {
                // Same operand check as binary arithmetic's `+`/`-` — an array,
                // object, enum case, or non-numeric string throws a real PHP
                // `TypeError`, not the "coerce to 0" fallback `negate_type`'s
                // catch-all assumes.
                if operand_is_non_numeric(&operand_ty) {
                    self.emit(
                        IssueKind::InvalidOperand {
                            op: if u.op == UnaryPrefixOp::Negate { "-" } else { "+" }.to_string(),
                            left: operand_ty.to_string(),
                            right: String::new(),
                        },
                        Severity::Warning,
                        u.operand.span,
                    );
                }
                if u.op == UnaryPrefixOp::Negate {
                    negate_type(&operand_ty)
                } else {
                    operand_ty
                }
            }
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
                } else if operand_contains_null(&operand_ty) {
                    self.emit(
                        IssueKind::PossiblyNullOperand {
                            op: "~".to_string(),
                            ty: operand_ty.to_string(),
                        },
                        Severity::Info,
                        u.operand.span,
                    );
                }
                Type::single(Atomic::TInt)
            }
            UnaryPrefixOp::PreIncrement | UnaryPrefixOp::PreDecrement => {
                // Same operand check as postfix ++/-- — the same PHP warning/
                // deprecation fires for both forms, only postfix was flagged.
                if operand_is_definitely_bool(&operand_ty)
                    || operand_is_non_empty_literal_string(&operand_ty)
                {
                    self.emit(
                        IssueKind::InvalidOperand {
                            op: if u.op == UnaryPrefixOp::PreIncrement {
                                "++"
                            } else {
                                "--"
                            }
                            .to_string(),
                            left: operand_ty.to_string(),
                            right: String::new(),
                        },
                        Severity::Warning,
                        u.operand.span,
                    );
                }
                if let Some(var_name) = extract_simple_var(&u.operand) {
                    let ty = ctx.get_var(&var_name);
                    let new_ty = if ty.contains(|t| {
                        matches!(
                            t,
                            Atomic::TFloat | Atomic::TIntegralFloat | Atomic::TLiteralFloat(..)
                        )
                    }) {
                        Type::single(Atomic::TFloat)
                    } else {
                        let op = if u.op == UnaryPrefixOp::PreIncrement {
                            BinaryOp::Add
                        } else {
                            BinaryOp::Sub
                        };
                        let one = Type::single(Atomic::TLiteralInt(1));
                        infer_int_range_arithmetic(&ty, &one, op)
                            .unwrap_or_else(|| Type::single(Atomic::TInt))
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
                let op_str = if u.op == UnaryPostfixOp::PostIncrement {
                    "++"
                } else {
                    "--"
                };
                // Flag increment/decrement on a definitely-bool operand (PHP
                // silently no-ops; TFalse reserved for future FalseOperand kind)
                // or on a non-empty literal string (deprecated in PHP 8.3+).
                if operand_is_definitely_bool(&operand_ty)
                    || operand_is_non_empty_literal_string(&operand_ty)
                {
                    self.emit(
                        IssueKind::InvalidOperand {
                            op: op_str.to_string(),
                            left: operand_ty.to_string(),
                            right: String::new(),
                        },
                        Severity::Warning,
                        u.operand.span,
                    );
                }
                if let Some(var_name) = extract_simple_var(&u.operand) {
                    let new_ty = if operand_ty.contains(|t| {
                        matches!(
                            t,
                            Atomic::TFloat | Atomic::TIntegralFloat | Atomic::TLiteralFloat(..)
                        )
                    }) {
                        Type::single(Atomic::TFloat)
                    } else {
                        let op = if u.op == UnaryPostfixOp::PostIncrement {
                            BinaryOp::Add
                        } else {
                            BinaryOp::Sub
                        };
                        let one = Type::single(Atomic::TLiteralInt(1));
                        infer_int_range_arithmetic(&operand_ty, &one, op)
                            .unwrap_or_else(|| Type::single(Atomic::TInt))
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

/// Infer the result type of the unary negation operator (`-$x`).
///
/// Propagates literal and range information through negation so that
/// e.g. `-7` gives `int<-7,-7>` rather than bare `int`, and
/// `-positive-int` gives `negative-int`.
fn negate_type(ty: &Type) -> Type {
    if ty.is_mixed() {
        return Type::mixed();
    }
    // Safe negation of an optional bound: None (unbounded) stays None; overflow
    // (i64::MIN.checked_neg() == None) falls back to None (unbounded).
    let neg_bound = |v: Option<i64>| v.and_then(|n| n.checked_neg());

    let mut result = Type::empty();
    for a in &ty.types {
        let atom = match a {
            Atomic::TLiteralInt(n) => {
                if let Some(neg) = n.checked_neg() {
                    Atomic::TLiteralInt(neg)
                } else {
                    // i64::MIN — falls back to bare int.
                    Atomic::TInt
                }
            }
            Atomic::TPositiveInt => Atomic::TNegativeInt,
            Atomic::TNegativeInt => Atomic::TPositiveInt,
            // non-negative-int negated is int<-∞, 0> (no named type — use TIntRange).
            Atomic::TNonNegativeInt => Atomic::TIntRange {
                min: None,
                max: Some(0),
            },
            Atomic::TInt => Atomic::TInt,
            Atomic::TIntRange { min, max } => {
                let new_min = neg_bound(*max);
                let new_max = neg_bound(*min);
                match (new_min, new_max) {
                    (Some(1), None) => Atomic::TPositiveInt,
                    (Some(0), None) => Atomic::TNonNegativeInt,
                    (None, Some(-1)) => Atomic::TNegativeInt,
                    (None, None) => Atomic::TInt,
                    _ => Atomic::TIntRange {
                        min: new_min,
                        max: new_max,
                    },
                }
            }
            Atomic::TFloat | Atomic::TIntegralFloat | Atomic::TLiteralFloat(..) => Atomic::TFloat,
            _ => {
                // Non-numeric operands: fallback to int (PHP coerces to 0 then negates).
                if ty.contains(|t| t.is_int()) {
                    Atomic::TInt
                } else {
                    Atomic::TFloat
                }
            }
        };
        result.add_type(atom);
    }
    result
}
