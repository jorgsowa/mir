use super::*;
use crate::expr::helpers::canonical_int_array_key;
use mir_types::StringRefinement;
use std::cell::RefCell;

fn associative_array_marker() -> Type {
    Type::single(Atomic::TKeyedArray {
        properties: Arc::default(),
        is_open: true,
        is_list: false,
    })
}

fn associative_array_type(key: Type, value: Type, non_empty: bool) -> Type {
    let key = Box::new(key);
    let value = Box::new(value);
    let array = if non_empty {
        Atomic::TNonEmptyArray { key, value }
    } else {
        Atomic::TArray { key, value }
    };
    Type::single(Atomic::TIntersection {
        parts: mir_types::union::vec_to_type_params(vec![
            Type::single(array),
            associative_array_marker(),
        ]),
    })
}

fn refined_string(refinement: StringRefinement, non_empty: bool) -> Type {
    Type::single(Atomic::TRefinedString {
        refinement,
        non_empty,
    })
}

/// `non-empty-string&lowercase-string` is `non-empty-lowercase-string`.
fn intersect_refined_string(parts: &[Type]) -> Option<Type> {
    let refinement = parts.iter().find_map(|part| match part.types.as_slice() {
        [Atomic::TRefinedString { refinement, .. }] => Some(*refinement),
        _ => None,
    })?;
    let non_empty_string = Type::single(Atomic::TNonEmptyString);
    let mut non_empty = false;
    for part in parts {
        match part.types.as_slice() {
            [Atomic::TRefinedString {
                refinement: other,
                non_empty: part_non_empty,
            }] if *other == refinement => non_empty |= part_non_empty,
            [Atomic::TString] => {}
            _ if part.is_subtype_structural(&non_empty_string) => non_empty = true,
            _ => return None,
        }
    }
    Some(refined_string(refinement, non_empty))
}

fn arraylike_object_type(key: Type, value: Type) -> Type {
    // The internal interfaces are written with a leading backslash so the
    // collector's namespace resolution keeps them global. `Name::from`
    // without one would be qualified against the docblock's namespace
    // (e.g. `App\Countable` in a namespaced file) and the existence check
    // would flag the keyword expansion itself as an undefined class.
    // `Countable` in particular is excluded from
    // `is_global_builtin_docblock_class` (a name users commonly redeclare
    // locally), so the backslash is the only thing that pins it global.
    Type::single(Atomic::TIntersection {
        parts: mir_types::union::vec_to_type_params(vec![
            Type::single(Atomic::TNamedObject {
                fqcn: mir_types::Name::from("\\ArrayAccess"),
                type_params: mir_types::union::vec_to_type_params(vec![key.clone(), value.clone()]),
            }),
            Type::single(Atomic::TNamedObject {
                fqcn: mir_types::Name::from("\\Countable"),
                type_params: Default::default(),
            }),
            Type::single(Atomic::TNamedObject {
                fqcn: mir_types::Name::from("\\Traversable"),
                type_params: mir_types::union::vec_to_type_params(vec![key, value]),
            }),
        ]),
    })
}

/// Docblock type keywords/pseudo-types — never class names.
///
/// The single source of truth for "docblock type keyword": consulted by
/// `parse_type_string`/`parse_generic` (keyword vs. class), `validate`
/// (template/`self`/class-string), `collector::is_php_builtin_type`, and the
/// `diagnostics::is_docblock_keyword` gate (skips namespace-qualification and
/// `class_exists` probing). Entries are native types, keywords, and hyphenated
/// Psalm pseudo-types — all reserved: a class cannot legally contain a hyphen,
/// and the bare spellings are PHP keywords.
pub(crate) const DOCBLOCK_TYPE_KEYWORDS: &[&str] = &[
    "$this",
    "__benevolent",
    "array",
    "array-key",
    "arraylike-object",
    "associative-array",
    "bool",
    "boolean",
    "callable",
    "callable-array",
    "callable-object",
    "callable-string",
    "class-string",
    "class-string-map",
    "closed-resource",
    "decimal-int-string",
    "double",
    "empty",
    "enum-string",
    "false",
    "float",
    "int",
    "int-mask",
    "int-mask-of",
    "int-range",
    "integer",
    "interface-string",
    "iterable",
    "key-of",
    "list",
    "literal-int",
    "literal-string",
    "lowercase-string",
    "mixed",
    "negative-int",
    "new",
    "never",
    "never-return",
    "never-returns",
    "no-return",
    "non-decimal-int-string",
    "non-empty-array",
    "non-empty-list",
    "non-empty-literal-string",
    "non-empty-lowercase-string",
    "non-empty-mixed",
    "non-empty-scalar",
    "non-empty-string",
    "non-empty-uppercase-string",
    "non-falsy-string",
    "non-negative-int",
    "non-null-mixed",
    "non-positive-int",
    "non-zero-int",
    "noreturn",
    "null",
    "numeric",
    "numeric-string",
    "object",
    "open-resource",
    "parent",
    "positive-int",
    "private-properties-of",
    "properties-of",
    "protected-properties-of",
    "public-properties-of",
    "pure-callable",
    "pure-closure",
    "resource",
    "scalar",
    "self",
    "static",
    "static-closure",
    "static-pure-closure",
    "string",
    "stringable-object",
    "trait-string",
    "true",
    "truthy-string",
    "unset",
    "uppercase-string",
    "value-of",
    "void",
];

/// Is `name` a docblock type keyword (see [`DOCBLOCK_TYPE_KEYWORDS`])?
/// Case-insensitive, tolerating a leading `\` (`\int`); a global class name
/// such as `\Foo` is not a keyword, so the strip cannot hide a real class.
pub(crate) fn is_docblock_type_keyword(name: &str) -> bool {
    let name = name.strip_prefix('\\').unwrap_or(name);
    // The keywords are ASCII, so an ASCII name needs no lowercased copy.
    if name.is_ascii() {
        return DOCBLOCK_TYPE_KEYWORDS
            .iter()
            .any(|k| k.eq_ignore_ascii_case(name));
    }
    let lower = name.to_lowercase();
    DOCBLOCK_TYPE_KEYWORDS.iter().any(|k| *k == lower)
}

/// Parse an assertion annotation's type, recognizing the leading `!` negation
/// marker (`@psalm-assert !null $x` — asserts `$x` is NOT this type) that only
/// assertion tags use, never ordinary `@param`/`@return` type positions.
pub(crate) fn parse_assertion_type(s: &str) -> (Type, bool) {
    let s = s.trim();
    // `=Type` (strict-equality assertion) narrows identically to `Type`.
    let s = s.strip_prefix('=').unwrap_or(s);
    match s.strip_prefix('!') {
        Some(rest) => (parse_type_string(rest), true),
        None => (parse_type_string(s), false),
    }
}

/// Split a trailing (possibly nested) literal array-key suffix off an
/// assertion target name (`arr['a']['b']` -> `("arr", [String("a"),
/// String("b")])`) — used by `@psalm-assert(-if-true/-if-false) Type
/// $arr['a']['b']` to target a specific, possibly nested key path of a
/// parameter instead of the whole parameter.
/// `parse_param_line` (shared with plain `@param` parsing, which never has
/// this suffix on a real parameter name) returns the bracket text verbatim;
/// splitting it here keeps that function's generic parsing behavior
/// untouched. Every bracket in the chain must hold a recognized literal key
/// — if ANY of them doesn't (e.g. a class constant, `$config[self::KEY]['x']`),
/// the whole name is left UNTOUCHED rather than partially split, same
/// no-corruption reasoning as the single-key case below.
pub(crate) fn split_array_key_suffix(name: &str) -> (String, Vec<mir_types::ArrayKey>) {
    let mut rest = name;
    let mut keys = Vec::new();
    while rest.ends_with(']') {
        let Some(open) = rest.rfind('[') else {
            return (name.to_string(), Vec::new());
        };
        if open == 0 {
            return (name.to_string(), Vec::new());
        }
        let inner = &rest[open + 1..rest.len() - 1];
        // A bracket suffix whose key isn't a recognized literal must leave
        // the name UNTOUCHED, not just drop the key — stripping the
        // brackets here would turn `config[self::KEY]` into the bare name
        // `config`, which then matches the real `$config` parameter and
        // (via `apply_assertions` treating an empty path as "target the
        // whole variable") silently overwrites the entire parameter's type
        // with the assertion's leaf type instead of a harmless no-op.
        // Returning the original bracketed text keeps this a no-match
        // against any real parameter name instead.
        let Some(key) = parse_literal_array_key(inner.trim()) else {
            return (name.to_string(), Vec::new());
        };
        keys.push(key);
        rest = &rest[..open];
    }
    if rest.is_empty() {
        return (name.to_string(), Vec::new());
    }
    // Keys were collected innermost-last-to-first (`arr['a']['b']` pushes
    // "b" then "a"); the path must read outermost-first to match traversal
    // order.
    keys.reverse();
    (rest.to_string(), keys)
}

fn parse_literal_array_key(inner: &str) -> Option<mir_types::ArrayKey> {
    if let Some(s) = inner.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) {
        return Some(
            canonical_int_array_key(s)
                .map(mir_types::ArrayKey::Int)
                .unwrap_or_else(|| mir_types::ArrayKey::String(std::sync::Arc::from(s))),
        );
    }
    if let Some(s) = inner.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        return Some(
            canonical_int_array_key(s)
                .map(mir_types::ArrayKey::Int)
                .unwrap_or_else(|| mir_types::ArrayKey::String(std::sync::Arc::from(s))),
        );
    }
    inner.parse::<i64>().ok().map(mir_types::ArrayKey::Int)
}

fn parse_shape_key(inner: &str) -> mir_types::ArrayKey {
    if let Some(key) = parse_literal_array_key(inner) {
        return key;
    }
    mir_types::ArrayKey::String(std::sync::Arc::from(inner))
}

/// `new<T>` and offset access `T[K]`, whose result needs the bound template.
fn is_deferred_type(s: &str) -> bool {
    let s = s.trim();
    (s.ends_with(']') && !s.ends_with("[]") && s.contains('['))
        || (s.ends_with('>') && strip_ascii_ci_prefix(s, "new<").is_some())
}

/// `new<C>` / `T[K]` as the `new` / `offset-access` pseudo-types the call site resolves.
fn parse_deferred_type(s: &str) -> Type {
    let (name, args) = match strip_ascii_ci_prefix(s, "new<") {
        Some(inner) => ("\\new", vec![parse_type_string(&inner[..inner.len() - 1])]),
        None => {
            let mut depth = 0;
            let open = s
                .char_indices()
                .rev()
                .find(|&(_, c)| {
                    match c {
                        ']' => depth += 1,
                        '[' => depth -= 1,
                        _ => {}
                    }
                    depth == 0
                })
                .map_or(0, |(i, _)| i);
            let target = parse_type_string(&s[..open]);
            let offset = parse_type_string(&s[open + 1..s.len() - 1]);
            ("\\offset-access", vec![target, offset])
        }
    };
    Type::single(Atomic::TNamedObject {
        fqcn: mir_types::Name::from(name),
        type_params: mir_types::union::vec_to_type_params(args),
    })
}

/// Scalars that are not falsy: `0.0` stays possible since float has no atom without it.
fn non_empty_scalar() -> Type {
    Type::from_vec(vec![
        Atomic::TNonFalsyString,
        Atomic::TNegativeInt,
        Atomic::TPositiveInt,
        Atomic::TFloat,
        Atomic::TTrue,
    ])
}

pub(crate) fn parse_type_string(s: &str) -> Type {
    let s = s.trim();
    // Docblock keywords are legal with a leading global `\` (`@param \int $x`);
    // class names cannot contain `-`, so a keyword is a keyword regardless.
    let mut s = s;
    if let Some(stripped) = s.strip_prefix('\\') {
        if is_docblock_type_keyword(stripped) {
            s = stripped;
        }
    }

    // Call-site variance on a type argument: `covariant Model` is "some subtype of
    // Model", `contravariant Model` "some supertype", `*` "any argument".
    let call_site = match s.split_once(' ') {
        Some(("covariant" | "contravariant", inner)) => Some(parse_type_string(inner)),
        None if s == "*" => Some(Type::mixed()),
        _ => None,
    };
    if let Some(bound) = call_site {
        return Type::single(Atomic::TTemplateParam {
            name: mir_types::Name::from(s),
            as_type: Box::new(bound),
            defining_entity: mir_types::Name::from(""),
        });
    }

    // `Foo::class` / `T::class` names the class string of that class or template.
    if let Some(class) = s
        .len()
        .checked_sub("::class".len())
        .filter(|&i| s.is_char_boundary(i) && s[i..].eq_ignore_ascii_case("::class"))
        .map(|i| &s[..i])
        .filter(|c| !c.is_empty() && !c.contains(['<', '(', '{', '|', '&', ' ', ':']))
    {
        return Type::single(Atomic::TClassString(Some(normalize_fqcn(class).into())));
    }

    // Nullable shorthand: `?Type`
    if let Some(inner) = s.strip_prefix('?') {
        let inner_ty = parse_type_string(inner);
        let mut u = inner_ty;
        u.add_type(Atomic::TNull);
        return u;
    }

    // Conditional type: `($param is TypeName ? TrueType : FalseType)`
    // Parenthesized type: `(A&B)|null` — strip outer parens and recurse.
    // `(A)|(B)` starts and ends with parens but is a union, not one group.
    if is_balanced_parens(s) {
        let inner = s[1..s.len() - 1].trim();
        if let Some(conditional) = parse_conditional_type(inner) {
            return conditional;
        }
        return parse_type_string(inner);
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
            // `non-empty-string&lowercase-string` is the narrowest of its scalar parts.
            let is_scalar = |part: &Type| part.types.iter().all(|a| a.is_string() || a.is_int());
            if parts.iter().all(is_scalar) {
                if let Some(refined) = intersect_refined_string(&parts) {
                    return refined;
                }
                if let Some(narrowest) = parts
                    .iter()
                    .find(|part| parts.iter().all(|other| part.is_subtype_structural(other)))
                {
                    return narrowest.clone();
                }
            }
            return Type::single(Atomic::TIntersection {
                parts: mir_types::union::vec_to_type_params(parts),
            });
        }
    }

    // Array shorthand: `Type[]` or `Type[][]` — Psalm/PHPStan document this as
    // `array<array-key, Type>`, not `array<int, Type>`: it makes no claim about
    // key shape, so a string-keyed array (e.g. `array_column`'s output, a
    // PSR-3 `$context` array) must still satisfy it.
    // A `[]` after a callable's `: ret` belongs to the return type
    // (`Closure(): string[]` returns `string[]`), so it is not an array-of-callable.
    let callable_return_suffix =
        split_callable_signature(s).is_some_and(|(_, _, _, after)| after.starts_with(':'));
    if let Some(value_str) = s.strip_suffix("[]").filter(|_| !callable_return_suffix) {
        let value = parse_type_string(value_str);
        return Type::single(Atomic::TArray {
            key: Box::new(Type::array_key()),
            value: Box::new(value),
        });
    }

    // Callable/closure syntax: `Closure(T): R` or `callable(T): R`
    if let Some(call_ty) = parse_callable_syntax(s) {
        return call_ty;
    }

    if is_deferred_type(s) {
        return parse_deferred_type(s);
    }

    // Array shape: `array{key: Type, ...}` or `list{Type, ...}`
    if s.ends_with('}') {
        if let Some(open) = s.find('{') {
            let prefix = s[..open].trim_end().to_lowercase();
            let inner = &s[open + 1..s.len() - 1];
            if prefix == "array" {
                return parse_keyed_array(inner, false);
            } else if prefix == "list" {
                return parse_keyed_array(inner, true);
            } else if matches!(prefix.as_str(), "object" | "stdclass" | "\\stdclass") {
                return match parse_keyed_array(inner, false).types.into_iter().next() {
                    Some(Atomic::TKeyedArray { properties, .. }) if !properties.is_empty() => {
                        Type::single(Atomic::TObjectShape { properties })
                    }
                    _ => Type::single(Atomic::TObject),
                };
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

    // Lowercase only: `Number` stays a class name (`BcMath\Number`).
    if s == "number" {
        let mut u = Type::single(Atomic::TInt);
        u.add_type(Atomic::TFloat);
        return u;
    }

    // Keywords
    let lower = s.to_lowercase();
    match lower.as_str() {
        "string" => Type::single(Atomic::TString),
        "non-empty-string" => Type::single(Atomic::TNonEmptyString),
        "numeric-string" => Type::single(Atomic::TNumericString),
        "truthy-string" | "non-falsy-string" => Type::single(Atomic::TNonFalsyString),
        "literal-string" => refined_string(StringRefinement::Literal, false),
        "non-empty-literal-string" => refined_string(StringRefinement::Literal, true),
        "decimal-int-string" => refined_string(StringRefinement::DecimalInt, false),
        "non-decimal-int-string" => refined_string(StringRefinement::NonDecimalInt, false),
        "lowercase-string" => refined_string(StringRefinement::Lowercase, false),
        "uppercase-string" => refined_string(StringRefinement::Uppercase, false),
        "non-empty-lowercase-string" => refined_string(StringRefinement::Lowercase, true),
        "non-empty-uppercase-string" => refined_string(StringRefinement::Uppercase, true),
        "class-string" => Type::single(Atomic::TClassString(None)),
        "interface-string" => Type::single(Atomic::TInterfaceString(None)),
        "trait-string" => Type::single(Atomic::TTraitString),
        "enum-string" => Type::single(Atomic::TEnumString),
        "int-mask" | "int-mask-of" => Type::single(Atomic::TInt),
        "class-string-map" => {
            let mut t = Type::single(Atomic::TArray {
                key: Box::new(Type::single(Atomic::TClassString(None))),
                value: Box::new(Type::mixed()),
            });
            t.add_type(Atomic::TObject);
            t
        }
        "non-empty-associative-array" => {
            associative_array_type(Type::array_key(), Type::mixed(), true)
        }
        "non-empty-list" => Type::single(Atomic::TNonEmptyList {
            value: Box::new(Type::mixed()),
        }),
        "arraylike-object" => {
            let mut t = arraylike_object_type(Type::array_key(), Type::mixed());
            t.add_type(Atomic::TObject);
            t
        }
        "int" | "integer" | "int-range" => Type::single(Atomic::TInt),
        "literal-int" => Type::single(Atomic::TRefinedInt(mir_types::IntRefinement::Literal)),
        "positive-int" => Type::single(Atomic::TPositiveInt),
        "negative-int" => Type::single(Atomic::TNegativeInt),
        "non-negative-int" => Type::single(Atomic::TNonNegativeInt),
        "non-positive-int" => Type::single(Atomic::TIntRange {
            min: None,
            max: Some(0),
        }),
        "non-zero-int" => {
            let mut u = Type::single(Atomic::TNegativeInt);
            u.add_type(Atomic::TPositiveInt);
            u
        }
        "float" | "double" => Type::single(Atomic::TFloat),
        "bool" | "boolean" => Type::single(Atomic::TBool),
        "true" => Type::single(Atomic::TTrue),
        "false" => Type::single(Atomic::TFalse),
        // Reading an unset variable yields null.
        "null" | "unset" => Type::single(Atomic::TNull),
        "void" => Type::single(Atomic::TVoid),
        "never" | "never-return" | "no-return" | "never-returns" | "noreturn" => {
            Type::single(Atomic::TNever)
        }
        "mixed" => Type::single(Atomic::TMixed),
        "non-null-mixed" => Type::from_vec(vec![
            Atomic::TScalar,
            Atomic::TArray {
                key: Box::new(Type::array_key()),
                value: Box::new(Type::mixed()),
            },
            Atomic::TObject,
            Atomic::TResource,
        ]),
        "non-empty-mixed" => {
            let mut u = non_empty_scalar();
            u.add_type(Atomic::TNonEmptyArray {
                key: Box::new(Type::array_key()),
                value: Box::new(Type::mixed()),
            });
            u.add_type(Atomic::TObject);
            u.add_type(Atomic::TResource);
            u
        }
        "non-empty-scalar" => non_empty_scalar(),
        "object" => Type::single(Atomic::TObject),
        "array" => Type::single(Atomic::TArray {
            key: Box::new(Type::array_key()),
            value: Box::new(Type::mixed()),
        }),
        "callable-object" | "callable-array" => {
            let base = if lower == "callable-object" {
                Atomic::TObject
            } else {
                Atomic::TArray {
                    key: Box::new(Type::array_key()),
                    value: Box::new(Type::mixed()),
                }
            };
            Type::single(Atomic::TIntersection {
                parts: mir_types::union::vec_to_type_params(vec![
                    Type::single(base),
                    Type::single(Atomic::TCallable {
                        params: None,
                        return_type: None,
                        is_pure: false,
                    }),
                ]),
            })
        }
        "non-empty-array" => Type::single(Atomic::TNonEmptyArray {
            key: Box::new(Type::array_key()),
            value: Box::new(Type::mixed()),
        }),
        "associative-array" => associative_array_type(Type::array_key(), Type::mixed(), false),
        "list" => Type::single(Atomic::TList {
            value: Box::new(Type::mixed()),
        }),
        // Bare `pure-callable`/`pure-Closure` with no `(...)` signature — the
        // parenthesized form is handled above by `parse_callable_syntax`
        // (which already strips the purity qualifier), but a bare keyword
        // with no signature never reaches it and would otherwise fall through
        // to the named-class catch-all below as a bogus class literally named
        // "pure-callable".
        "callable" => Type::single(Atomic::TCallable {
            params: None,
            return_type: None,
            is_pure: false,
        }),
        "pure-callable" => Type::single(Atomic::TCallable {
            params: None,
            return_type: None,
            is_pure: true,
        }),
        // Bare `Closure` isn't listed here — it's a real PHP class, so it
        // already resolves correctly via the named-class fallthrough below.
        // `pure-closure` isn't a real class name, so it needs the same
        // explicit mapping.
        "pure-closure" | "static-closure" | "static-pure-closure" => {
            let signature = if lower.contains("pure") {
                "pure-Closure(mixed...): mixed"
            } else {
                "Closure(mixed...): mixed"
            };
            let is_static = lower.starts_with("static");
            Type::from_vec(
                parse_type_string(signature)
                    .types
                    .into_iter()
                    .map(|a| match a {
                        Atomic::TClosure { mut data } => {
                            data.is_static = is_static;
                            Atomic::TClosure { data }
                        }
                        other => other,
                    })
                    .collect(),
            )
        }
        "callable-string" => Type::single(Atomic::TCallableString),
        // The leading backslash keeps namespace resolution from qualifying it.
        "stringable-object" => Type::single(Atomic::TNamedObject {
            fqcn: mir_types::Name::from("\\Stringable"),
            type_params: Default::default(),
        }),
        "iterable" => {
            let mut u = Type::single(Atomic::TArray {
                key: Box::new(Type::array_key()),
                value: Box::new(Type::mixed()),
            });
            u.add_type(Atomic::TNamedObject {
                fqcn: mir_types::Name::from("\\Traversable"),
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
                properties: Arc::default(),
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
        "resource" | "open-resource" => Type::single(Atomic::TResource),
        "closed-resource" => Type::single(Atomic::TClosedResource),
        // self/static/parent: emit sentinel with empty FQCN; collector fills it in.
        "static" | "$this" => Type::single(Atomic::TStaticObject {
            fqcn: mir_types::Name::from(""),
        }),
        "self" => Type::single(Atomic::TSelf {
            fqcn: mir_types::Name::from(""),
        }),
        "parent" => Type::single(Atomic::TParent {
            fqcn: mir_types::Name::from(""),
        }),

        // A docblock keyword with no arm above (table/arms drift) is still a
        // keyword — approximate as mixed, never a named class to resolve.
        _ if is_docblock_type_keyword(s) => Type::mixed(),

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
            if let Some(t) = resolve_class_constant_type(s) {
                return t;
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

        // String literal: `'foo'` or `"bar"` (len check excludes a lone quote, which would
        // otherwise satisfy both starts_with/ends_with and panic on the slice below)
        _ if s.len() >= 2
            && ((s.starts_with('\'') && s.ends_with('\''))
                || (s.starts_with('"') && s.ends_with('"'))) =>
        {
            let inner = &s[1..s.len() - 1];
            Type::single(Atomic::TLiteralString(Arc::from(inner)))
        }

        _ => Type::mixed(),
    }
}

pub(super) fn parse_generic(name: &str, inner: &str) -> Type {
    // A leading `\` marks a class name global, but keywords may legally
    // carry one (`\int-mask<...>` — it stays a keyword, never a qualified
    // class). Strip only for keywords: for a real class the backslash is
    // the only thing keeping the name global, and losing it would let the
    // collector re-qualify it against the docblock's namespace.
    let mut name = name;
    if let Some(stripped) = name.strip_prefix('\\') {
        if is_docblock_type_keyword(stripped) {
            name = stripped;
        }
    }
    match name.to_lowercase().as_str() {
        "array" => {
            let params = split_generics(inner);
            let (key, value) = match params.len() {
                n if n >= 2 => (
                    parse_type_string(params[0].trim()),
                    parse_type_string(params[1].trim()),
                ),
                1 => (Type::array_key(), parse_type_string(params[0].trim())),
                _ => (Type::array_key(), Type::mixed()),
            };
            Type::single(Atomic::TArray {
                key: Box::new(key),
                value: Box::new(value),
            })
        }
        "associative-array" | "non-empty-associative-array" => {
            let params = split_generics(inner);
            let (key, value) = match params.len() {
                n if n >= 2 => (
                    parse_type_string(params[0].trim()),
                    parse_type_string(params[1].trim()),
                ),
                1 => (Type::array_key(), parse_type_string(params[0].trim())),
                _ => (Type::array_key(), Type::mixed()),
            };
            associative_array_type(key, value, name.to_lowercase().starts_with("non-empty"))
        }
        // Psalm-only pseudo-type: any object implementing ArrayAccess,
        // Countable, and Traversable. The bare spelling remains unresolved in
        // Psalm too; only the generic form is meaningful.
        "arraylike-object" => {
            let params = split_generics(inner);
            let (key, value) = match params.len() {
                n if n >= 2 => (
                    parse_type_string(params[0].trim()),
                    parse_type_string(params[1].trim()),
                ),
                1 => (Type::array_key(), parse_type_string(params[0].trim())),
                _ => (Type::array_key(), Type::mixed()),
            };
            arraylike_object_type(key, value)
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
            let (key, value) = match params.len() {
                n if n >= 2 => (
                    parse_type_string(params[0].trim()),
                    parse_type_string(params[1].trim()),
                ),
                1 => (Type::array_key(), parse_type_string(params[0].trim())),
                _ => (Type::array_key(), Type::mixed()),
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
                1 => (Type::array_key(), parse_type_string(params[0].trim())),
                _ => (Type::array_key(), Type::mixed()),
            };
            let mut u = Type::single(Atomic::TArray {
                key: Box::new(key.clone()),
                value: Box::new(value.clone()),
            });
            u.add_type(Atomic::TNamedObject {
                fqcn: mir_types::Name::from("\\Traversable"),
                type_params: mir_types::union::vec_to_type_params(vec![key, value]),
            });
            u
        }
        "class-string" => {
            // `class-string<A|B>` is `class-string<A>|class-string<B>`; an intersection keeps its
            // first member, the only one the atom can name.
            let mut u = Type::empty();
            for part in split_union(inner.trim()) {
                let part = part.trim().trim_start_matches('(').trim_end_matches(')');
                let first = split_intersection(part)
                    .into_iter()
                    .next()
                    .unwrap_or_default();
                u.add_type(Atomic::TClassString(Some(
                    normalize_fqcn(strip_generic_args(first.trim())).into(),
                )));
            }
            if u.is_empty() {
                u.add_type(Atomic::TClassString(None));
            }
            u
        }
        "interface-string" => Type::single(Atomic::TInterfaceString(Some(
            normalize_fqcn(strip_generic_args(inner.trim())).into(),
        ))),
        "int" | "int-range" => {
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
        // PHPStan's lenient union: the members still describe the value.
        "__benevolent" => parse_type_string(inner.trim()),
        keyword @ ("properties-of"
        | "public-properties-of"
        | "protected-properties-of"
        | "private-properties-of") => {
            let filter = match keyword {
                "public-properties-of" => mir_types::PropertyFilter::Public,
                "protected-properties-of" => mir_types::PropertyFilter::Protected,
                "private-properties-of" => mir_types::PropertyFilter::Private,
                _ => mir_types::PropertyFilter::All,
            };
            // Only a plain class name can be looked up later.
            match parse_type_string(inner.trim()).types.as_slice() {
                [Atomic::TNamedObject { fqcn, type_params }] if type_params.is_empty() => {
                    Type::single(Atomic::TPropertiesOf {
                        class: *fqcn,
                        filter,
                    })
                }
                _ => Type::single(Atomic::TArray {
                    key: Box::new(Type::string()),
                    value: Box::new(Type::mixed()),
                }),
            }
        }
        // `template-type<Obj, Cls, 'T'>` stays a named pseudo-type until `Obj` is concrete.
        // The leading backslash keeps namespace resolution from qualifying it.
        "template-type" => Type::single(Atomic::TNamedObject {
            fqcn: mir_types::Name::from("\\template-type"),
            type_params: mir_types::union::vec_to_type_params(
                split_generics(inner)
                    .iter()
                    .map(|arg| parse_type_string(arg.trim()))
                    .collect(),
            ),
        }),
        // `key-of<T>` — the union of array/shape key types of `T`.
        "key-of" => {
            let inner_ty = parse_type_string(inner.trim());
            eval_key_of(&inner_ty).unwrap_or_else(|| {
                Type::single(Atomic::TKeyOf {
                    target: Box::new(inner_ty),
                })
            })
        }
        // `value-of<T>` — the union of array/shape value types of `T`.
        "value-of" => {
            let inner_ty = parse_type_string(inner.trim());
            eval_value_of(&inner_ty).unwrap_or_else(|| {
                Type::single(Atomic::TValueOf {
                    target: Box::new(inner_ty),
                })
            })
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
        // Another class's constants are not known here, so that mask is resolved when checked.
        "int-mask-of" => resolve_int_mask_of(inner).unwrap_or_else(|| {
            Type::single(match deferred_int_mask_target(inner) {
                Some(target) => Atomic::TRefinedInt(mir_types::IntRefinement::MaskOf(target)),
                None => Atomic::TInt,
            })
        }),
        // `class-string-map<T, V>` — maps `class-string<T>` keys to `V` values.
        // mir does not tie the value type to the specific class looked up (that
        // needs flow-sensitive template binding at each access site), so
        // approximate as a plain array from `class-string` to `V`.
        "class-string-map" => {
            let params = split_generics(inner);
            // `T of Bound` declares a template in place; a value naming it is read as the bound.
            let inline_template = params.first().and_then(|p| {
                let p = p.trim();
                let (name, bound) = p.split_once(" of ").or(p.split_once(" as "))?;
                Some((name.trim(), bound.trim()))
            });
            // The two-arg form (`class-string-map<T, V>`) is canonical, but
            // Psalm also accepts the one-arg shorthand (`class-string-map<T>`)
            // where the value type defaults to `T` itself, not `mixed`.
            let value_is_key = params.len() == 1
                || inline_template.is_some_and(|(name, _)| params[1].trim() == name);
            let value = params
                .get(1)
                .or(params.first())
                .map(|p| match inline_template {
                    Some((_, bound)) if value_is_key => parse_type_string(bound),
                    _ => parse_type_string(p.trim()),
                })
                .unwrap_or_else(Type::mixed);
            // `class-string<X>` keys mapped to `X` mark each value as an instance of its key.
            let key_class = match value.types.as_slice() {
                [Atomic::TNamedObject { fqcn, type_params }]
                    if value_is_key && type_params.is_empty() =>
                {
                    Some(*fqcn)
                }
                _ => None,
            };
            Type::single(Atomic::TArray {
                key: Box::new(Type::single(Atomic::TClassString(key_class))),
                value: Box::new(value),
            })
        }
        // `static<T>`/`parent<T>` — a common Doctrine/Collection-style
        // "generic self-referencing collection" pattern. These pseudo-type atoms
        // carry no `type_params` field of their own (unlike `TNamedObject`), so
        // the written `<...>` args are dropped rather than parsed as a bogus
        // named class called "static"/"parent" — substitution re-attaches
        // the real receiver's own type params regardless.
        "static" | "$this" => Type::single(Atomic::TStaticObject {
            fqcn: mir_types::Name::from(""),
        }),
        // `self<T>` names the declaring class with these args; the collector fills the class in.
        "self" => Type::single(Atomic::TNamedObject {
            fqcn: mir_types::Name::from("self"),
            type_params: mir_types::union::vec_to_type_params(
                split_generics(inner)
                    .iter()
                    .map(|p| parse_type_string(p.trim()))
                    .collect(),
            ),
        }),
        "parent" => Type::single(Atomic::TParent {
            fqcn: mir_types::Name::from(""),
        }),
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
pub(crate) fn expand_int_mask_members(members: &[i64]) -> Option<Type> {
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
type SelfIntConstants = (Arc<str>, Arc<Vec<(Arc<str>, Atomic)>>);

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
    pub(crate) fn activate(fqcn: &str, constants: &Arc<Vec<(Arc<str>, Atomic)>>) -> Self {
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

/// Literal values of the active class's constants matching `pattern` (`NAME` or
/// a glob such as `PREFIX*`), when `class_ref` is `self`/`static`/the class's own name.
fn self_constant_values(class_ref: &str, pattern: &str) -> Option<Vec<Atomic>> {
    let class_ref = class_ref.trim();
    let pattern = pattern.trim();
    SELF_INT_CONSTANTS.with(|cell| {
        let active = cell.borrow();
        let (fqcn, constants) = active.as_ref()?;
        // `class_ref` is raw docblock text, never namespace-resolved, so a bare
        // short name counts as a self-reference.
        let short_name = fqcn.rsplit('\\').next().unwrap_or(fqcn);
        let is_self_ref = class_ref == "$this"
            || class_ref.eq_ignore_ascii_case("self")
            || class_ref.eq_ignore_ascii_case("static")
            || normalize_fqcn(class_ref).eq_ignore_ascii_case(fqcn)
            || (!class_ref.contains('\\') && class_ref.eq_ignore_ascii_case(short_name));
        if !is_self_ref {
            return None;
        }
        Some(
            constants
                .iter()
                .filter(|(name, _)| constant_glob_matches(pattern, name))
                .map(|(_, v)| v.clone())
                .collect(),
        )
    })
}

/// `*` matches any run of characters: `PREFIX_*`, `*_SUFFIX`, `A_*_B`, `*`.
pub(crate) fn constant_glob_matches(pattern: &str, name: &str) -> bool {
    let mut parts = pattern.split('*');
    let first = parts.next().unwrap_or_default();
    let Some(mut rest) = name.strip_prefix(first) else {
        return false;
    };
    let mut parts = parts.peekable();
    if parts.peek().is_none() {
        return rest.is_empty();
    }
    while let Some(part) = parts.next() {
        if parts.peek().is_none() {
            return rest.ends_with(part);
        }
        match rest.find(part) {
            Some(i) => rest = &rest[i + part.len()..],
            None => return false,
        }
    }
    true
}

/// Resolve `int-mask-of<T::CONST_PREFIX*>` (or bare `T::*`) using the ambient
/// class constants set by `SelfIntConstantsGuard`. Returns `None` when there
/// is no active guard, `T` isn't a self-reference, no constants match the
/// prefix, or the match set can't be expanded (see `expand_int_mask_members`).
pub(super) fn resolve_int_mask_of(inner: &str) -> Option<Type> {
    let (class_ref, pattern) = inner.trim().split_once("::")?;
    if !pattern.contains('*') {
        return None;
    }
    let mut members: Vec<i64> = self_constant_values(class_ref, pattern)?
        .into_iter()
        .filter_map(|a| match a {
            Atomic::TLiteralInt(n) => Some(n),
            _ => None,
        })
        .collect();
    members.sort_unstable();
    members.dedup();
    expand_int_mask_members(&members)
}

/// `Cls::PATTERN*` naming a class other than the one being collected.
fn deferred_int_mask_target(inner: &str) -> Option<mir_types::Name> {
    let inner = inner.trim();
    let (class_ref, pattern) = inner.split_once("::")?;
    (pattern.contains('*') && self_constant_values(class_ref, pattern).is_none())
        .then(|| mir_types::Name::from(inner))
}

/// Resolve `self::NAME` / `self::PREFIX_*` to the union of the matching literal
/// int/string constants. `None` for other classes and non-literal constants,
/// which keep their named-class fallback.
fn resolve_class_constant_type(s: &str) -> Option<Type> {
    let (class_ref, pattern) = s.split_once("::")?;
    let values = self_constant_values(class_ref, pattern)?;
    if values.is_empty() {
        return None;
    }
    let mut u = Type::empty();
    for v in values {
        u.add_type(v);
    }
    Some(u)
}

/// Case-insensitive ASCII prefix strip that can never panic on a char
/// boundary: `prefix` must be pure ASCII, and the comparison runs byte-for-
/// byte, so a multi-byte character in `s` can never spuriously "match" it the
/// way `s.to_lowercase().starts_with(prefix)` could if lowercasing changed a
/// character's byte length (see `docblock_unicode_type_no_panic.phpt`).
pub(super) fn strip_ascii_ci_prefix<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    let bytes = s.as_bytes();
    (bytes.len() >= prefix.len() && bytes[..prefix.len()].eq_ignore_ascii_case(prefix.as_bytes()))
        .then(|| &s[prefix.len()..])
}

/// Drops the `// …` comments a multi-line shape may carry after its items.
fn strip_line_comments(s: &str) -> std::borrow::Cow<'_, str> {
    if !s.contains("//") {
        return std::borrow::Cow::Borrowed(s);
    }
    let code = |line: &str| {
        let mut prev_slash = None;
        for (i, ch) in unquoted_char_indices(line) {
            if ch == '/' && prev_slash.is_some_and(|p| p + 1 == i) {
                return line[..i - 1].to_string();
            }
            prev_slash = (ch == '/').then_some(i);
        }
        line.to_string()
    };
    std::borrow::Cow::Owned(s.lines().map(code).collect::<Vec<_>>().join("\n"))
}

pub(super) fn parse_keyed_array(inner: &str, is_list: bool) -> Type {
    use mir_types::atomic::KeyedProperty;
    let mut properties: mir_types::KeyedProperties = mir_types::KeyedProperties::default();
    let mut is_open = false;
    let mut remainder = None;
    let mut auto_index = 0i64;

    let inner = strip_line_comments(inner);
    for item in split_generics(&inner) {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        if let Some(rest) = item.strip_prefix("...") {
            is_open = true;
            if let Some(args) = rest.strip_prefix('<').and_then(|r| r.strip_suffix('>')) {
                remainder = split_generics(args)
                    .last()
                    .map(|v| parse_type_string(v.trim()));
            }
            continue;
        }
        // Find a colon that is not inside nested generics/braces
        let colon_pos = {
            let mut depth = 0i32;
            let mut found = None;
            for (i, ch) in unquoted_char_indices(item) {
                match ch {
                    '<' | '(' | '{' => depth += 1,
                    '>' | ')' | '}' => depth -= 1,
                    // A colon after `)` is a callable's return-type separator; `::` is a constant.
                    ':' if depth == 0
                        && !item[..i].trim_end().ends_with(')')
                        && !item[..i].ends_with(':')
                        && !item[i + 1..].starts_with(':') =>
                    {
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
            let key = parse_shape_key(key_str);
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

    properties.remainder = remainder;
    Type::single(Atomic::TKeyedArray {
        properties: Arc::new(properties),
        is_open,
        is_list,
    })
}

/// Splits `[pure-]Closure(params): ret` / `[pure-]callable(params)` into
/// (is_closure, is_pure, params, text after `)`).
fn split_callable_signature(s: &str) -> Option<(bool, bool, &str, &str)> {
    let s = s.trim_start_matches('\\');
    let (is_pure, s_after_pure) = match strip_ascii_ci_prefix(s, "pure-") {
        Some(rest) => (true, rest),
        None => (false, s),
    };
    let (is_closure, rest) = if let Some(rest) = strip_ascii_ci_prefix(s_after_pure, "closure") {
        (true, rest)
    } else {
        let rest = strip_ascii_ci_prefix(s_after_pure, "callable")?;
        (false, rest)
    };
    let rest = rest.trim_start();
    if !rest.starts_with('(') {
        return None;
    }
    let close = find_matching_paren(rest)?;
    Some((
        is_closure,
        is_pure,
        &rest[1..close],
        rest[close + 1..].trim(),
    ))
}

pub(super) fn parse_callable_syntax(s: &str) -> Option<Type> {
    let (is_closure, is_pure, params_str, after) = split_callable_signature(s)?;
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
            let (p, mut is_variadic) = match p.strip_prefix("...") {
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
            // Postfix variadic: `mixed...` / `int ...$rest`.
            let ty_str = match ty_str.strip_suffix("...") {
                Some(rest) => {
                    is_variadic = true;
                    rest.trim_end()
                }
                None => ty_str,
            };
            let (ty_str, is_byref) = match ty_str.strip_suffix('&') {
                Some(rest) => (rest.trim_end(), true),
                None => (ty_str, false),
            };
            mir_types::atomic::FnParam {
                name: name.into(),
                ty: Some(mir_types::SimpleType::from_union(parse_type_string(ty_str))),
                out_ty: None,
                default: None,
                is_variadic,
                is_byref,
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
                is_pure,
                is_static: false,
                rejects_surplus_args: false,
                is_impure: false,
            }),
        }))
    } else {
        Some(Type::single(Atomic::TCallable {
            params: Some(Box::new(params.into_vec())),
            return_type,
            is_pure,
        }))
    }
}

pub(super) fn find_matching_paren(s: &str) -> Option<usize> {
    if !s.starts_with('(') {
        return None;
    }
    let mut depth = 0i32;
    for (i, ch) in unquoted_char_indices(s) {
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
        let mut bound_tokens: Vec<&str> = Vec::new();
        let mut depth: i32 = 0;
        while let Some(&t) = tokens.peek() {
            // A trailing `= Default` (e.g. `@template T of Bound = Default`)
            // belongs to the default value, not the bound.
            if t == "=" && depth == 0 {
                break;
            }
            // Stop at trailing free-text description once the bound's own
            // brackets are balanced (e.g. `@template T of Collection<int> the
            // type of the collection's items`) — a token reached at depth 0
            // that isn't glued to the previous one by a dangling `|`/`&`
            // union/intersection operator marks the end of the type
            // expression, not a continuation of it. The first bound token is
            // always kept regardless (a bound is never empty).
            if depth == 0 && !bound_tokens.is_empty() {
                let prev_ends_with_op = bound_tokens
                    .last()
                    .is_some_and(|p| p.ends_with('|') || p.ends_with('&'));
                let starts_with_op = t.starts_with('|') || t.starts_with('&');
                if !prev_ends_with_op && !starts_with_op {
                    break;
                }
            }
            for c in t.chars() {
                match c {
                    '<' | '(' | '[' | '{' => depth += 1,
                    '>' | ')' | ']' | '}' => depth -= 1,
                    _ => {}
                }
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

/// Splits `@psalm-type` / `@phpstan-type` as `Name = Expr` or `Name Expr`. A generic
/// name's `<T>` suffix is dropped so bare use sites resolve; T is not substituted.
pub(super) fn split_type_alias_decl(body: &str) -> Option<(&str, &str)> {
    let mut depth = 0i32;
    let end = body.char_indices().find_map(|(i, ch)| {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            _ if depth == 0 && (ch == '=' || ch.is_whitespace()) => return Some(i),
            _ => {}
        }
        None
    })?;
    let name = body[..end].split('<').next().unwrap_or_default().trim();
    let expr = body[end..].trim_start();
    let expr = expr.strip_prefix('=').unwrap_or(expr).trim();
    (!name.is_empty() && !expr.is_empty()).then_some((name, expr))
}

/// Parse `@psalm-import-type` body.
///
/// Formats:
/// - `AliasName from SourceClass`
/// - `AliasName as LocalAlias from SourceClass`
pub(super) fn parse_import_type(body: &str) -> Option<DocImportType> {
    // Token-based, so the `from` clause may continue on the next docblock line.
    let tokens: Vec<&str> = body.split_whitespace().collect();
    let from = tokens.iter().position(|t| *t == "from")?;
    let from_class = tokens.get(from + 1)?.to_string();
    let (original, local) = match tokens[..from] {
        [name] => (name, name),
        [original, "as", local] => (original, local),
        _ => return None,
    };
    Some(DocImportType {
        original: original.to_string(),
        local: local.to_string(),
        from_class,
    })
}

/// Prose in a tag `body` after its leading `prefix`, whitespace-collapsed.
/// Empty when `body` does not start with `prefix`.
pub(super) fn description_after(body: &str, prefix: &str) -> String {
    body.trim_start()
        .strip_prefix(prefix)
        .map(|rest| rest.split_whitespace().collect::<Vec<_>>().join(" "))
        .unwrap_or_default()
}

/// Prose after the type and `[&][...]$name` of a `@param` body.
pub(super) fn param_description(body: &str, ty: &str, name: &str) -> String {
    let after_ty = body
        .trim_start()
        .strip_prefix(ty)
        .unwrap_or("")
        .trim_start();
    let after_flags = after_ty.strip_prefix('&').unwrap_or(after_ty);
    let after_flags = after_flags.strip_prefix("...").unwrap_or(after_flags);
    after_flags
        .strip_prefix('$')
        .and_then(|rest| rest.strip_prefix(name))
        .map(|rest| rest.split_whitespace().collect::<Vec<_>>().join(" "))
        .unwrap_or_default()
}

pub(super) fn parse_param_line(s: &str) -> Option<(String, String)> {
    // Formats: `Type $name`, `Type $name description`, or `Type &$name ...` (byref)
    // Types can contain $-named params inside callable syntax (`callable(int $a): void`),
    // so we track bracket depth and return the FIRST `$identifier` found at depth 0.
    // Using first-match (not last) prevents description text that contains $var references
    // from being mistaken for the parameter name. Scanning the whole body (not just its
    // first physical line) is what lets a wrapped multi-line `array{...}`/`array<...>`
    // shape still resolve to its `$name`, and the depth tracking already keeps the
    // shape's interior (and any newlines inside it) from being mistaken for the boundary.
    //
    // The type is optional in PHPDoc grammar (`@param $name description`); a body
    // starting directly with `$`/`&$`/`...$` at depth 0 has no leading whitespace
    // to trigger the loop below, so handle it up front with an empty type part.
    let leading = s.strip_prefix('&').unwrap_or(s);
    let leading = leading.strip_prefix("...").unwrap_or(leading);
    if let Some(rest) = leading.strip_prefix('$') {
        if let Some(name) = rest.split(char::is_whitespace).next() {
            if !name.is_empty() {
                return Some((String::new(), name.to_string()));
            }
        }
    }

    let mut depth: i32 = 0;

    for (i, ch) in unquoted_char_indices(s) {
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

/// Like `parse_param_line`, but for `@var`. `@param`'s name always
/// immediately follows the type, so scanning past a non-`$` token to find
/// one deeper in the body (as `parse_param_line` deliberately does, to skip
/// callable-syntax internals) is safe. `@var` is usually nameless free-form
/// prose instead (`@var string This mentions $foo somewhere`), where that
/// same scan would mistake a `$mention` in the description for the
/// variable's name. Only accept a name found at the FIRST depth-0 token
/// boundary; anything else means there is no name at all.
pub(super) fn parse_var_line(s: &str) -> Option<(String, String)> {
    let mut depth: i32 = 0;

    for (i, ch) in unquoted_char_indices(s) {
        match ch {
            '<' | '(' | '{' => depth += 1,
            '>' | ')' | '}' => depth = (depth - 1).max(0),
            _ if ch.is_whitespace() && depth == 0 => {
                let after = s[i..].trim_start();
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
                } else if after.starts_with('|')
                    || after.starts_with('&')
                    || s[..i].trim_end().ends_with(['|', '&'])
                    || s[..i].trim_end().ends_with(':')
                {
                    // A `|`/`&` immediately before or after this whitespace
                    // means the type expression is continuing across it (a
                    // spaced union/intersection like `string | null`), not a
                    // name boundary — keep scanning rather than giving up. A
                    // trailing `:` means this whitespace is the space in
                    // `callable(...): ReturnType`/`Closure(...): ReturnType`
                    // (the return-type arrow only ever appears at depth 0,
                    // after the signature's closing paren) — also a
                    // continuation, not a name boundary.
                    continue;
                }
                // The token right after the type isn't a name, and isn't a
                // continuation of the type either — unlike @param, @var
                // never has a name further into the description.
                return None;
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

    let track_quotes = !has_unterminated_quote(s);
    let mut in_quote: Option<char> = None;
    for (i, ch) in s.char_indices() {
        if let Some(q) = in_quote {
            current_token.push(ch);
            if ch == q {
                in_quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' if track_quotes => {
                in_quote = Some(ch);
                current_token.push(ch);
            }
            '<' | '(' | '{' => {
                depth += 1;
                current_token.push(ch);
            }
            '>' | ')' | '}' => {
                depth = (depth - 1).max(0);
                current_token.push(ch);
            }
            _ if ch.is_whitespace() && depth == 0 => {
                // A spaced union/intersection (`int | null`) continues the
                // type across the whitespace instead of ending it here.
                let after = s[i..].trim_start();
                let before = s[..i].trim_end();
                if after.starts_with('|') || after.starts_with('&') || before.ends_with(['|', '&'])
                {
                    current_token.push(' ');
                    continue;
                }
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

    // A sentence-ending dot (`@return void.`) isn't part of the type; `1.` and `...` are kept.
    let token = current_token.trim();
    match token.strip_suffix('.') {
        Some(t) if t.ends_with(|c: char| c.is_ascii_alphabetic() || c == '>' || c == ']') => {
            t.to_string()
        }
        _ => token.to_string(),
    }
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
    let track_quotes = !has_unterminated_quote(s);
    let mut in_quote: Option<char> = None;
    for ch in s.chars() {
        if let Some(q) = in_quote {
            current.push(ch);
            if ch == q {
                in_quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' if track_quotes => {
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
    let last = s.len() - 1;
    for (i, ch) in unquoted_char_indices(s) {
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
    let track_quotes = !has_unterminated_quote(s);
    let mut in_quote: Option<char> = None;
    for ch in s.chars() {
        if let Some(q) = in_quote {
            current.push(ch);
            if ch == q {
                in_quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' if track_quotes => {
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
    for (i, ch) in unquoted_char_indices(s) {
        match ch {
            '<' | '(' | '{' => depth += 1,
            '>' | ')' | '}' => depth -= 1,
            _ if ch.is_whitespace() && depth == 0 => {
                // A `|`/`&` immediately before or after this whitespace means
                // the type is continuing across it (a spaced union/
                // intersection); a trailing `:` means it's the space in
                // `callable(...): ReturnType`/`Closure(...): ReturnType`
                // (only ever appears at depth 0, right after the closing
                // paren) — same continuation logic as `parse_var_line`, kept
                // in sync since this is its nameless-form fallback.
                let after = s[i..].trim_start();
                let before = s[..i].trim_end();
                if after.starts_with('|')
                    || after.starts_with('&')
                    || before.ends_with(['|', '&'])
                    || before.ends_with(':')
                {
                    continue;
                }
                end = i;
                break;
            }
            _ => {}
        }
    }
    &s[..end]
}

/// `s.char_indices()` minus quoted literals (delimiters included), so a
/// bracket or separator inside `'->'` or `'a,b'` isn't treated as syntax.
/// An unterminated quote is scanned as plain text.
pub(super) fn unquoted_char_indices(s: &str) -> impl Iterator<Item = (usize, char)> + '_ {
    let skip_quotes = !has_unterminated_quote(s);
    let mut in_quote: Option<char> = None;
    s.char_indices().filter(move |&(_, ch)| {
        if !skip_quotes {
            return true;
        }
        if let Some(q) = in_quote {
            if ch == q {
                in_quote = None;
            }
            return false;
        }
        if ch == '\'' || ch == '"' {
            in_quote = Some(ch);
            return false;
        }
        true
    })
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

/// Whether `s` contains a `'` or `"` that is opened but never closed —
/// e.g. a lone `'`, or `'foo` with no closing quote. PHP type syntax never
/// contains an unterminated string literal, at top level or nested inside a
/// generic/shape (`array{': int}`).
pub(super) fn has_unterminated_quote(s: &str) -> bool {
    let mut in_quote: Option<char> = None;
    for ch in s.chars() {
        match in_quote {
            Some(q) if ch == q => in_quote = None,
            Some(_) => {}
            None if ch == '\'' || ch == '"' => in_quote = Some(ch),
            None => {}
        }
    }
    in_quote.is_some()
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
            // `A ? x : B ? y : z` chains in the else-branch.
            if_false: parse_conditional_type(if_false_str)
                .unwrap_or_else(|| parse_type_string(if_false_str)),
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

/// `Handler<R, Q>` -> `Handler`: a class-string names a class, not its type arguments.
fn strip_generic_args(name: &str) -> &str {
    match name.split_once('<') {
        Some((base, _))
            if name.ends_with('>')
                && !base.is_empty()
                && base
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '\\') =>
        {
            base
        }
        _ => name,
    }
}
