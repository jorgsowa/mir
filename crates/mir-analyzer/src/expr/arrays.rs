use super::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Type};
use php_ast::owned::{ArrayAccessExpr, ArrayElement, Expr, ExprKind};
use std::sync::Arc;

/// For a spread (`...`) element in an array literal, return the union of key types
/// across all array atomics. Mirrors [`crate::call::spread_element_type`], which does
/// the same for value types. E.g. `array<string, int>` → `string`, `list<int>` → `int`.
fn spread_key_type(arr_ty: &Type) -> Type {
    use mir_types::atomic::ArrayKey;

    let mut result = Type::empty();
    for atomic in arr_ty.types.iter() {
        match atomic {
            Atomic::TArray { key, .. } | Atomic::TNonEmptyArray { key, .. } => {
                for t in key.types.iter() {
                    result.add_type(t.clone());
                }
            }
            Atomic::TList { .. } | Atomic::TNonEmptyList { .. } => {
                result.add_type(Atomic::TInt);
            }
            Atomic::TKeyedArray { properties, .. } => {
                for key in properties.keys() {
                    match key {
                        ArrayKey::Int(_) => result.add_type(Atomic::TInt),
                        ArrayKey::String(s) => result.add_type(Atomic::TLiteralString(s.clone())),
                    }
                }
            }
            // Traversable<K, V>, Iterator<K, V>, Generator<K, V, ...> — key is param[0].
            Atomic::TNamedObject { type_params, .. } if type_params.len() >= 2 => {
                for t in type_params[0].types.iter() {
                    result.add_type(t.clone());
                }
            }
            _ => return Type::mixed(),
        }
    }
    if result.types.is_empty() {
        Type::mixed()
    } else {
        result
    }
}

/// Whether `fqcn` implements `ArrayAccess`, directly or via an ancestor
/// class/interface — `$obj[$idx]` on such a receiver is governed by the
/// class's own `offsetGet`/`offsetSet`/`offsetExists` signatures, not the
/// plain-PHP-array "offset must be an array-key" rule.
fn implements_array_access(db: &dyn crate::db::MirDatabase, fqcn: &mir_types::Name) -> bool {
    let bare = fqcn.as_ref().trim_start_matches('\\');
    crate::db::class_ancestors_by_fqcn(db, crate::db::Fqcn::from_str(db, bare))
        .iter()
        .any(|a| {
            a.trim_start_matches('\\')
                .eq_ignore_ascii_case("ArrayAccess")
        })
}

/// Resolve the value type `$obj[$idx]` yields for an `ArrayAccess`-implementing
/// receiver: prefer an explicit `@implements ArrayAccess<TKey, TValue>`
/// annotation (substituting the receiver's own concrete type args), falling
/// back to `offsetGet()`'s resolved return type. `None` means `fqcn` doesn't
/// implement `ArrayAccess` at all.
fn resolve_array_access_value_type(
    db: &dyn crate::db::MirDatabase,
    fqcn: &mir_types::Name,
    type_params: &[Type],
) -> Option<Type> {
    if !implements_array_access(db, fqcn) {
        return None;
    }
    let bare = fqcn.as_ref().trim_start_matches('\\');
    let class = crate::db::find_class_like(db, crate::db::Fqcn::from_str(db, bare))?;
    let class_tps = crate::db::class_template_params(db, bare).unwrap_or_default();
    let bindings = crate::generic::build_class_bindings(&class_tps, type_params);

    let annotated = class
        .implements_type_args()
        .iter()
        .find_map(|(iface, args)| {
            (iface
                .trim_start_matches('\\')
                .eq_ignore_ascii_case("ArrayAccess"))
            .then_some(args)
        });
    if let Some(args) = annotated {
        if args.len() >= 2 {
            return Some(args[1].substitute_templates(&bindings));
        }
    }

    let (_, def) =
        crate::db::find_method_in_chain(db, crate::db::Fqcn::from_str(db, bare), "offsetget")?;
    let ty = def
        .return_type
        .as_deref()
        .cloned()
        .unwrap_or_else(Type::mixed);
    Some(ty.substitute_templates(&bindings))
}

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
                // Float keys are silently truncated to int in PHP; TIntegralFloat is always
                // whole-valued so the truncation is lossless — suppress the warning for it.
                if key_ty.contains(|t| matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..)))
                    && !key_ty.contains(|t| matches!(t, Atomic::TIntegralFloat))
                {
                    self.emit(
                        IssueKind::ImplicitFloatToIntCast {
                            from: key_ty.to_string(),
                        },
                        Severity::Warning,
                        key_expr.span,
                    );
                }
                match key_ty.types.as_slice() {
                    // PHP canonicalizes a numeric string key ("0", "42", ...)
                    // to an int key at runtime — without this, `['0' => 'x']`
                    // and `$arr[0]` would be treated as different slots.
                    [Atomic::TLiteralString(s)] => match super::helpers::canonical_int_array_key(s)
                    {
                        Some(i) => {
                            next_int_key = i + 1;
                            ArrayKey::Int(i)
                        }
                        None => ArrayKey::String(s.clone()),
                    },
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
            // A repeated key silently overwrites the earlier entry at runtime
            // (`['a' => 1, 'b' => 2, 'a' => 3]` evaluates to `['a' => 3, 'b' =>
            // 2]`) — almost always a copy-paste mistake, not intentional.
            if keyed_props.contains_key(&array_key) {
                let key_str = match &array_key {
                    ArrayKey::String(s) => format!("'{s}'"),
                    ArrayKey::Int(i) => i.to_string(),
                };
                self.emit(
                    IssueKind::DuplicateArrayKey { key: key_str },
                    Severity::Warning,
                    elem.key.as_ref().map_or(elem.value.span, |k| k.span),
                );
            }
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
        for elem in elements.iter() {
            let value_ty = self.analyze(&elem.value, ctx);
            if elem.unpack {
                // Merge the spread source's own key/value types instead of
                // giving up on the whole literal — `[...$x, ...$y]` should
                // type as the union of $x's and $y's key/value types, not
                // unconditionally collapse to `array<mixed, mixed>`.
                all_value_types.merge_with(&crate::call::spread_element_type(&value_ty));
                key_union.merge_with(&spread_key_type(&value_ty));
            } else {
                all_value_types.merge_with(&value_ty);
                if let Some(key_expr) = &elem.key {
                    let key_ty = self.analyze(key_expr, ctx);
                    // Float keys are silently truncated to int in PHP; TIntegralFloat is
                    // always whole-valued so the truncation is lossless — no warning.
                    if key_ty.contains(|t| matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..)))
                        && !key_ty.contains(|t| matches!(t, Atomic::TIntegralFloat))
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
        // Purity check: `$GLOBALS['x']` reaches the same external mutable
        // state as `global $x;`, but only the `global` statement was ever
        // checked — accessing the superglobal array directly inside a
        // @pure function bypassed the check entirely.
        if ctx.is_in_pure_fn {
            if let ExprKind::Variable(name) = &aa.array.kind {
                if name.trim_start_matches('$') == "GLOBALS" {
                    let variable = aa
                        .index
                        .as_ref()
                        .and_then(|idx| super::helpers::extract_string_from_expr(idx))
                        .unwrap_or_else(|| "GLOBALS".to_string());
                    self.emit(
                        IssueKind::ImpureGlobalVariable { variable },
                        Severity::Warning,
                        expr.span,
                    );
                }
            }
        }
        let arr_ty = self.analyze(&aa.array, ctx);
        // `ArrayAccess` receivers (WeakMap, SplObjectStorage, user collections)
        // define their own offset type via offsetGet/offsetSet/offsetExists —
        // the plain-PHP-array "offset must be an array-key" rule below doesn't
        // apply to them (e.g. WeakMap is keyed by `object`).
        let receiver_is_array_access = arr_ty.types.iter().any(
            |a| matches!(a, Atomic::TNamedObject { fqcn, .. } if implements_array_access(self.db, fqcn)),
        );
        if let Some(idx) = &aa.index {
            let idx_ty = self.analyze(idx, ctx);
            if receiver_is_array_access {
                // The array-key rule below is plain-PHP-array-only; skip it.
            } else if idx_ty.contains(|t| matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..))) {
                // Float keys are silently truncated to int in PHP
                self.emit(
                    IssueKind::ImplicitFloatToIntCast {
                        from: idx_ty.to_string(),
                    },
                    Severity::Warning,
                    idx.span,
                );
            } else if idx_ty.is_mixed() {
                self.emit(IssueKind::MixedArrayOffset, Severity::Info, idx.span);
            } else if !idx_ty.types.is_empty()
                && idx_ty.types.iter().all(|a| {
                    matches!(
                        a,
                        Atomic::TNamedObject { .. }
                            | Atomic::TObject
                            | Atomic::TArray { .. }
                            | Atomic::TList { .. }
                            | Atomic::TKeyedArray { .. }
                            | Atomic::TNonEmptyArray { .. }
                            | Atomic::TNonEmptyList { .. }
                            | Atomic::TClosure { .. }
                    )
                })
            {
                self.emit(
                    IssueKind::InvalidArrayOffset {
                        expected: "array-key".to_string(),
                        actual: idx_ty.to_string(),
                    },
                    Severity::Error,
                    idx.span,
                );
            }
        }

        if arr_ty.is_mixed() {
            self.emit(IssueKind::MixedArrayAccess, Severity::Info, expr.span);
            return Type::mixed();
        }

        // InvalidArrayAccess: definitely non-subscriptable type (not array, not string, not object)
        if !arr_ty.is_mixed()
            && !arr_ty.types.is_empty()
            && arr_ty.types.iter().all(|a| {
                matches!(
                    a,
                    Atomic::TInt
                        | Atomic::TLiteralInt(_)
                        | Atomic::TIntRange { .. }
                        | Atomic::TPositiveInt
                        | Atomic::TFloat
                        | Atomic::TIntegralFloat
                        | Atomic::TLiteralFloat(_, _)
                        | Atomic::TBool
                        | Atomic::TTrue
                        | Atomic::TFalse
                )
            })
        {
            self.emit(
                IssueKind::InvalidArrayAccess {
                    ty: arr_ty.to_string(),
                },
                Severity::Error,
                expr.span,
            );
            return Type::mixed();
        }

        // PossiblyInvalidArrayAccess: union has some subscriptable members and some that aren't.
        let is_invalid_for_access = |a: &Atomic| {
            matches!(
                a,
                Atomic::TInt
                    | Atomic::TLiteralInt(_)
                    | Atomic::TIntRange { .. }
                    | Atomic::TPositiveInt
                    | Atomic::TFloat
                    | Atomic::TLiteralFloat(_, _)
                    | Atomic::TBool
                    | Atomic::TTrue
                    | Atomic::TFalse
            )
        };
        if !arr_ty.is_mixed()
            && !arr_ty.types.is_empty()
            && !self.in_existence_check
            && arr_ty.types.iter().any(is_invalid_for_access)
            && !arr_ty.types.iter().all(is_invalid_for_access)
        {
            self.emit(
                IssueKind::PossiblyInvalidArrayAccess {
                    ty: arr_ty.to_string(),
                },
                Severity::Info,
                expr.span,
            );
        }

        if arr_ty.contains(|t| matches!(t, Atomic::TNull)) && arr_ty.is_single() {
            self.emit(IssueKind::NullArrayAccess, Severity::Error, expr.span);
            return Type::mixed();
        }
        if arr_ty.is_nullable() && !self.in_existence_check {
            self.emit(
                IssueKind::PossiblyNullArrayAccess,
                Severity::Info,
                expr.span,
            );
        }

        let literal_key: Option<mir_types::atomic::ArrayKey> =
            aa.index.as_ref().and_then(|idx| match &idx.kind {
                ExprKind::String(s) => Some(match super::helpers::canonical_int_array_key(s) {
                    Some(i) => mir_types::atomic::ArrayKey::Int(i),
                    None => mir_types::atomic::ArrayKey::String(Arc::from(s.as_ref())),
                }),
                ExprKind::Int(i) => Some(mir_types::atomic::ArrayKey::Int(*i)),
                _ => None,
            });

        let idx_span = aa.index.as_ref().map(|i| i.span).unwrap_or(expr.span);

        // When every atomic in the union is a shape and the index is a
        // literal key, merge the key's type across every union member
        // instead of returning as soon as the first shape happens to match —
        // `array{a: int}|array{a: string}` accessed via `$x['a']` must yield
        // `int|string`, not just the first arm's `int`.
        if let Some(ref key) = literal_key {
            if !arr_ty.types.is_empty()
                && arr_ty
                    .types
                    .iter()
                    .all(|a| matches!(a, Atomic::TKeyedArray { .. }))
            {
                let mut result = Type::empty();
                for atomic in &arr_ty.types {
                    let Atomic::TKeyedArray {
                        properties,
                        is_open,
                        ..
                    } = atomic
                    else {
                        unreachable!("filtered to TKeyedArray above")
                    };
                    if let Some(prop) = properties.get(key) {
                        // An optional key (`array{b?: string}`) may be absent at
                        // runtime — accessing it can yield null via the array's
                        // undefined-offset warning-then-null semantics, so the
                        // result must include null, not just the declared value type.
                        if prop.optional {
                            let mut widened = prop.ty.clone();
                            widened.add_type(Atomic::TNull);
                            result.merge_with(&widened);
                        } else {
                            result.merge_with(&prop.ty);
                        }
                    } else if !is_open && !self.in_existence_check {
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
                    } else {
                        for prop in properties.values() {
                            result.merge_with(&prop.ty);
                        }
                    }
                }
                return if result.types.is_empty() {
                    Type::mixed()
                } else {
                    result
                };
            }
        }

        for atomic in &arr_ty.types {
            match atomic {
                Atomic::TKeyedArray {
                    properties,
                    is_open,
                    ..
                } => {
                    if let Some(ref key) = literal_key {
                        if let Some(prop) = properties.get(key) {
                            if prop.optional {
                                let mut widened = prop.ty.clone();
                                widened.add_type(Atomic::TNull);
                                return widened;
                            }
                            return prop.ty.clone();
                        }
                        if !is_open && !self.in_existence_check {
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
                Atomic::TNamedObject { fqcn, type_params } => {
                    if let Some(value_ty) =
                        resolve_array_access_value_type(self.db, fqcn, type_params)
                    {
                        return value_ty;
                    }
                }
                _ => {}
            }
        }
        Type::mixed()
    }
}
