use super::helpers::is_non_empty_when_concat;
use super::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Type};
use php_ast::ast::CastKind;
use php_ast::owned::Expr;

/// Try to fold a `(int)` cast of a single known literal to a literal int result.
fn fold_int_cast(ty: &Type) -> Option<Type> {
    if ty.types.len() != 1 {
        return None;
    }
    let lit = match &ty.types[0] {
        Atomic::TLiteralInt(n) => *n,
        Atomic::TTrue => 1,
        Atomic::TFalse => 0,
        Atomic::TLiteralString(s) => {
            let t = s.trim();
            // PHP truncates floats: "3.7" → 3
            if let Ok(n) = t.parse::<i64>() {
                n
            } else if let Ok(f) = t.parse::<f64>() {
                f as i64
            } else {
                return None;
            }
        }
        _ => return None,
    };
    Some(Type::single(Atomic::TLiteralInt(lit)))
}

/// Try to fold a `(string)` cast of a single known literal to a literal string result.
fn fold_string_cast(ty: &Type) -> Option<Type> {
    if ty.types.len() != 1 {
        return None;
    }
    let s = match &ty.types[0] {
        Atomic::TLiteralInt(n) => n.to_string(),
        Atomic::TTrue => "1".to_string(),
        Atomic::TFalse => String::new(),
        Atomic::TLiteralString(s) => s.as_ref().to_string(),
        Atomic::TLiteralFloat(hi, lo) => {
            let bits = ((*hi as u64) << 32) | (*lo as u32 as u64);
            f64::from_bits(bits).to_string()
        }
        _ => return None,
    };
    Some(Type::single(Atomic::TLiteralString(s.into())))
}

/// Try to fold a `(bool)` cast of a single known literal to `TTrue` or `TFalse`.
fn fold_bool_cast(ty: &Type) -> Option<Type> {
    if ty.types.len() != 1 {
        return None;
    }
    let result = match &ty.types[0] {
        Atomic::TLiteralInt(0) | Atomic::TFalse => false,
        Atomic::TLiteralInt(_) | Atomic::TTrue => true,
        Atomic::TLiteralString(s) if s.is_empty() || s.as_ref() == "0" => false,
        Atomic::TLiteralString(_) => true,
        _ => return None,
    };
    Some(Type::single(if result {
        Atomic::TTrue
    } else {
        Atomic::TFalse
    }))
}

/// Returns true for atomic types that are safely castable to any scalar.
/// Used to suppress InvalidCast on union types that include both "bad" atoms
/// (arrays/objects) and "good" atoms (scalars) — e.g. `string|array|bool|null`
/// from console option() return types.
fn is_scalar_safe(t: &Atomic) -> bool {
    t.is_string()
        || t.is_int()
        || matches!(
            t,
            Atomic::TFloat
                | Atomic::TLiteralFloat(..)
                | Atomic::TBool
                | Atomic::TTrue
                | Atomic::TFalse
                | Atomic::TNull
                | Atomic::TMixed
                | Atomic::TScalar
        )
}

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_cast(
        &mut self,
        kind: &CastKind,
        inner: &Expr,
        ctx: &mut FlowState,
    ) -> Type {
        let inner_ty = self.analyze(inner, ctx);
        match kind {
            CastKind::Int => {
                // Literal fold: (int)42, (int)"123", (int)true, etc.
                if let Some(folded) = fold_int_cast(&inner_ty) {
                    // TLiteralInt input is a redundant cast.
                    if inner_ty.is_single() && inner_ty.contains(|t| t.is_int()) {
                        self.emit(
                            IssueKind::RedundantCast {
                                from: inner_ty.to_string(),
                                to: "int".to_string(),
                            },
                            Severity::Info,
                            inner.span,
                        );
                    }
                    return folded;
                }
                // Check for RedundantCast when already int (non-literal)
                if inner_ty.is_single() && inner_ty.contains(|t| t.is_int()) {
                    self.emit(
                        IssueKind::RedundantCast {
                            from: inner_ty.to_string(),
                            to: "int".to_string(),
                        },
                        Severity::Info,
                        inner.span,
                    );
                }
                // Check for InvalidCast from array/object — only when the union has no
                // scalar-safe atoms.  If the union also contains string/bool/null/int/float,
                // the cast is valid for those branches and we suppress the warning to avoid
                // FPs from over-broad return types (e.g. console option()).
                else if inner_ty.contains(|t| {
                    matches!(
                        t,
                        Atomic::TArray { .. }
                            | Atomic::TNonEmptyArray { .. }
                            | Atomic::TList { .. }
                            | Atomic::TNonEmptyList { .. }
                            | Atomic::TKeyedArray { .. }
                            | Atomic::TNamedObject { .. }
                            | Atomic::TObject
                    )
                }) && !inner_ty.contains(is_scalar_safe)
                {
                    self.emit(
                        IssueKind::InvalidCast {
                            from: inner_ty.to_string(),
                            to: "int".to_string(),
                        },
                        Severity::Warning,
                        inner.span,
                    );
                }
                Type::single(Atomic::TInt)
            }
            CastKind::Float => {
                // Check for RedundantCast when already float
                if inner_ty.is_single()
                    && inner_ty
                        .contains(|t| matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..)))
                {
                    self.emit(
                        IssueKind::RedundantCast {
                            from: inner_ty.to_string(),
                            to: "float".to_string(),
                        },
                        Severity::Info,
                        inner.span,
                    );
                }
                // Check for InvalidCast from array/object — same "no scalars" guard as int.
                else if inner_ty.contains(|t| {
                    matches!(
                        t,
                        Atomic::TArray { .. }
                            | Atomic::TNonEmptyArray { .. }
                            | Atomic::TList { .. }
                            | Atomic::TNonEmptyList { .. }
                            | Atomic::TKeyedArray { .. }
                            | Atomic::TNamedObject { .. }
                            | Atomic::TObject
                    )
                }) && !inner_ty.contains(is_scalar_safe)
                {
                    self.emit(
                        IssueKind::InvalidCast {
                            from: inner_ty.to_string(),
                            to: "float".to_string(),
                        },
                        Severity::Warning,
                        inner.span,
                    );
                }
                Type::single(Atomic::TFloat)
            }
            CastKind::String => {
                // Literal fold: (string)42 → "42", (string)true → "1", etc.
                if let Some(folded) = fold_string_cast(&inner_ty) {
                    // TLiteralString input is a redundant cast.
                    if inner_ty.is_single() && inner_ty.contains(|t| t.is_string()) {
                        self.emit(
                            IssueKind::RedundantCast {
                                from: inner_ty.to_string(),
                                to: "string".to_string(),
                            },
                            Severity::Info,
                            inner.span,
                        );
                    }
                    return folded;
                }
                // Check for RedundantCast when already string (non-literal)
                if inner_ty.is_single() && inner_ty.contains(|t| t.is_string()) {
                    self.emit(
                        IssueKind::RedundantCast {
                            from: inner_ty.to_string(),
                            to: "string".to_string(),
                        },
                        Severity::Info,
                        inner.span,
                    );
                }
                // Check for InvalidCast from array
                else if inner_ty.contains(|t| {
                    matches!(
                        t,
                        Atomic::TArray { .. }
                            | Atomic::TNonEmptyArray { .. }
                            | Atomic::TList { .. }
                            | Atomic::TNonEmptyList { .. }
                            | Atomic::TKeyedArray { .. }
                    )
                }) {
                    self.emit(
                        IssueKind::InvalidCast {
                            from: inner_ty.to_string(),
                            to: "string".to_string(),
                        },
                        Severity::Warning,
                        inner.span,
                    );
                }
                // Check for InvalidCast from a concrete class without __toString.
                // Interfaces/abstract classes are skipped since an implementing subclass
                // may provide __toString; only flag concrete (instantiatable) classes.
                else {
                    for atom in inner_ty.types.iter() {
                        if let Atomic::TNamedObject { fqcn, .. } = atom {
                            let fqcn_str = fqcn.as_str();
                            let here = crate::db::Fqcn::from_str(self.db, fqcn_str);
                            let is_concrete = crate::db::find_class_like(self.db, here)
                                .map(|c| matches!(c, crate::db::ClassLike::Class(_)))
                                .unwrap_or(false);
                            if !is_concrete {
                                continue;
                            }
                            let has_to_string = crate::db::find_method_in_chain(
                                self.db,
                                crate::db::Fqcn::from_str(self.db, fqcn_str),
                                "__tostring",
                            )
                            .is_some();
                            if !has_to_string {
                                self.emit(
                                    IssueKind::InvalidCast {
                                        from: inner_ty.to_string(),
                                        to: "string".to_string(),
                                    },
                                    Severity::Warning,
                                    inner.span,
                                );
                                break;
                            }
                        }
                    }
                }
                if is_non_empty_when_concat(&inner_ty) {
                    Type::single(Atomic::TNonEmptyString)
                } else {
                    Type::single(Atomic::TString)
                }
            }
            CastKind::Bool => {
                // Literal fold: (bool)0 → false, (bool)1 → true, (bool)"" → false, etc.
                if let Some(folded) = fold_bool_cast(&inner_ty) {
                    // TTrue/TFalse input is a redundant cast.
                    if inner_ty.is_single()
                        && inner_ty.contains(|t| {
                            matches!(t, Atomic::TBool | Atomic::TTrue | Atomic::TFalse)
                        })
                    {
                        self.emit(
                            IssueKind::RedundantCast {
                                from: inner_ty.to_string(),
                                to: "bool".to_string(),
                            },
                            Severity::Info,
                            inner.span,
                        );
                    }
                    return folded;
                }
                // Check for RedundantCast when already bool (non-literal)
                if inner_ty.is_single()
                    && inner_ty
                        .contains(|t| matches!(t, Atomic::TBool | Atomic::TTrue | Atomic::TFalse))
                {
                    self.emit(
                        IssueKind::RedundantCast {
                            from: inner_ty.to_string(),
                            to: "bool".to_string(),
                        },
                        Severity::Info,
                        inner.span,
                    );
                }
                Type::single(Atomic::TBool)
            }
            CastKind::Array => {
                // Check for RedundantCast when already array
                if inner_ty.is_single()
                    && inner_ty.contains(|t| {
                        matches!(
                            t,
                            Atomic::TArray { .. }
                                | Atomic::TNonEmptyArray { .. }
                                | Atomic::TList { .. }
                                | Atomic::TNonEmptyList { .. }
                                | Atomic::TKeyedArray { .. }
                        )
                    })
                {
                    self.emit(
                        IssueKind::RedundantCast {
                            from: inner_ty.to_string(),
                            to: "array".to_string(),
                        },
                        Severity::Info,
                        inner.span,
                    );
                }
                Type::single(Atomic::TArray {
                    key: Box::new(Type::single(Atomic::TMixed)),
                    value: Box::new(Type::mixed()),
                })
            }
            CastKind::Object => Type::single(Atomic::TObject),
            CastKind::Unset | CastKind::Void => Type::single(Atomic::TNull),
        }
    }
}
