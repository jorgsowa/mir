use super::*;
use rustc_hash::FxHashMap;
use std::cell::RefCell;

/// Parse an assertion annotation's type, recognizing the leading `!` negation
/// marker (`@psalm-assert !null $x` — asserts `$x` is NOT this type) that only
/// assertion tags use, never ordinary `@param`/`@return` type positions.
pub(crate) fn parse_assertion_type(s: &str) -> (Type, bool) {
    match s.trim().strip_prefix('!') {
        Some(rest) => (parse_type_string(rest), true),
        None => (parse_type_string(s), false),
    }
}

pub(crate) fn parse_type_string(s: &str) -> Type {
    let s = s.trim();

    // Nullable shorthand: `?Type`
    if let Some(inner) = s.strip_prefix('?') {
        let inner_ty = parse_type_string(inner);
        let mut u = inner_ty;
        u.add_type(Atomic::TNull);
        return u;
    }

    // Conditional type: `($param is TypeName ? TrueType : FalseType)`
    // Parenthesized type: `(A&B)|null` — strip outer parens and recurse.
    if s.starts_with('(') && s.ends_with(')') {
        let inner = s[1..s.len() - 1].trim();
        if let Some(conditional) = parse_conditional_type(inner) {
            return conditional;
        }
        // Strip balanced outer parens: verify depth doesn't go negative before the end.
        if is_balanced_parens(s) {
            return parse_type_string(inner);
        }
    }

    // Type: `A|B|C`
    if s.contains('|') && !is_inside_generics(s) {
        let parts = split_union(s);
        if parts.len() > 1 {
            let mut u = Type::empty();
            for part in parts {
                for atomic in parse_type_string(&part).types {
                    u.add_type(atomic);
                }
            }
            return u;
        }
    }

    // Intersection: `A&B&C` — PHP 8.1+ pure intersection type.
    // Use a depth-aware split so `&` inside generics (e.g. `array<K,V>`) is not broken.
    if s.contains('&') && !is_inside_generics(s) {
        let parts = split_intersection(s);
        if parts.len() > 1 {
            let parts: Vec<Type> = parts.iter().map(|p| parse_type_string(p.trim())).collect();
            return Type::single(Atomic::TIntersection {
                parts: mir_types::union::vec_to_type_params(parts),
            });
        }
    }

    // Array shorthand: `Type[]` or `Type[][]`
    if let Some(value_str) = s.strip_suffix("[]") {
        let value = parse_type_string(value_str);
        return Type::single(Atomic::TArray {
            key: Box::new(Type::single(Atomic::TInt)),
            value: Box::new(value),
        });
    }

    // Callable/closure syntax: `Closure(T): R` or `callable(T): R`
    if let Some(call_ty) = parse_callable_syntax(s) {
        return call_ty;
    }

    // Array shape: `array{key: Type, ...}` or `list{Type, ...}`
    if s.ends_with('}') {
        if let Some(open) = s.find('{') {
            let prefix = s[..open].to_lowercase();
            let inner = &s[open + 1..s.len() - 1];
            if prefix == "array" {
                return parse_keyed_array(inner, false);
            } else if prefix == "list" {
                return parse_keyed_array(inner, true);
            } else if prefix == "object" {
                // `object{prop: Type, ...}` — mir has no object-shape atom,
                // so approximate as a plain `object` (property shape is lost).
                return Type::single(Atomic::TObject);
            }
        }
    }

    // Generic: `name<...>`
    if let Some(open) = s.find('<') {
        if s.ends_with('>') {
            let name = &s[..open];
            let inner = &s[open + 1..s.len() - 1];
            return parse_generic(name, inner);
        }
    }

    // Float literal: `3.14`, `-0.5`. Must run before the keyword/named-class
    // arms (which only parse integer literals) so `@return 3.14` is not misread
    // as a class named "3.14". Requires a decimal point so plain ints and class
    // names (which never contain `.`) are left untouched.
    if let Some(f) = parse_float_literal(s) {
        let bits = f.to_bits();
        return Type::single(Atomic::TLiteralFloat(
            (bits >> 32) as i64,
            (bits & 0xFFFF_FFFF) as i64,
        ));
    }

    // Keywords
    match s.to_lowercase().as_str() {
        "string" => Type::single(Atomic::TString),
        "non-empty-string" => Type::single(Atomic::TNonEmptyString),
        "numeric-string" => Type::single(Atomic::TNumericString),
        // Psalm string refinements. mir does not model case/falsiness of strings
        // precisely; approximate with the closest representable atom (same
        // approach Psalm documents for back-compat). `truthy-string` /
        // `non-falsy-string` ≈ non-empty-string; case-constrained strings ≈ string.
        "truthy-string" | "non-falsy-string" => Type::single(Atomic::TNonEmptyString),
        "lowercase-string"
        | "uppercase-string"
        | "non-empty-lowercase-string"
        | "non-empty-uppercase-string" => Type::single(Atomic::TString),
        "class-string" => Type::single(Atomic::TClassString(None)),
        "interface-string" => Type::single(Atomic::TInterfaceString(None)),
        "int" | "integer" => Type::single(Atomic::TInt),
        "positive-int" => Type::single(Atomic::TPositiveInt),
        "negative-int" => Type::single(Atomic::TNegativeInt),
        "non-negative-int" => Type::single(Atomic::TNonNegativeInt),
        "float" | "double" => Type::single(Atomic::TFloat),
        "bool" | "boolean" => Type::single(Atomic::TBool),
        "true" => Type::single(Atomic::TTrue),
        "false" => Type::single(Atomic::TFalse),
        "null" => Type::single(Atomic::TNull),
        "void" => Type::single(Atomic::TVoid),
        "never" | "never-return" | "no-return" | "never-returns" => Type::single(Atomic::TNever),
        "mixed" => Type::single(Atomic::TMixed),
        "object" => Type::single(Atomic::TObject),
        "array" => Type::single(Atomic::TArray {
            key: Box::new(Type::single(Atomic::TMixed)),
            value: Box::new(Type::mixed()),
        }),
        "list" => Type::single(Atomic::TList {
            value: Box::new(Type::mixed()),
        }),
        // Bare `pure-callable`/`pure-Closure` with no `(...)` signature — the
        // parenthesized form is handled above by `parse_callable_syntax`
        // (which already strips the purity qualifier), but a bare keyword
        // with no signature never reaches it and would otherwise fall through
        // to the named-class catch-all below as a bogus class literally named
        // "pure-callable". Purity is tracked separately at the function level.
        "callable" | "pure-callable" => Type::single(Atomic::TCallable {
            params: None,
            return_type: None,
        }),
        // Bare `Closure` isn't listed here — it's a real PHP class, so it
        // already resolves correctly via the named-class fallthrough below.
        // `pure-closure` isn't a real class name, so it needs the same
        // explicit mapping.
        "pure-closure" => Type::single(Atomic::TNamedObject {
            fqcn: mir_types::Name::from("Closure"),
            type_params: Default::default(),
        }),
        "callable-string" => Type::single(Atomic::TCallableString),
        "iterable" => {
            let mut u = Type::single(Atomic::TArray {
                key: Box::new(Type::single(Atomic::TMixed)),
                value: Box::new(Type::mixed()),
            });
            u.add_type(Atomic::TNamedObject {
                fqcn: mir_types::Name::from("Traversable"),
                type_params: Default::default(),
            });
            u
        }
        "scalar" => Type::single(Atomic::TScalar),
        "numeric" => Type::single(Atomic::TNumeric),
        // `empty` — Psalm's falsy pseudo-type. mir has no single "falsy" atom,
        // so approximate as the union of all falsy literals it can express.
        "empty" => {
            let mut u = Type::single(Atomic::TFalse);
            u.add_type(Atomic::TNull);
            u.add_type(Atomic::TLiteralInt(0));
            u.add_type(Atomic::TLiteralFloat(0, 0));
            u.add_type(Atomic::TLiteralString(Arc::from("")));
            u.add_type(Atomic::TLiteralString(Arc::from("0")));
            u.add_type(Atomic::TKeyedArray {
                properties: Box::default(),
                is_open: false,
                is_list: true,
            });
            u
        }
        "array-key" => {
            let mut u = Type::single(Atomic::TInt);
            u.add_type(Atomic::TString);
            u
        }
        "resource" => Type::mixed(), // treat as mixed
        // self/static/parent: emit sentinel with empty FQCN; collector fills it in.
        "static" => Type::single(Atomic::TStaticObject {
            fqcn: mir_types::Name::from(""),
        }),
        "self" | "$this" => Type::single(Atomic::TSelf {
            fqcn: mir_types::Name::from(""),
        }),
        "parent" => Type::single(Atomic::TParent {
            fqcn: mir_types::Name::from(""),
        }),

        // Named class
        _ if !s.is_empty()
            && s.chars()
                .next()
                .map(|c| c.is_alphanumeric() || c == '\\' || c == '_')
                .unwrap_or(false) =>
        {
            // Integer literal: `1`, `-42`, `0` etc.
            if let Ok(n) = s.parse::<i64>() {
                return Type::single(Atomic::TLiteralInt(n));
            }
            Type::single(Atomic::TNamedObject {
                fqcn: normalize_fqcn(s).into(),
                type_params: mir_types::union::empty_type_params(),
            })
        }

        // Negative integer literal: `-1`, `-42` — starts with `-`, not caught by alphanumeric check
        _ if s.starts_with('-') && s.len() > 1 && s[1..].chars().all(|c| c.is_ascii_digit()) => {
            if let Ok(n) = s.parse::<i64>() {
                Type::single(Atomic::TLiteralInt(n))
            } else {
                Type::mixed()
            }
        }

        // String literal: `'foo'` or `"bar"`
        _ if (s.starts_with('\'') && s.ends_with('\''))
            || (s.starts_with('"') && s.ends_with('"')) =>
        {
            let inner = &s[1..s.len() - 1];
            Type::single(Atomic::TLiteralString(Arc::from(inner)))
        }

        _ => Type::mixed(),
    }
}

pub(super) fn parse_generic(name: &str, inner: &str) -> Type {
    match name.to_lowercase().as_str() {
        "array" => {
            let params = split_generics(inner);
            let array_key = || {
                let mut k = Type::single(Atomic::TInt);
                k.add_type(Atomic::TString);
                k
            };
            let (key, value) = match params.len() {
                n if n >= 2 => (
                    parse_type_string(params[0].trim()),
                    parse_type_string(params[1].trim()),
                ),
                1 => (array_key(), parse_type_string(params[0].trim())),
                _ => (array_key(), Type::mixed()),
            };
            Type::single(Atomic::TArray {
                key: Box::new(key),
                value: Box::new(value),
            })
        }
        "list" | "non-empty-list" => {
            let value = parse_type_string(inner.trim());
            if name.to_lowercase().starts_with("non-empty") {
                Type::single(Atomic::TNonEmptyList {
                    value: Box::new(value),
                })
            } else {
                Type::single(Atomic::TList {
                    value: Box::new(value),
                })
            }
        }
        "non-empty-array" => {
            let params = split_generics(inner);
            let array_key = || {
                let mut k = Type::single(Atomic::TInt);
                k.add_type(Atomic::TString);
                k
            };
            let (key, value) = match params.len() {
                n if n >= 2 => (
                    parse_type_string(params[0].trim()),
                    parse_type_string(params[1].trim()),
                ),
                1 => (array_key(), parse_type_string(params[0].trim())),
                _ => (array_key(), Type::mixed()),
            };
            Type::single(Atomic::TNonEmptyArray {
                key: Box::new(key),
                value: Box::new(value),
            })
        }
        "iterable" => {
            let params = split_generics(inner);
            let (key, value) = match params.len() {
                n if n >= 2 => (
                    parse_type_string(params[0].trim()),
                    parse_type_string(params[1].trim()),
                ),
                1 => (
                    Type::single(Atomic::TMixed),
                    parse_type_string(params[0].trim()),
                ),
                _ => (Type::single(Atomic::TMixed), Type::mixed()),
            };
            let mut u = Type::single(Atomic::TArray {
                key: Box::new(key.clone()),
                value: Box::new(value.clone()),
            });
            u.add_type(Atomic::TNamedObject {
                fqcn: mir_types::Name::from("Traversable"),
                type_params: mir_types::union::vec_to_type_params(vec![key, value]),
            });
            u
        }
        "class-string" => Type::single(Atomic::TClassString(Some(
            normalize_fqcn(inner.trim()).into(),
        ))),
        "interface-string" => Type::single(Atomic::TInterfaceString(Some(
            normalize_fqcn(inner.trim()).into(),
        ))),
        "int" => {
            // int<min, max> — `min`/`max` keywords (or a missing/garbled bound)
            // mean "unbounded on that side"; a numeric literal is an inclusive
            // bound. e.g. `int<0, max>` → min 0, no upper bound.
            let parse_bound = |s: &str| -> Option<i64> {
                match s.trim() {
                    "min" | "max" => None,
                    n => n.parse::<i64>().ok(),
                }
            };
            let bounds = split_generics(inner);
            let (min, max) = match bounds.as_slice() {
                [lo, hi] => (parse_bound(lo), parse_bound(hi)),
                _ => (None, None),
            };
            Type::single(Atomic::TIntRange { min, max })
        }
        // `key-of<T>` — the union of array/shape key types of `T`.
        "key-of" => {
            let inner_ty = parse_type_string(inner.trim());
            eval_key_of(&inner_ty).unwrap_or_else(Type::mixed)
        }
        // `value-of<T>` — the union of array/shape value types of `T`.
        "value-of" => {
            let inner_ty = parse_type_string(inner.trim());
            eval_value_of(&inner_ty).unwrap_or_else(Type::mixed)
        }
        // `int-mask<V1, V2, ...>` — when all members are non-negative integer
        // literals and the set is small (≤ 8), expand to the union of all
        // possible OR-combinations (including 0 for "no flags set"). This lets
        // the call checker reject out-of-range literals statically. Falls back
        // to `int` when members include class constants or the set is too large.
        "int-mask" => {
            let parts = split_generics(inner);
            let members: Vec<i64> = parts
                .iter()
                .filter_map(|s| s.trim().parse::<i64>().ok())
                .collect();
            if !members.is_empty() && members.len() == parts.len() {
                expand_int_mask_members(&members).unwrap_or_else(|| Type::single(Atomic::TInt))
            } else {
                Type::single(Atomic::TInt)
            }
        }
        // `int-mask-of<T::CONST_*>` — resolves against the constants of the
        // *currently-collected* class when `T` is `self`/`static`/the class's
        // own name (see `SelfIntConstantsGuard`); any other class reference
        // needs cross-file lookup that isn't available here, so it falls back
        // to `int`.
        "int-mask-of" => resolve_int_mask_of(inner).unwrap_or_else(|| Type::single(Atomic::TInt)),
        // `class-string-map<T, V>` — maps `class-string<T>` keys to `V` values.
        // mir does not tie the value type to the specific class looked up (that
        // needs flow-sensitive template binding at each access site), so
        // approximate as a plain array from `class-string` to `V`.
        "class-string-map" => {
            let params = split_generics(inner);
            // The two-arg form (`class-string-map<T, V>`) is canonical, but
            // Psalm also accepts the one-arg shorthand (`class-string-map<T>`)
            // where the value type defaults to `T` itself, not `mixed`.
            let value = params
                .get(1)
                .or(params.first())
                .map(|p| parse_type_string(p.trim()))
                .unwrap_or_else(Type::mixed);
            Type::single(Atomic::TArray {
                key: Box::new(Type::single(Atomic::TClassString(None))),
                value: Box::new(value),
            })
        }
        // Named class with type params
        _ => {
            let params: Vec<Type> = split_generics(inner)
                .iter()
                .map(|p| parse_type_string(p.trim()))
                .collect();
            Type::single(Atomic::TNamedObject {
                fqcn: normalize_fqcn(name).into(),
                type_params: mir_types::union::vec_to_type_params(params),
            })
        }
    }
}

/// Parse a floating-point literal type such as `3.14` or `-0.5`.
///
/// Returns `None` for anything that is not unambiguously a float: the string
/// must contain a decimal point and parse as `f64`. Plain integers (`42`) and
/// class names are rejected so they fall through to their normal handling.
pub(super) fn parse_float_literal(s: &str) -> Option<f64> {
    if !s.contains('.') {
        return None;
    }
    // Reject anything with stray characters (e.g. `1.2.3`, `Foo.Bar`).
    let body = s.strip_prefix('-').unwrap_or(s);
    if !body
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-')
    {
        return None;
    }
    s.parse::<f64>().ok()
}

/// Evaluate `key-of<T>`: collect the key types of every array/shape atom in `T`.
///
/// Returns `None` if any atom's keys cannot be determined statically (e.g. a
/// template parameter or a named class), so the caller can fall back to `mixed`.
pub(super) fn eval_key_of(t: &Type) -> Option<Type> {
    let mut result = Type::empty();
    for atomic in &t.types {
        match atomic {
            Atomic::TArray { key, .. } | Atomic::TNonEmptyArray { key, .. } => {
                for k in &key.types {
                    result.add_type(k.clone());
                }
            }
            Atomic::TList { .. } | Atomic::TNonEmptyList { .. } => {
                result.add_type(Atomic::TInt);
            }
            Atomic::TKeyedArray { properties, .. } => {
                for key in properties.keys() {
                    match key {
                        mir_types::atomic::ArrayKey::Int(n) => {
                            result.add_type(Atomic::TLiteralInt(*n));
                        }
                        mir_types::atomic::ArrayKey::String(s) => {
                            result.add_type(Atomic::TLiteralString(s.clone()));
                        }
                    }
                }
            }
            _ => return None,
        }
    }
    (!result.types.is_empty()).then_some(result)
}

/// Evaluate `value-of<T>`: collect the value types of every array/shape atom in
/// `T`. Returns `None` if any atom's values cannot be determined statically.
pub(super) fn eval_value_of(t: &Type) -> Option<Type> {
    let mut result = Type::empty();
    for atomic in &t.types {
        match atomic {
            Atomic::TArray { value, .. }
            | Atomic::TNonEmptyArray { value, .. }
            | Atomic::TList { value }
            | Atomic::TNonEmptyList { value } => {
                for v in &value.types {
                    result.add_type(v.clone());
                }
            }
            Atomic::TKeyedArray { properties, .. } => {
                for prop in properties.values() {
                    for v in &prop.ty.types {
                        result.add_type(v.clone());
                    }
                }
            }
            _ => return None,
        }
    }
    (!result.types.is_empty()).then_some(result)
}

/// Expand a small set of non-negative int-literal flags into the union of
/// every OR-combination (including 0 for "no flags set"). Shared by
/// `int-mask<...>` (literal members) and `int-mask-of<...>` (members read
/// from resolved class constants). Returns `None` when the set is empty,
/// contains a negative value, or is too large to enumerate (> 8 members).
pub(super) fn expand_int_mask_members(members: &[i64]) -> Option<Type> {
    if members.is_empty() || members.len() > 8 || members.iter().any(|&v| v < 0) {
        return None;
    }
    let mut values = std::collections::BTreeSet::new();
    for subset in 0u32..(1u32 << members.len()) {
        let value = members
            .iter()
            .enumerate()
            .filter(|(i, _)| subset & (1 << i) != 0)
            .fold(0i64, |acc, (_, &v)| acc | v);
        values.insert(value);
    }
    let mut u = Type::empty();
    for v in values {
        u.add_type(Atomic::TLiteralInt(v));
    }
    Some(u)
}

/// `(declaring class FQCN, its own literal-int constants)`.
type SelfIntConstants = (Arc<str>, Arc<FxHashMap<Arc<str>, i64>>);

thread_local! {
    /// Ambient constants of the class currently being collected, so
    /// `int-mask-of<self::*>` can resolve without threading a context
    /// parameter through every recursive `parse_type_string` call. Set for
    /// the duration of one class's member loop by `SelfIntConstantsGuard`;
    /// cleared (restored) on drop.
    static SELF_INT_CONSTANTS: RefCell<Option<SelfIntConstants>> = const {
        RefCell::new(None)
    };
}

/// RAII guard that makes a class's literal-int constants available to
/// `int-mask-of<...>` parsing for as long as it is alive. Only classes/traits
/// resolve `self`/`static`/own-name references this way; any other class
/// reference (needing cross-file lookup) keeps the `int` fallback.
pub(crate) struct SelfIntConstantsGuard {
    previous: Option<SelfIntConstants>,
}

impl SelfIntConstantsGuard {
    pub(crate) fn activate(fqcn: &str, constants: &Arc<FxHashMap<Arc<str>, i64>>) -> Self {
        let previous = SELF_INT_CONSTANTS
            .with(|cell| cell.replace(Some((Arc::from(fqcn), constants.clone()))));
        Self { previous }
    }
}

impl Drop for SelfIntConstantsGuard {
    fn drop(&mut self) {
        SELF_INT_CONSTANTS.with(|cell| *cell.borrow_mut() = self.previous.take());
    }
}

/// Resolve `int-mask-of<T::CONST_PREFIX*>` (or bare `T::*`) using the ambient
/// class constants set by `SelfIntConstantsGuard`. Returns `None` when there
/// is no active guard, `T` isn't a self-reference, no constants match the
/// prefix, or the match set can't be expanded (see `expand_int_mask_members`).
pub(super) fn resolve_int_mask_of(inner: &str) -> Option<Type> {
    let (class_ref, pattern) = inner.trim().split_once("::")?;
    let class_ref = class_ref.trim();
    let prefix = pattern.trim().strip_suffix('*')?;
    SELF_INT_CONSTANTS.with(|cell| {
        let active = cell.borrow();
        let (fqcn, constants) = active.as_ref()?;
        // A bare (non-backslash-qualified) reference to the class's own short
        // name is the common case for a namespaced class referencing itself
        // (`int-mask-of<Flags::*>` inside `namespace App; class Flags {...}`)
        // — `class_ref` here is the raw docblock text, never namespace-resolved.
        let short_name = fqcn.rsplit('\\').next().unwrap_or(fqcn);
        let is_self_ref = matches!(class_ref, "self" | "static" | "$this")
            || normalize_fqcn(class_ref).eq_ignore_ascii_case(fqcn)
            || (!class_ref.contains('\\') && class_ref.eq_ignore_ascii_case(short_name));
        if !is_self_ref {
            return None;
        }
        let mut members: Vec<i64> = constants
            .iter()
            .filter(|(name, _)| name.starts_with(prefix))
            .map(|(_, &v)| v)
            .collect();
        members.sort_unstable();
        members.dedup();
        expand_int_mask_members(&members)
    })
}

pub(super) fn strip_quotes(s: &str) -> &str {
    if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')) {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

pub(super) fn parse_keyed_array(inner: &str, is_list: bool) -> Type {
    use mir_types::atomic::KeyedProperty;
    let mut properties: IndexMap<ArrayKey, KeyedProperty> = IndexMap::new();
    let mut is_open = false;
    let mut auto_index = 0i64;

    for item in split_generics(inner) {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        if item == "..." {
            is_open = true;
            continue;
        }
        // Find a colon that is not inside nested generics/braces
        let colon_pos = {
            let mut depth = 0i32;
            let mut found = None;
            for (i, ch) in item.char_indices() {
                match ch {
                    '<' | '(' | '{' => depth += 1,
                    '>' | ')' | '}' => depth -= 1,
                    ':' if depth == 0 => {
                        found = Some(i);
                        break;
                    }
                    _ => {}
                }
            }
            found
        };
        if let Some(colon) = colon_pos {
            let key_part = item[..colon].trim();
            let ty_part = item[colon + 1..].trim();
            let optional = key_part.ends_with('?');
            let key_str = key_part.trim_end_matches('?').trim();
            let key_str = strip_quotes(key_str);
            let key = if let Ok(n) = key_str.parse::<i64>() {
                ArrayKey::Int(n)
            } else {
                ArrayKey::String(Arc::from(key_str))
            };
            properties.insert(
                key,
                KeyedProperty {
                    ty: parse_type_string(ty_part),
                    optional,
                },
            );
        } else {
            properties.insert(
                ArrayKey::Int(auto_index),
                KeyedProperty {
                    ty: parse_type_string(item),
                    optional: false,
                },
            );
            auto_index += 1;
        }
    }

    Type::single(Atomic::TKeyedArray {
        properties: Box::new(properties),
        is_open,
        is_list,
    })
}

pub(super) fn parse_callable_syntax(s: &str) -> Option<Type> {
    let s = s.trim_start_matches('\\');
    let lower = s.to_lowercase();
    // `pure-callable(...)` / `pure-Closure(...)` — mir does not track purity
    // on the type itself, so parse the structural shape and drop the
    // purity qualifier (purity is tracked separately at the function level).
    let (lower, pure_prefix_len) = match lower.strip_prefix("pure-") {
        Some(rest) => (rest.to_string(), "pure-".len()),
        None => (lower, 0),
    };
    let is_closure = lower.starts_with("closure");
    let is_callable = lower.starts_with("callable");
    if !is_closure && !is_callable {
        return None;
    }
    let prefix_len = pure_prefix_len
        + if is_closure {
            "closure".len()
        } else {
            "callable".len()
        };
    let rest = s[prefix_len..].trim_start();
    if !rest.starts_with('(') {
        return None;
    }
    let close = find_matching_paren(rest)?;
    let params_str = &rest[1..close];
    let after = rest[close + 1..].trim();
    let return_type = after
        .strip_prefix(':')
        .map(|ret_str| Box::new(parse_type_string(ret_str.trim())));
    let params: Box<[mir_types::atomic::FnParam]> = split_generics(params_str)
        .into_iter()
        .enumerate()
        .filter(|(_, p)| !p.trim().is_empty())
        .map(|(i, p)| {
            let p = p.trim();
            // `...$rest` (variadic) / trailing `=` (optional) — e.g.
            // `callable(string, int=):void` or `callable(string, ...$args):void`.
            let (p, is_variadic) = match p.strip_prefix("...") {
                Some(rest) => (rest.trim_start(), true),
                None => (p, false),
            };
            let (p, is_optional) = match p.strip_suffix('=') {
                Some(rest) => (rest.trim_end(), true),
                None => (p, false),
            };
            let (ty_str, name) = if let Some(dollar) = p.rfind('$') {
                (p[..dollar].trim(), p[dollar + 1..].to_string())
            } else {
                (p, format!("arg{i}"))
            };
            mir_types::atomic::FnParam {
                name: name.into(),
                ty: Some(mir_types::SimpleType::from_union(parse_type_string(ty_str))),
                out_ty: None,
                default: None,
                is_variadic,
                is_byref: false,
                is_optional: is_optional || is_variadic,
            }
        })
        .collect();
    if is_closure {
        Some(Type::single(Atomic::TClosure {
            data: Box::new(mir_types::atomic::ClosureData {
                params,
                return_type: return_type
                    .map_or_else(|| Type::single(Atomic::TVoid), |boxed| *boxed),
                this_type: None,
            }),
        }))
    } else {
        Some(Type::single(Atomic::TCallable {
            params: Some(params),
            return_type,
        }))
    }
}

pub(super) fn find_matching_paren(s: &str) -> Option<usize> {
    if !s.starts_with('(') {
        return None;
    }
    let mut depth = 0i32;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' | '<' | '{' => depth += 1,
            ')' | '>' | '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse template tag format: `T`, `T of Bound`, or `T as Bound`.
///
/// For single-line docblocks (e.g. `/** @template T @param T $x @return T */`)
/// `phpdoc_parser` hands us a body that runs all the way to the closing `*/`,
/// so the body may carry trailing tags (`T @param T $x @return T`). The template
/// name is the first whitespace-delimited token and any `of`/`as` bound is only
/// parsed up to the next `@` tag.
pub(super) fn parse_template_line(
    _tag_name: &str,
    body: Option<String>,
) -> Option<(String, Option<String>, Option<String>)> {
    let body = body?;
    let body = body.trim();
    // The body may also carry FOLLOWING PROSE LINES (a description written
    // after the tag) — only the first line holds the name and optional bound.
    // Otherwise a description like "Returns an array of class attributes."
    // is misparsed as a bound via its " of ".
    let body = body.lines().next().unwrap_or(body).trim_end();
    // Stop at the next embedded tag so single-line docblocks don't swallow the
    // following `@param`/`@return` tokens into the template name/bound.
    let body = match body.find(" @") {
        Some(idx) => body[..idx].trim_end(),
        None => body,
    };
    if body.is_empty() {
        return None;
    }
    // Whitespace-split rather than matching a literal " of "/" as " substring
    // — a tab-separated docblock (`@template\tT\tof\tBound`) must bind the
    // bound too, not silently fall through to the no-bound case.
    let mut tokens = body.split_whitespace().peekable();
    let name = tokens.next()?;
    let bound = if matches!(tokens.peek(), Some(&"of") | Some(&"as")) {
        tokens.next();
        let mut bound_tokens = Vec::new();
        while let Some(&t) = tokens.peek() {
            // A trailing `= Default` (e.g. `@template T of Bound = Default`)
            // belongs to the default value, not the bound.
            if t == "=" {
                break;
            }
            bound_tokens.push(t);
            tokens.next();
        }
        let bound = bound_tokens.join(" ");
        (!bound.is_empty()).then_some(bound)
    } else {
        None
    };
    // `@template T = Default` — the value used when nothing binds T.
    let default = if matches!(tokens.peek(), Some(&"=")) {
        tokens.next();
        let default: String = tokens.collect::<Vec<_>>().join(" ");
        (!default.is_empty()).then_some(default)
    } else {
        None
    };
    Some((name.to_string(), bound, default))
}

/// Extract the description text (all prose before the first `@` tag) from a raw docblock.
pub(super) fn extract_description(text: &str) -> String {
    let mut desc_lines: Vec<&str> = Vec::new();
    for line in text.lines() {
        let l = line.trim();
        let l = l.trim_start_matches("/**").trim();
        let l = l.trim_end_matches("*/").trim();
        let l = l.trim_start_matches("*/").trim();
        let l = l.strip_prefix("* ").unwrap_or(l.trim_start_matches('*'));
        let l = l.trim();
        if l.starts_with('@') {
            break;
        }
        if !l.is_empty() {
            desc_lines.push(l);
        }
    }
    desc_lines.join(" ")
}

/// Parse `@psalm-import-type` body.
///
/// Formats:
/// - `AliasName from SourceClass`
/// - `AliasName as LocalAlias from SourceClass`
pub(super) fn parse_import_type(body: &str) -> Option<DocImportType> {
    // Split on " from " (with spaces to avoid matching partial words)
    let (before_from, from_class_raw) = body.split_once(" from ")?;
    let from_class = from_class_raw.trim().trim_start_matches('\\').to_string();
    if from_class.is_empty() {
        return None;
    }
    // Check for " as " in before_from
    let (original, local) = if let Some((orig, loc)) = before_from.split_once(" as ") {
        (orig.trim().to_string(), loc.trim().to_string())
    } else {
        let name = before_from.trim().to_string();
        (name.clone(), name)
    };
    if original.is_empty() || local.is_empty() {
        return None;
    }
    Some(DocImportType {
        original,
        local,
        from_class,
    })
}

pub(super) fn parse_param_line(s: &str) -> Option<(String, String)> {
    // Formats: `Type $name`, `Type $name description`, or `Type &$name ...` (byref)
    // Types can contain $-named params inside callable syntax (`callable(int $a): void`),
    // so we track bracket depth and return the FIRST `$identifier` found at depth 0.
    // Using first-match (not last) prevents description text that contains $var references
    // from being mistaken for the parameter name. Scanning the whole body (not just its
    // first physical line) is what lets a wrapped multi-line `array{...}`/`array<...>`
    // shape still resolve to its `$name` — the depth tracking already keeps the shape's
    // interior (and any newlines inside it) from being mistaken for the boundary.
    let mut depth: i32 = 0;

    for (i, ch) in s.char_indices() {
        match ch {
            '<' | '(' | '{' => depth += 1,
            '>' | ')' | '}' => depth = (depth - 1).max(0),
            _ if ch.is_whitespace() && depth == 0 => {
                let after = s[i..].trim_start();
                // Accept `$name`, `&$name` (by-reference), and `...$name` /
                // `&...$name` (variadic) — a variadic docblock param must
                // still resolve to a name or the whole @param line is lost.
                let after_stripped = after.strip_prefix('&').unwrap_or(after);
                let after_stripped = after_stripped.strip_prefix("...").unwrap_or(after_stripped);
                if after_stripped.starts_with('$') {
                    if let Some(name_with_dollar) = after_stripped.split(char::is_whitespace).next()
                    {
                        let name = name_with_dollar.trim_start_matches('$').to_string();
                        if !name.is_empty() {
                            let type_part = s[..i].trim().to_string();
                            if !type_part.is_empty() {
                                return Some((type_part, name));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    None
}

pub(super) fn extract_return_type(s: &str) -> String {
    // Extract just the type portion from a @return tag body.
    // Format: `Type [description...]`
    // Types can contain generics <>, unions |, intersections &, but descriptions are
    // separated by whitespace after the type token.
    // Example: `bool true if var is of type string` -> `bool`
    // Example: `array<string, int> an associative array` -> `array<string, int>`
    // Example: `\Closure(): T description` -> `\Closure(): T`

    let mut depth: i32 = 0;
    let mut current_token = String::new();

    for ch in s.chars() {
        match ch {
            '<' | '(' | '{' => {
                depth += 1;
                current_token.push(ch);
            }
            '>' | ')' | '}' => {
                depth = (depth - 1).max(0);
                current_token.push(ch);
            }
            _ if ch.is_whitespace() && depth == 0 => {
                break;
            }
            _ => {
                current_token.push(ch);
            }
        }
    }

    // Callable return type syntax: `\Closure(): T` — the token ends with ':'
    // because the space between ':' and 'T' caused an early stop. Append the
    // return-type token that follows.
    if current_token.ends_with(':') {
        let offset = current_token.len();
        let rest = s[offset..].trim_start();
        if !rest.is_empty() {
            let ret_type = extract_return_type(rest);
            current_token.push_str(&ret_type);
        }
    }

    current_token.trim().to_string()
}

pub(super) fn split_union(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut in_quote: Option<char> = None;
    let mut current = String::new();
    for ch in s.chars() {
        if let Some(q) = in_quote {
            current.push(ch);
            if ch == q {
                in_quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' => {
                in_quote = Some(ch);
                current.push(ch);
            }
            '<' | '(' | '{' => {
                depth += 1;
                current.push(ch);
            }
            '>' | ')' | '}' => {
                depth -= 1;
                current.push(ch);
            }
            '|' if depth == 0 => {
                parts.push(current.trim().to_string());
                current = String::new();
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

/// Depth-aware split on `&` — does not break `&` inside `<>`, `()`, or `{}`.
pub(super) fn split_intersection(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    for ch in s.chars() {
        match ch {
            '<' | '(' | '{' => {
                depth += 1;
                current.push(ch);
            }
            '>' | ')' | '}' => {
                depth -= 1;
                current.push(ch);
            }
            '&' if depth == 0 => {
                parts.push(current.trim().to_string());
                current = String::new();
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

/// Returns true when `s` starts with `(` and ends with `)` and those two
/// characters are a matched pair (i.e. the depth never goes below 1 before
/// the final character).
pub(super) fn is_balanced_parens(s: &str) -> bool {
    if !s.starts_with('(') || !s.ends_with(')') {
        return false;
    }
    let mut depth = 0i32;
    let chars: Vec<char> = s.chars().collect();
    let last = chars.len() - 1;
    for (i, ch) in chars.iter().enumerate() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                // If depth reaches 0 before the last char, the outer parens
                // are not a single balanced pair (e.g. `(A)(B)`).
                if depth == 0 && i < last {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

pub(super) fn split_generics(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut current = String::new();
    for ch in s.chars() {
        match ch {
            '<' | '(' | '{' => {
                depth += 1;
                current.push(ch);
            }
            '>' | ')' | '}' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                parts.push(current.trim().to_string());
                current = String::new();
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

/// Return the leading type expression from `s`, stopping at top-level whitespace.
/// Spaces inside `<…>` brackets are kept so that `array<string, int>` is returned whole.
pub(super) fn extract_type_prefix(s: &str) -> &str {
    let mut depth = 0i32;
    let mut end = s.len();
    for (i, ch) in s.char_indices() {
        match ch {
            '<' | '(' | '{' => depth += 1,
            '>' | ')' | '}' => depth -= 1,
            _ if ch.is_whitespace() && depth == 0 => {
                end = i;
                break;
            }
            _ => {}
        }
    }
    &s[..end]
}

/// Whether `target` occurs anywhere in `s` outside a single- or
/// double-quoted literal — used to validate PHP type syntax (which never
/// contains a bare `@`) without misreading one embedded in a literal-string
/// type like `'admin@example.com'`.
pub(super) fn contains_unquoted(s: &str, target: char) -> bool {
    let mut in_quote: Option<char> = None;
    for ch in s.chars() {
        if let Some(q) = in_quote {
            if ch == q {
                in_quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' => in_quote = Some(ch),
            _ if ch == target => return true,
            _ => {}
        }
    }
    false
}

pub(super) fn is_inside_generics(s: &str) -> bool {
    let mut depth = 0i32;
    let mut in_quote: Option<char> = None;
    for ch in s.chars() {
        if let Some(q) = in_quote {
            if ch == q {
                in_quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' => in_quote = Some(ch),
            '<' | '(' | '{' => depth += 1,
            '>' | ')' | '}' => depth -= 1,
            _ => {}
        }
    }
    depth != 0
}

/// Parses `$param is TypeName ? TrueType : FalseType` or `T is TypeName ? TrueType : FalseType`
/// (template-type conditional, no `$`) into a `TConditional`.
pub(super) fn parse_conditional_type(s: &str) -> Option<Type> {
    // `$x is not T ? A : B` is sugar for `$x is T ? B : A` (Psalm/PHPStan both
    // support the negated form). The marker must be found at nesting depth 0 —
    // a plain substring search would also match an `is`/`is not` that belongs
    // to a nested conditional inside the true/false branch (e.g.
    // `$x is string ? int : ($x is not int ? bool : float)`), splitting the
    // outer conditional at the wrong position.
    let (is_pos, is_marker_len, negated) = find_is_marker_at_depth(s)?;
    let param_raw = s[..is_pos].trim();

    // Accept either `$identifier` (regular param) or a bare identifier (template name).
    let param_name_str: &str = if let Some(name) = param_raw.strip_prefix('$') {
        if name.is_empty() {
            return None;
        }
        name
    } else {
        // Bare template name: must be a valid identifier and the string must contain `?`
        // so we don't accidentally parse class-hierarchy expressions.
        if param_raw.is_empty()
            || !param_raw.starts_with(|c: char| c.is_alphabetic() || c == '_')
            || !param_raw.chars().all(|c| c.is_alphanumeric() || c == '_')
            || !s.contains('?')
        {
            return None;
        }
        param_raw
    };
    let param_name = Some(mir_types::Name::new(param_name_str));
    let after_is = s[is_pos + is_marker_len..].trim();
    let q_pos = find_char_at_depth(after_is, '?')?;
    let subject_str = after_is[..q_pos].trim();
    let rest = after_is[q_pos + 1..].trim();
    let colon_pos = find_char_at_depth(rest, ':')?;
    let true_str = rest[..colon_pos].trim();
    let false_str = rest[colon_pos + 1..].trim();
    let (if_true_str, if_false_str) = if negated {
        (false_str, true_str)
    } else {
        (true_str, false_str)
    };
    Some(Type::single(Atomic::TConditional {
        data: Box::new(mir_types::atomic::ConditionalData {
            param_name,
            subject: parse_type_string(subject_str),
            if_true: parse_type_string(if_true_str),
            if_false: parse_type_string(if_false_str),
        }),
    }))
}

/// Finds the leftmost ` is not ` or ` is ` marker in `s` at nesting depth 0
/// (not inside `<>`, `()`, `{}`), preferring the longer `is not` marker when
/// both start at the same position. Returns `(position, marker_len, negated)`.
fn find_is_marker_at_depth(s: &str) -> Option<(usize, usize, bool)> {
    let mut depth = 0i32;
    let bytes = s.as_bytes();
    for (i, ch) in s.char_indices() {
        match ch {
            '<' | '(' | '{' => depth += 1,
            '>' | ')' | '}' => depth -= 1,
            ' ' if depth == 0 => {
                if bytes[i..].starts_with(b" is not ") {
                    return Some((i, " is not ".len(), true));
                }
                if bytes[i..].starts_with(b" is ") {
                    return Some((i, " is ".len(), false));
                }
            }
            _ => {}
        }
    }
    None
}

/// Finds `target` in `s` at nesting depth 0 (not inside `<>`, `()`, `{}`).
pub(super) fn find_char_at_depth(s: &str, target: char) -> Option<usize> {
    let mut depth = 0i32;
    for (i, ch) in s.char_indices() {
        match ch {
            '<' | '(' | '{' => depth += 1,
            '>' | ')' | '}' => depth -= 1,
            _ if ch == target && depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}
