use super::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Type};
use php_ast::owned::{ArrayAccessExpr, ArrayElement, Expr, ExprKind};
use std::sync::Arc;

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_array(&mut self, elements: &[ArrayElement], ctx: &mut FlowState) -> Type {
        use mir_types::atomic::{ArrayKey, KeyedProperty};

        if elements.is_empty() {
            return Type::single(Atomic::TKeyedArray {
                properties: indexmap::IndexMap::new(),
                is_open: false,
                is_list: true,
            });
        }

        let mut keyed_props: indexmap::IndexMap<ArrayKey, KeyedProperty> =
            indexmap::IndexMap::new();
        let mut is_list = true;
        let mut can_be_keyed = true;
        let mut next_int_key: i64 = 0;

        for elem in elements.iter() {
            if elem.unpack {
                self.analyze(&elem.value, ctx);
                can_be_keyed = false;
                break;
            }
            let value_ty = self.analyze(&elem.value, ctx);
            let array_key = if let Some(key_expr) = &elem.key {
                is_list = false;
                let key_ty = self.analyze(key_expr, ctx);
                // Float keys are silently truncated to int in PHP
                if key_ty.contains(|t| matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..))) {
                    self.emit(
                        IssueKind::ImplicitFloatToIntCast {
                            from: key_ty.to_string(),
                        },
                        Severity::Warning,
                        key_expr.span,
                    );
                }
                match key_ty.types.as_slice() {
                    [Atomic::TLiteralString(s)] => ArrayKey::String(s.clone()),
                    [Atomic::TLiteralInt(i)] => {
                        next_int_key = *i + 1;
                        ArrayKey::Int(*i)
                    }
                    _ => {
                        can_be_keyed = false;
                        break;
                    }
                }
            } else {
                let k = ArrayKey::Int(next_int_key);
                next_int_key += 1;
                k
            };
            keyed_props.insert(
                array_key,
                KeyedProperty {
                    ty: value_ty,
                    optional: false,
                },
            );
        }

        if can_be_keyed {
            return Type::single(Atomic::TKeyedArray {
                properties: keyed_props,
                is_open: false,
                is_list,
            });
        }

        // Fallback: generic TArray
        let mut all_value_types = Type::empty();
        let mut key_union = Type::empty();
        let mut has_unpack = false;
        for elem in elements.iter() {
            let value_ty = self.analyze(&elem.value, ctx);
            if elem.unpack {
                has_unpack = true;
            } else {
                all_value_types.merge_with(&value_ty);
                if let Some(key_expr) = &elem.key {
                    let key_ty = self.analyze(key_expr, ctx);
                    // Float keys are silently truncated to int in PHP
                    if key_ty.contains(|t| matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..)))
                    {
                        self.emit(
                            IssueKind::ImplicitFloatToIntCast {
                                from: key_ty.to_string(),
                            },
                            Severity::Warning,
                            key_expr.span,
                        );
                    }
                    key_union.merge_with(&key_ty);
                } else {
                    key_union.add_type(Atomic::TInt);
                }
            }
        }
        if has_unpack {
            return Type::single(Atomic::TArray {
                key: Box::new(Type::single(Atomic::TMixed)),
                value: Box::new(Type::mixed()),
            });
        }
        if key_union.is_empty() {
            key_union.add_type(Atomic::TInt);
        }
        Type::single(Atomic::TArray {
            key: Box::new(key_union),
            value: Box::new(all_value_types),
        })
    }

    pub(super) fn analyze_array_access(
        &mut self,
        aa: &ArrayAccessExpr,
        expr: &Expr,
        ctx: &mut FlowState,
    ) -> Type {
        let arr_ty = self.analyze(&aa.array, ctx);
        if let Some(idx) = &aa.index {
            let idx_ty = self.analyze(idx, ctx);
            // Float keys are silently truncated to int in PHP
            if idx_ty.contains(|t| matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..))) {
                self.emit(
                    IssueKind::ImplicitFloatToIntCast {
                        from: idx_ty.to_string(),
                    },
                    Severity::Warning,
                    idx.span,
                );
            }
        }

        if arr_ty.contains(|t| matches!(t, Atomic::TNull)) && arr_ty.is_single() {
            self.emit(IssueKind::NullArrayAccess, Severity::Error, expr.span);
            return Type::mixed();
        }
        if arr_ty.is_nullable() {
            self.emit(
                IssueKind::PossiblyNullArrayAccess,
                Severity::Info,
                expr.span,
            );
        }

        let literal_key: Option<mir_types::atomic::ArrayKey> =
            aa.index.as_ref().and_then(|idx| match &idx.kind {
                ExprKind::String(s) => {
                    Some(mir_types::atomic::ArrayKey::String(Arc::from(s.as_ref())))
                }
                ExprKind::Int(i) => Some(mir_types::atomic::ArrayKey::Int(*i)),
                _ => None,
            });

        let idx_span = aa.index.as_ref().map(|i| i.span).unwrap_or(expr.span);

        for atomic in &arr_ty.types {
            match atomic {
                Atomic::TKeyedArray {
                    properties,
                    is_open,
                    ..
                } => {
                    if let Some(ref key) = literal_key {
                        if let Some(prop) = properties.get(key) {
                            return prop.ty.clone();
                        }
                        if !is_open {
                            let key_str = match key {
                                mir_types::atomic::ArrayKey::String(s) => s.to_string(),
                                mir_types::atomic::ArrayKey::Int(i) => i.to_string(),
                            };
                            self.emit(
                                IssueKind::NonExistentArrayOffset { key: key_str },
                                Severity::Error,
                                idx_span,
                            );
                            return Type::mixed();
                        }
                    }
                    let mut result = Type::empty();
                    for prop in properties.values() {
                        result.merge_with(&prop.ty);
                    }
                    return if result.types.is_empty() {
                        Type::mixed()
                    } else {
                        result
                    };
                }
                Atomic::TArray { value, .. } | Atomic::TNonEmptyArray { value, .. } => {
                    return *value.clone();
                }
                Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
                    return *value.clone();
                }
                Atomic::TString | Atomic::TLiteralString(_) => {
                    return Type::single(Atomic::TString);
                }
                _ => {}
            }
        }
        Type::mixed()
    }
}
