use super::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Type};
use php_ast::ast::CastKind;
use php_ast::owned::Expr;

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
                // Check for RedundantCast when already int
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
                // Check for InvalidCast from array/object
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
                }) {
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
                // Check for InvalidCast from array/object
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
                }) {
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
                // Check for RedundantCast when already string
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
                Type::single(Atomic::TString)
            }
            CastKind::Bool => {
                // Check for RedundantCast when already bool
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
