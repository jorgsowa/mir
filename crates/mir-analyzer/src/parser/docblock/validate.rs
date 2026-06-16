use super::*;

pub(super) fn normalize_fqcn(s: &str) -> String {
    // Preserve a leading `\` — it signals an absolute FQCN and must survive into
    // resolve_type_name so that use-alias resolution is NOT applied to it.
    // Without the `\`, `\Carbon\CarbonImmutable` would be mis-resolved to
    // `Illuminate\Support\Carbon\CarbonImmutable` when the file imports
    // `use Illuminate\Support\Carbon`.  resolve_type_name strips the `\` after
    // confirming the name is already absolute, so the final stored FQCN is clean.
    s.to_string()
}

/// Returns an error message if `s` is a malformed PHPDoc type expression, otherwise `None`.
///
/// Detects:
/// - unclosed generics (`array<`, `Foo<Bar`)
/// - `$variable` in type position (only `$this` is valid)
pub(super) fn validate_type_str(s: &str, tag: &str) -> Option<String> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if is_inside_generics(s) {
        return Some(format!("@{tag} has unclosed generic type `{s}`"));
    }
    // Skip empty generics check for callable/closure types (e.g., `callable(): T`, `\Closure(): T`)
    let is_callable_type = s.to_lowercase().contains("callable") || s.contains("Closure");
    if !is_callable_type && has_empty_generics(s) {
        return Some(format!("@{tag} has empty generic type parameter in `{s}`"));
    }
    for part in split_union(s) {
        let p = part.trim();
        if p.starts_with('$') && p != "$this" {
            return Some(format!("@{tag} contains variable `{p}` in type position"));
        }
        if let Some(err) = validate_generic_semantics(p, tag) {
            return Some(err);
        }
    }
    None
}

/// Validates semantic constraints on generic type expressions like `int<min, max>` and `array<key, value>`.
pub(super) fn validate_generic_semantics(s: &str, tag: &str) -> Option<String> {
    let lower = s.to_lowercase();
    let (name, inner) = extract_generic_content(s)?;
    match lower[..name.len()].as_ref() {
        "int" => validate_int_range_inner(inner, tag),
        "array" | "non-empty-array" => validate_array_key_inner(inner, tag),
        _ => None,
    }
}

/// Extracts `(name, inner)` from `Name<inner>`. Returns `None` if not a generic.
pub(super) fn extract_generic_content(s: &str) -> Option<(&str, &str)> {
    let lt = s.find('<')?;
    let name = s[..lt].trim();
    if name.is_empty() {
        return None;
    }
    let after_lt = &s[lt + 1..];
    let mut depth = 1i32;
    for (i, ch) in after_lt.char_indices() {
        match ch {
            '<' | '(' | '{' => depth += 1,
            '>' | ')' | '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((name, &after_lt[..i]));
                }
            }
            _ => {}
        }
    }
    None
}

pub(super) fn validate_int_range_inner(inner: &str, tag: &str) -> Option<String> {
    let mut parts = inner.splitn(2, ',');
    let min_str = parts.next()?.trim();
    let max_str = parts.next()?.trim();

    if min_str == "max" {
        return Some(format!(
            "@{tag} has invalid int range: `max` must be the second argument, not the first"
        ));
    }
    if max_str == "min" {
        return Some(format!(
            "@{tag} has invalid int range: `min` must be the first argument, not the second"
        ));
    }

    let is_valid_bound = |s: &str| s == "min" || s == "max" || s.parse::<i64>().is_ok();

    if !is_valid_bound(min_str) {
        return Some(format!(
            "@{tag} has invalid int range boundary `{min_str}`: must be an integer literal, `min`, or `max`"
        ));
    }
    if !is_valid_bound(max_str) {
        return Some(format!(
            "@{tag} has invalid int range boundary `{max_str}`: must be an integer literal, `min`, or `max`"
        ));
    }

    if let (Ok(lo), Ok(hi)) = (min_str.parse::<i64>(), max_str.parse::<i64>()) {
        if lo > hi {
            return Some(format!(
                "@{tag} has invalid int range: min ({lo}) must not be greater than max ({hi})"
            ));
        }
    }
    None
}

pub(super) fn validate_array_key_inner(inner: &str, tag: &str) -> Option<String> {
    let params = split_generics(inner);
    if params.len() < 2 {
        return None;
    }
    let key_str = params[0].trim();
    // Only flag types that are fundamentally invalid as PHP array keys (float, bool variants).
    // Reference types like `object` are technically invalid array keys in PHP but are
    // left to the template-bound checker (InvalidTemplateParam) to handle more precisely.
    let invalid_key_types = ["float", "bool", "true", "false"];
    if invalid_key_types.contains(&key_str.to_lowercase().as_str()) {
        return Some(format!(
            "@{tag} has invalid array key type `{key_str}`: must be a subtype of int|string"
        ));
    }
    None
}

pub(super) fn has_empty_generics(s: &str) -> bool {
    let mut depth = 0;
    let mut prev_open = false;
    for ch in s.chars() {
        match ch {
            '<' | '(' | '{' => {
                if prev_open && depth == 0 {
                    return true;
                }
                prev_open = true;
                depth += 1;
            }
            '>' | ')' | '}' => {
                depth -= 1;
                if depth == 0 {
                    if prev_open {
                        return true;
                    }
                    prev_open = false;
                }
            }
            c if !c.is_whitespace() => {
                prev_open = false;
            }
            _ => {}
        }
    }
    false
}

/// Validate `@method` body for common errors before parsing.
/// Returns `Some(error_message)` if the annotation is invalid.
pub(super) fn validate_method_body(s: &str) -> Option<String> {
    let s = s.trim();
    if s.is_empty() {
        return Some("@method annotation is missing a method definition".to_string());
    }
    // Strip optional `static` prefix
    let rest = if s.to_lowercase().starts_with("static ") {
        s["static ".len()..].trim_start()
    } else {
        s
    };
    // Extract the method name (the token immediately before `(`)
    let open = rest.find('(').unwrap_or(rest.len());
    let prefix = rest[..open].trim();
    let parts: Vec<&str> = prefix.split_whitespace().collect();
    let name = parts.last().unwrap_or(&"");
    // Check for invalid characters in method name (e.g., dash)
    if !name.is_empty() && !is_valid_php_identifier(name) {
        return Some(format!(
            "@method has invalid method name `{name}`: must be a valid PHP identifier"
        ));
    }
    // Validate parameters for by-ref annotations
    if rest.contains('(') {
        let params_str = rest;
        let open_pos = params_str.find('(').unwrap();
        let after_open = &params_str[open_pos + 1..];
        if let Some(rel_close) = find_matching_paren(&params_str[open_pos..]) {
            let close_pos = open_pos + rel_close;
            let inner = params_str[open_pos + 1..close_pos].trim();
            if !inner.is_empty() {
                for param in split_generics(inner) {
                    let param = param.trim();
                    if param.starts_with('&') {
                        return Some(format!(
                            "@method parameter `{param}` uses by-reference (`&`) which is not supported in @method annotations"
                        ));
                    }
                    // Detect `type & $name` pattern (ampersand with space before `$`)
                    if let Some(amp_pos) = param.find('&') {
                        let before_amp = &param[..amp_pos];
                        let after_amp = param[amp_pos + 1..].trim_start();
                        if !before_amp.trim().is_empty() && after_amp.starts_with('$') {
                            return Some(format!(
                                "@method parameter `{param}` uses by-reference (`&`) which is not supported in @method annotations"
                            ));
                        }
                    }
                }
            }
        } else {
            let _ = after_open;
        }
    }
    None
}

pub(super) fn is_valid_php_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Parse `[static] [ReturnType] name(...)` for @method tags.
pub(super) fn parse_method_line(s: &str) -> Option<DocMethod> {
    let mut rest = s.trim();
    if rest.is_empty() {
        return None;
    }
    let is_static = rest
        .split_whitespace()
        .next()
        .map(|w| w.eq_ignore_ascii_case("static"))
        .unwrap_or(false);
    if is_static {
        rest = rest["static".len()..].trim_start();
    }

    let open = rest.find('(').unwrap_or(rest.len());
    let prefix = rest[..open].trim();
    let mut parts: Vec<&str> = prefix.split_whitespace().collect();
    let name = parts.pop()?.to_string();
    if name.is_empty() {
        return None;
    }
    let return_type = parts.join(" ");
    Some(DocMethod {
        return_type,
        name,
        is_static,
        params: parse_method_params(rest),
    })
}

pub(super) fn parse_method_params(name_part: &str) -> Vec<DocMethodParam> {
    let Some(open) = name_part.find('(') else {
        return vec![];
    };
    // Use the existing balanced-paren matcher, which expects the slice to start
    // at '('. This avoids capturing closing parens from description text that
    // follows the method signature (e.g. Carbon's "@method addDay() ... (desc)").
    let Some(rel_close) = find_matching_paren(&name_part[open..]) else {
        return vec![];
    };
    let close = open + rel_close;
    let inner = name_part[open + 1..close].trim();
    if inner.is_empty() {
        return vec![];
    }

    split_generics(inner)
        .into_iter()
        .filter_map(|param| parse_method_param(&param))
        .collect()
}

pub(super) fn parse_method_param(param: &str) -> Option<DocMethodParam> {
    let before_default = param.split('=').next()?.trim();
    let is_optional = param.contains('=');
    let mut tokens: Vec<&str> = before_default.split_whitespace().collect();
    let raw_name = tokens.pop()?;
    let is_variadic = raw_name.contains("...");
    let is_byref = raw_name.contains('&');
    let name = raw_name
        .trim_start_matches('&')
        .trim_start_matches("...")
        .trim_start_matches('&')
        .trim_start_matches('$')
        .to_string();
    if name.is_empty() {
        return None;
    }
    Some(DocMethodParam {
        name,
        type_hint: tokens.join(" "),
        is_variadic,
        is_byref,
        is_optional: is_optional || is_variadic,
    })
}
