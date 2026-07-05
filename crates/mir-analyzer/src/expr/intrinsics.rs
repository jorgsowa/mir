use super::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Type};
use php_ast::ast::MagicConstKind;
use php_ast::owned::YieldExpr;

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_yield(&mut self, y: &YieldExpr, ctx: &mut FlowState) -> Type {
        let key_ty = y.key.as_ref().map(|key| self.analyze(key, ctx));
        let value_ty = y.value.as_ref().map(|value| {
            let ty = self.analyze(value, ctx);
            if y.is_from {
                self.check_yield_from_iterable(&ty, value.span);
            }
            ty
        });

        // Record this yield's (key, value) contribution so the enclosing
        // function's inferred return type can be built as
        // `Generator<TKey, TValue, TSend, TReturn>`. `yield from $iterable`
        // has no explicit key/value sub-expressions — its contribution comes
        // from the delegated iterable's own key/value types instead.
        let (contributed_key, contributed_value) = if y.is_from {
            value_ty
                .as_ref()
                .map(extract_iterable_key_value)
                .unwrap_or_else(|| (Type::mixed(), Type::mixed()))
        } else {
            (
                key_ty.unwrap_or_else(|| Type::single(Atomic::TInt)),
                value_ty.unwrap_or_else(|| Type::single(Atomic::TNull)),
            )
        };
        self.yielded_types
            .push((contributed_key, contributed_value));

        // The value of a `yield` expression itself is whatever a future
        // `Generator::send()` call provides — mir does not infer that from
        // usage, so it stays `mixed` (also mir's `TSend` for the inferred
        // `Generator<K, V, TSend, R>`).
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

/// (key, value) that `yield` contributes when delegating to `$iterable` via
/// `yield from`. Arrays/lists/shapes contribute their own key/value types; a
/// `Generator<K, V, ...>` (or other named object with type params, taken
/// positionally as `K, V`) contributes its first one or two type params.
/// Anything else (plain `Traversable`, `mixed`, …) falls back to
/// `mixed`/`mixed` — no false precision from a source mir can't see into.
fn extract_iterable_key_value(ty: &Type) -> (Type, Type) {
    let mut key = Type::empty();
    let mut value = Type::empty();
    let mut matched_any = false;

    for atomic in &ty.types {
        match atomic {
            Atomic::TArray { key: k, value: v } | Atomic::TNonEmptyArray { key: k, value: v } => {
                key.merge_with(k);
                value.merge_with(v);
                matched_any = true;
            }
            Atomic::TList { value: v } | Atomic::TNonEmptyList { value: v } => {
                key.add_type(Atomic::TInt);
                value.merge_with(v);
                matched_any = true;
            }
            Atomic::TKeyedArray { properties, .. } => {
                for (k, prop) in properties.iter() {
                    key.add_type(match k {
                        mir_types::atomic::ArrayKey::Int(_) => Atomic::TInt,
                        mir_types::atomic::ArrayKey::String(_) => Atomic::TString,
                    });
                    value.merge_with(&prop.ty);
                }
                matched_any = true;
            }
            Atomic::TNamedObject { type_params, .. } if type_params.len() >= 2 => {
                key.merge_with(&type_params[0]);
                value.merge_with(&type_params[1]);
                matched_any = true;
            }
            Atomic::TNamedObject { type_params, .. } if type_params.len() == 1 => {
                key.add_type(Atomic::TInt);
                value.merge_with(&type_params[0]);
                matched_any = true;
            }
            _ => {}
        }
    }

    if matched_any {
        (key, value)
    } else {
        (Type::mixed(), Type::mixed())
    }
}
