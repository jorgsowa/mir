use super::ExpressionAnalyzer;
use crate::context::Context;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Union};
use php_ast::ast::{CastKind, Expr};

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_cast<'arena, 'src>(
        &mut self,
        kind: &CastKind,
        inner: &Expr<'arena, 'src>,
        ctx: &mut Context,
    ) -> Union {
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
                Union::single(Atomic::TInt)
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
                Union::single(Atomic::TFloat)
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
                Union::single(Atomic::TString)
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
                Union::single(Atomic::TBool)
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
                Union::single(Atomic::TArray {
                    key: Box::new(Union::single(Atomic::TMixed)),
                    value: Box::new(Union::mixed()),
                })
            }
            CastKind::Object => Union::single(Atomic::TObject),
            CastKind::Unset | CastKind::Void => Union::single(Atomic::TNull),
        }
    }
}
