use super::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Type};
use php_ast::ast::MagicConstKind;
use php_ast::owned::YieldExpr;

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_yield(&mut self, y: &YieldExpr, ctx: &mut FlowState) -> Type {
        if let Some(key) = &y.key {
            self.analyze(key, ctx);
        }
        if let Some(value) = &y.value {
            let ty = self.analyze(value, ctx);
            if y.is_from {
                self.check_yield_from_iterable(&ty, value.span);
            }
        }
        Type::mixed()
    }

    fn check_yield_from_iterable(&mut self, ty: &Type, span: php_ast::Span) {
        if ty.is_mixed() {
            return;
        }

        // Only flag named objects that are not Traversable. Scalars and arrays
        // are handled by other checks (or intentionally not flagged).
        let mut all_invalid_objects = true;
        let mut any_invalid_object = false;
        let mut has_any_object = false;

        for atomic in &ty.types {
            match atomic {
                Atomic::TNamedObject { fqcn, .. } | Atomic::TStaticObject { fqcn, .. } => {
                    has_any_object = true;
                    if crate::db::extends_or_implements(self.db, fqcn.as_ref(), "Traversable") {
                        all_invalid_objects = false;
                    } else {
                        any_invalid_object = true;
                    }
                }
                Atomic::TObject => {
                    // Unknown object type — assume traversable, skip.
                    has_any_object = true;
                    all_invalid_objects = false;
                }
                Atomic::TArray { .. }
                | Atomic::TList { .. }
                | Atomic::TNonEmptyArray { .. }
                | Atomic::TNonEmptyList { .. }
                | Atomic::TKeyedArray { .. } => {
                    all_invalid_objects = false;
                }
                _ => {
                    // Scalars/null/bool/etc. — not flagged as RawObjectIteration.
                    all_invalid_objects = false;
                }
            }
        }

        if !ty.types.is_empty() && has_any_object {
            if all_invalid_objects && any_invalid_object {
                self.emit(
                    IssueKind::RawObjectIteration { ty: ty.to_string() },
                    Severity::Warning,
                    span,
                );
            } else if any_invalid_object {
                self.emit(
                    IssueKind::PossiblyRawObjectIteration { ty: ty.to_string() },
                    Severity::Info,
                    span,
                );
            }
        }
    }

    pub(super) fn analyze_magic_const(kind: &MagicConstKind) -> Type {
        match kind {
            MagicConstKind::Line => Type::single(Atomic::TInt),
            MagicConstKind::File
            | MagicConstKind::Dir
            | MagicConstKind::Function
            | MagicConstKind::Class
            | MagicConstKind::Method
            | MagicConstKind::Namespace
            | MagicConstKind::Trait
            | MagicConstKind::Property => Type::single(Atomic::TString),
        }
    }
}
