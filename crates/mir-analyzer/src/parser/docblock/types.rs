use super::*;

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

    // Keywords
    match s.to_lowercase().as_str() {
        "string" => Type::single(Atomic::TString),
        "non-empty-string" => Type::single(Atomic::TNonEmptyString),
        "numeric-string" => Type::single(Atomic::TNumericString),
        "class-string" => Type::single(Atomic::TClassString(None)),
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
        "callable" => Type::single(Atomic::TCallable {
            params: None,
            return_type: None,
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
        properties,
        is_open,
        is_list,
    })
}

pub(super) fn parse_callable_syntax(s: &str) -> Option<Type> {
    let s = s.trim_start_matches('\\');
    let lower = s.to_lowercase();
    let is_closure = lower.starts_with("closure");
    let is_callable = lower.starts_with("callable");
    if !is_closure && !is_callable {
        return None;
    }
    let prefix_len = if is_closure {
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
    let params: Vec<mir_types::atomic::FnParam> = split_generics(params_str)
        .into_iter()
        .enumerate()
        .filter(|(_, p)| !p.trim().is_empty())
        .map(|(i, p)| {
            let p = p.trim();
            let (ty_str, name) = if let Some(dollar) = p.rfind('$') {
                (p[..dollar].trim(), p[dollar + 1..].to_string())
            } else {
                (p, format!("arg{i}"))
            };
            mir_types::atomic::FnParam {
                name: name.into(),
                ty: Some(mir_types::SimpleType::from_union(parse_type_string(ty_str))),
                default: None,
                is_variadic: false,
                is_byref: false,
                is_optional: false,
            }
        })
        .collect();
    if is_closure {
        Some(Type::single(Atomic::TClosure {
            params,
            return_type: return_type.unwrap_or_else(|| Box::new(Type::single(Atomic::TVoid))),
            this_type: None,
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
) -> Option<(String, Option<String>)> {
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
    if let Some((name, bound)) = body.split_once(" of ").or_else(|| body.split_once(" as ")) {
        let bound = bound.trim();
        Some((
            name.trim().to_string(),
            (!bound.is_empty()).then(|| bound.to_string()),
        ))
    } else {
        // No bound: take just the first whitespace-delimited token as the name.
        let name = body.split_whitespace().next().unwrap_or(body);
        Some((name.to_string(), None))
    }
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
    // Types can contain spaces (e.g., `array<string, int>`), so we need to find the variable name.
    // The variable name is the `$identifier` that comes after whitespace (not part of type syntax).
    //
    // Only examine the first line to avoid matching `$var` references in multi-line descriptions.
    let first_line = s.lines().next().unwrap_or(s);

    // Strategy: find the last sequence of whitespace followed by `$identifier` or `&$identifier`
    // on the first line. This handles both simple types and types with generics/spaces.
    let mut best_split: Option<(String, String)> = None;

    for (i, ch) in first_line.char_indices() {
        if ch.is_whitespace() {
            let after = first_line[i..].trim_start();
            // Accept `$name` or `&$name` (by-reference params in PHPDoc)
            let after_stripped = after.strip_prefix('&').unwrap_or(after);
            if after_stripped.starts_with('$') {
                let mut var_parts = after_stripped.split(char::is_whitespace);
                if let Some(name_with_dollar) = var_parts.next() {
                    let name = name_with_dollar.trim_start_matches('$').to_string();
                    if !name.is_empty() {
                        let type_part = first_line[..i].trim().to_string();
                        if !type_part.is_empty() {
                            best_split = Some((type_part, name));
                        }
                    }
                }
            }
        }
    }

    best_split
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

pub(super) fn is_inside_generics(s: &str) -> bool {
    let mut depth = 0i32;
    for ch in s.chars() {
        match ch {
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
    let is_pos = s.find(" is ")?;
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
    let after_is = s[is_pos + 4..].trim();
    let q_pos = find_char_at_depth(after_is, '?')?;
    let subject_str = after_is[..q_pos].trim();
    let rest = after_is[q_pos + 1..].trim();
    let colon_pos = find_char_at_depth(rest, ':')?;
    let true_str = rest[..colon_pos].trim();
    let false_str = rest[colon_pos + 1..].trim();
    Some(Type::single(Atomic::TConditional {
        param_name,
        subject: Box::new(parse_type_string(subject_str)),
        if_true: Box::new(parse_type_string(true_str)),
        if_false: Box::new(parse_type_string(false_str)),
    }))
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
