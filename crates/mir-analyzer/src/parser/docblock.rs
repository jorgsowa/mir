use mir_types::{Atomic, Union};
/// Docblock parser — extracts `@param`, `@return`, `@var`, `@template`,
/// `@extends`, `@implements`, `@throws`, `@psalm-*`, and other annotations.
use std::sync::Arc;

// ---------------------------------------------------------------------------
// DocblockParser
// ---------------------------------------------------------------------------

pub struct DocblockParser;

impl DocblockParser {
    pub fn parse(text: &str) -> ParsedDocblock {
        let mut result = ParsedDocblock::default();

        // --- Description pre-pass: collect text before the first `@` tag ---
        {
            let raw_lines = extract_lines(text);
            let mut desc_lines: Vec<String> = Vec::new();
            for l in &raw_lines {
                let l = l.trim();
                if l.starts_with('@') {
                    break;
                }
                if !l.is_empty() {
                    desc_lines.push(l.to_string());
                }
            }
            result.description = desc_lines.join(" ");
        }

        let lines = extract_lines(text);

        for line in lines {
            let line = line.trim();
            if line.is_empty() || !line.starts_with('@') {
                continue;
            }
            if let Some(rest) = line.strip_prefix("@param") {
                let rest = rest.trim();
                if let Some((ty_str, name)) = parse_param_line(rest) {
                    let ty = parse_type_string(&ty_str);
                    result.params.push((name, ty));
                }
            } else if let Some(rest) = strip_tag(line, "@return") {
                let ty = parse_type_string(extract_type_token(rest.trim()));
                result.return_type = Some(ty);
            } else if let Some(rest) = line
                .strip_prefix("@var")
                .or_else(|| line.strip_prefix("@psalm-var"))
                .or_else(|| line.strip_prefix("@phpstan-var"))
            {
                let rest = rest.trim();
                let type_str = extract_type_token(rest);
                let ty = parse_type_string(type_str);
                result.var_type = Some(ty);
                // Extract optional variable name: `@var Type $name`
                let after_type = rest[type_str.len()..].trim();
                if after_type.starts_with('$') {
                    result.var_name = Some(
                        after_type
                            .trim_start_matches('$')
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .to_string(),
                    );
                }
            } else if let Some(rest) =
                strip_tag(line, "@psalm-return").or_else(|| strip_tag(line, "@phpstan-return"))
            {
                let ty = parse_type_string(extract_type_token(rest.trim()));
                result.return_type = Some(ty); // @psalm-return / @phpstan-return overrides @return
            } else if let Some(rest) = line
                .strip_prefix("@psalm-param")
                .or_else(|| line.strip_prefix("@phpstan-param"))
            {
                let rest = rest.trim();
                if let Some((ty_str, name)) = parse_param_line(rest) {
                    let ty = parse_type_string(&ty_str);
                    // Override or add
                    if let Some(entry) = result.params.iter_mut().find(|(n, _)| *n == name) {
                        entry.1 = ty;
                    } else {
                        result.params.push((name, ty));
                    }
                }
            } else if let Some(rest) = line.strip_prefix("@template") {
                let rest = rest.trim();
                let (name, bound) = parse_template_line(rest);
                result.templates.push((name, bound));
            } else if let Some(rest) = line
                .strip_prefix("@extends")
                .or_else(|| line.strip_prefix("@psalm-extends"))
            {
                result.extends = Some(rest.trim().to_string());
            } else if let Some(rest) = line
                .strip_prefix("@implements")
                .or_else(|| line.strip_prefix("@psalm-implements"))
            {
                result.implements.push(rest.trim().to_string());
            } else if let Some(rest) = line.strip_prefix("@throws") {
                let class = rest.split_whitespace().next().unwrap_or("").to_string();
                if !class.is_empty() {
                    result.throws.push(class);
                }
            } else if let Some(rest) = line
                .strip_prefix("@psalm-assert-if-true")
                .or_else(|| line.strip_prefix("@phpstan-assert-if-true"))
            {
                let rest = rest.trim();
                if let Some((ty_str, name)) = parse_param_line(rest) {
                    result
                        .assertions_if_true
                        .push((name, parse_type_string(&ty_str)));
                }
            } else if let Some(rest) = line
                .strip_prefix("@psalm-assert-if-false")
                .or_else(|| line.strip_prefix("@phpstan-assert-if-false"))
            {
                let rest = rest.trim();
                if let Some((ty_str, name)) = parse_param_line(rest) {
                    result
                        .assertions_if_false
                        .push((name, parse_type_string(&ty_str)));
                }
            } else if let Some(rest) = line
                .strip_prefix("@psalm-assert")
                .or_else(|| line.strip_prefix("@phpstan-assert"))
            {
                let rest = rest.trim();
                if let Some((ty_str, name)) = parse_param_line(rest) {
                    result.assertions.push((name, parse_type_string(&ty_str)));
                }
            } else if line.contains("@psalm-suppress") || line.contains("@phpstan-ignore") {
                let suppressed = line.split_whitespace().nth(1).unwrap_or("").to_string();
                if !suppressed.is_empty() {
                    result.suppressed_issues.push(suppressed);
                }
            } else if let Some(rest) = line.strip_prefix("@deprecated") {
                result.is_deprecated = true;
                let msg = rest.trim().to_string();
                result.deprecated = Some(msg);
            } else if line.starts_with("@see") || line.starts_with("@link") {
                let rest = if let Some(r) = line.strip_prefix("@see") {
                    r
                } else {
                    line.strip_prefix("@link").unwrap_or("")
                };
                let s = rest.trim().to_string();
                if !s.is_empty() {
                    result.see.push(s);
                }
            } else if let Some(rest) = line.strip_prefix("@mixin") {
                let cls = rest.trim().to_string();
                if !cls.is_empty() {
                    result.mixins.push(cls);
                }
            } else if line.starts_with("@property-read") {
                if let Some(rest) = line.strip_prefix("@property-read") {
                    if let Some(prop) = parse_property_line(rest.trim(), true, false) {
                        result.properties.push(prop);
                    }
                }
            } else if line.starts_with("@property-write") {
                if let Some(rest) = line.strip_prefix("@property-write") {
                    if let Some(prop) = parse_property_line(rest.trim(), false, true) {
                        result.properties.push(prop);
                    }
                }
            } else if line.starts_with("@property") {
                if let Some(rest) = line.strip_prefix("@property") {
                    if let Some(prop) = parse_property_line(rest.trim(), false, false) {
                        result.properties.push(prop);
                    }
                }
            } else if let Some(rest) = line.strip_prefix("@method") {
                if let Some(m) = parse_method_line(rest.trim()) {
                    result.methods.push(m);
                }
            } else if let Some(rest) = line
                .strip_prefix("@psalm-type")
                .or_else(|| line.strip_prefix("@phpstan-type"))
            {
                if let Some(alias) = parse_type_alias_line(rest.trim()) {
                    result.type_aliases.push(alias);
                }
            } else if line.starts_with("@internal") {
                result.is_internal = true;
            } else if line.starts_with("@psalm-pure") || line.starts_with("@pure") {
                result.is_pure = true;
            } else if line.starts_with("@psalm-immutable") || line.starts_with("@immutable") {
                result.is_immutable = true;
            } else if line.starts_with("@readonly") {
                result.is_readonly = true;
            } else if line.starts_with("@api") || line.starts_with("@psalm-api") {
                result.is_api = true;
            }
        }

        result
    }
}

// ---------------------------------------------------------------------------
// ParsedDocblock support types
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct DocProperty {
    pub type_hint: String,
    pub name: String,     // without leading $
    pub read_only: bool,  // true for @property-read
    pub write_only: bool, // true for @property-write
}

#[derive(Debug, Default, Clone)]
pub struct DocMethod {
    pub return_type: String,
    pub name: String,
    pub is_static: bool,
}

#[derive(Debug, Default, Clone)]
pub struct DocTypeAlias {
    pub name: String,
    pub type_expr: String,
}

// ---------------------------------------------------------------------------
// ParsedDocblock
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct ParsedDocblock {
    /// `@param Type $name`
    pub params: Vec<(String, Union)>,
    /// `@return Type`
    pub return_type: Option<Union>,
    /// `@var Type` or `@var Type $name` — type and optional variable name
    pub var_type: Option<Union>,
    /// Optional variable name from `@var Type $name`
    pub var_name: Option<String>,
    /// `@template T` / `@template T of Bound`
    pub templates: Vec<(String, Option<Union>)>,
    /// `@extends ClassName<T>`
    pub extends: Option<String>,
    /// `@implements InterfaceName<T>`
    pub implements: Vec<String>,
    /// `@throws ClassName`
    pub throws: Vec<String>,
    /// `@psalm-assert Type $var`
    pub assertions: Vec<(String, Union)>,
    /// `@psalm-assert-if-true Type $var`
    pub assertions_if_true: Vec<(String, Union)>,
    /// `@psalm-assert-if-false Type $var`
    pub assertions_if_false: Vec<(String, Union)>,
    /// `@psalm-suppress IssueName`
    pub suppressed_issues: Vec<String>,
    pub is_deprecated: bool,
    pub is_internal: bool,
    pub is_pure: bool,
    pub is_immutable: bool,
    pub is_readonly: bool,
    pub is_api: bool,
    /// Free text before first `@` tag — used for hover display
    pub description: String,
    /// `@deprecated message` — Some(message) or Some("") if no message
    pub deprecated: Option<String>,
    /// `@see ClassName` / `@link URL`
    pub see: Vec<String>,
    /// `@mixin ClassName`
    pub mixins: Vec<String>,
    /// `@property`, `@property-read`, `@property-write`
    pub properties: Vec<DocProperty>,
    /// `@method [static] ReturnType name([params])`
    pub methods: Vec<DocMethod>,
    /// `@psalm-type Alias = TypeExpr` / `@phpstan-type Alias = TypeExpr`
    pub type_aliases: Vec<DocTypeAlias>,
}

impl ParsedDocblock {
    /// Returns the type for a given parameter name (strips leading `$`).
    pub fn get_param_type(&self, name: &str) -> Option<&Union> {
        let name = name.trim_start_matches('$');
        self.params
            .iter()
            .find(|(n, _)| n.trim_start_matches('$') == name)
            .map(|(_, ty)| ty)
    }
}

// ---------------------------------------------------------------------------
// Type string parser
// ---------------------------------------------------------------------------

/// Parse a PHPDoc type expression string into a `Union`.
/// Handles: `string`, `int|null`, `array<string>`, `list<int>`,
/// `ClassName`, `?string` (nullable), `string[]` (array shorthand).
pub fn parse_type_string(s: &str) -> Union {
    let s = s.trim();

    // Nullable shorthand: `?Type`
    if let Some(inner) = s.strip_prefix('?') {
        let inner_ty = parse_type_string(inner);
        let mut u = inner_ty;
        u.add_type(Atomic::TNull);
        return u;
    }

    // Union: `A|B|C`
    if s.contains('|') && !is_inside_generics(s) {
        let parts = split_union(s);
        if parts.len() > 1 {
            let mut u = Union::empty();
            for part in parts {
                for atomic in parse_type_string(&part).types {
                    u.add_type(atomic);
                }
            }
            return u;
        }
    }

    // Intersection: `A&B` (simplified — treat as first type for now)
    if s.contains('&') && !is_inside_generics(s) {
        let first = s.split('&').next().unwrap_or(s);
        return parse_type_string(first.trim());
    }

    // Array shorthand: `Type[]` or `Type[][]`
    if let Some(value_str) = s.strip_suffix("[]") {
        let value = parse_type_string(value_str);
        return Union::single(Atomic::TArray {
            key: Box::new(Union::single(Atomic::TInt)),
            value: Box::new(value),
        });
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
        "string" => Union::single(Atomic::TString),
        "non-empty-string" => Union::single(Atomic::TNonEmptyString),
        "numeric-string" => Union::single(Atomic::TNumericString),
        "class-string" => Union::single(Atomic::TClassString(None)),
        "int" | "integer" => Union::single(Atomic::TInt),
        "positive-int" => Union::single(Atomic::TPositiveInt),
        "negative-int" => Union::single(Atomic::TNegativeInt),
        "non-negative-int" => Union::single(Atomic::TNonNegativeInt),
        "float" | "double" => Union::single(Atomic::TFloat),
        "bool" | "boolean" => Union::single(Atomic::TBool),
        "true" => Union::single(Atomic::TTrue),
        "false" => Union::single(Atomic::TFalse),
        "null" => Union::single(Atomic::TNull),
        "void" => Union::single(Atomic::TVoid),
        "never" | "never-return" | "no-return" | "never-returns" => Union::single(Atomic::TNever),
        "mixed" => Union::single(Atomic::TMixed),
        "object" => Union::single(Atomic::TObject),
        "array" => Union::single(Atomic::TArray {
            key: Box::new(Union::single(Atomic::TMixed)),
            value: Box::new(Union::mixed()),
        }),
        "list" => Union::single(Atomic::TList {
            value: Box::new(Union::mixed()),
        }),
        "callable" => Union::single(Atomic::TCallable {
            params: None,
            return_type: None,
        }),
        "iterable" => Union::single(Atomic::TArray {
            key: Box::new(Union::single(Atomic::TMixed)),
            value: Box::new(Union::mixed()),
        }),
        "scalar" => Union::single(Atomic::TScalar),
        "numeric" => Union::single(Atomic::TNumeric),
        "resource" => Union::mixed(), // treat as mixed
        // self/static/parent: emit sentinel with empty FQCN; collector fills it in.
        "static" => Union::single(Atomic::TStaticObject {
            fqcn: Arc::from(""),
        }),
        "self" | "$this" => Union::single(Atomic::TSelf {
            fqcn: Arc::from(""),
        }),
        "parent" => Union::single(Atomic::TParent {
            fqcn: Arc::from(""),
        }),

        // Named class
        _ if !s.is_empty()
            && s.chars()
                .next()
                .map(|c| c.is_alphanumeric() || c == '\\' || c == '_')
                .unwrap_or(false) =>
        {
            Union::single(Atomic::TNamedObject {
                fqcn: normalize_fqcn(s).into(),
                type_params: vec![],
            })
        }

        _ => Union::mixed(),
    }
}

fn parse_generic(name: &str, inner: &str) -> Union {
    match name.to_lowercase().as_str() {
        "array" => {
            let params = split_generics(inner);
            let (key, value) = if params.len() >= 2 {
                (
                    parse_type_string(params[0].trim()),
                    parse_type_string(params[1].trim()),
                )
            } else {
                (
                    Union::single(Atomic::TInt),
                    parse_type_string(params[0].trim()),
                )
            };
            Union::single(Atomic::TArray {
                key: Box::new(key),
                value: Box::new(value),
            })
        }
        "list" | "non-empty-list" => {
            let value = parse_type_string(inner.trim());
            if name.to_lowercase().starts_with("non-empty") {
                Union::single(Atomic::TNonEmptyList {
                    value: Box::new(value),
                })
            } else {
                Union::single(Atomic::TList {
                    value: Box::new(value),
                })
            }
        }
        "non-empty-array" => {
            let params = split_generics(inner);
            let (key, value) = if params.len() >= 2 {
                (
                    parse_type_string(params[0].trim()),
                    parse_type_string(params[1].trim()),
                )
            } else {
                (
                    Union::single(Atomic::TInt),
                    parse_type_string(params[0].trim()),
                )
            };
            Union::single(Atomic::TNonEmptyArray {
                key: Box::new(key),
                value: Box::new(value),
            })
        }
        "iterable" => {
            let params = split_generics(inner);
            let value = if params.len() >= 2 {
                parse_type_string(params[1].trim())
            } else {
                parse_type_string(params[0].trim())
            };
            Union::single(Atomic::TArray {
                key: Box::new(Union::single(Atomic::TMixed)),
                value: Box::new(value),
            })
        }
        "class-string" => Union::single(Atomic::TClassString(Some(
            normalize_fqcn(inner.trim()).into(),
        ))),
        "int" => {
            // int<min, max>
            Union::single(Atomic::TIntRange {
                min: None,
                max: None,
            })
        }
        // Named class with type params
        _ => {
            let params: Vec<Union> = split_generics(inner)
                .iter()
                .map(|p| parse_type_string(p.trim()))
                .collect();
            Union::single(Atomic::TNamedObject {
                fqcn: normalize_fqcn(name).into(),
                type_params: params,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Strip a tag prefix, requiring it be followed by whitespace or end of string.
/// Also handles plural forms like `@returns` when tag is `@return`.
fn strip_tag<'a>(line: &'a str, tag: &str) -> Option<&'a str> {
    // Try exact tag + whitespace boundary
    if let Some(rest) = line.strip_prefix(tag) {
        if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t') {
            return Some(rest);
        }
        // Allow plural form: "@return" matches "@returns "
        if let Some(after_s) = rest.strip_prefix('s') {
            if after_s.is_empty() || after_s.starts_with(' ') || after_s.starts_with('\t') {
                return Some(after_s);
            }
        }
    }
    None
}

/// Extract only the type token from a docblock annotation line, stopping before the description.
/// For example: `string|null The description here` → `string|null`
///              `array<string, int> Some description` → `array<string, int>`
fn extract_type_token(s: &str) -> &str {
    let mut depth = 0i32;
    let mut end = s.len();
    let chars: Vec<(usize, char)> = s.char_indices().collect();
    let mut i = 0;
    while i < chars.len() {
        let (byte_pos, ch) = chars[i];
        match ch {
            '<' | '(' | '{' => depth += 1,
            '>' | ')' | '}' => depth -= 1,
            ' ' | '\t' if depth == 0 => {
                end = byte_pos;
                break;
            }
            _ => {}
        }
        i += 1;
    }
    &s[..end]
}

fn extract_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(|l| {
            // Strip `/**`, `*/`, leading `*`
            let l = l.trim();
            let l = l.trim_start_matches("/**").trim();
            // Strip trailing `*/` (handles single-line `/** @return int */`)
            let l = l.trim_end_matches("*/").trim();
            let l = l.trim_start_matches("*/").trim();
            let l = if let Some(stripped) = l.strip_prefix("* ") {
                stripped
            } else {
                l.trim_start_matches('*')
            };
            l.trim().to_string()
        })
        .collect()
}

fn parse_param_line(s: &str) -> Option<(String, String)> {
    // Formats: `Type $name`, `Type $name description`
    let mut parts = s.splitn(3, char::is_whitespace);
    let ty = parts.next()?.trim().to_string();
    let name = parts.next()?.trim().trim_start_matches('$').to_string();
    if ty.is_empty() || name.is_empty() {
        return None;
    }
    Some((ty, name))
}

fn parse_template_line(s: &str) -> (String, Option<Union>) {
    // `T` or `T of Bound`
    let mut parts = s.splitn(3, char::is_whitespace);
    let name = parts.next().unwrap_or("").trim().to_string();
    let of_keyword = parts.next().unwrap_or("").trim().to_lowercase();
    let bound = if of_keyword == "of" {
        parts.next().map(|b| parse_type_string(b.trim()))
    } else {
        None
    };
    (name, bound)
}

fn split_union(s: &str) -> Vec<String> {
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

fn split_generics(s: &str) -> Vec<String> {
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

fn is_inside_generics(s: &str) -> bool {
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

fn normalize_fqcn(s: &str) -> String {
    // Strip leading backslash if present — we normalize all FQCNs without leading `\`
    s.trim_start_matches('\\').to_string()
}

/// Parse `[Type] $name [description]` for @property tags.
fn parse_property_line(s: &str, read_only: bool, write_only: bool) -> Option<DocProperty> {
    // s has already had the tag prefix stripped and trimmed
    // Formats: `$name`, `Type $name`, `Type $name description`
    let mut words = s.splitn(3, char::is_whitespace);
    let first = words.next()?.trim();
    if first.is_empty() {
        return None;
    }
    if first.starts_with('$') {
        // No type hint given
        Some(DocProperty {
            type_hint: String::new(),
            name: first.trim_start_matches('$').to_string(),
            read_only,
            write_only,
        })
    } else {
        // first word is the type hint
        let type_hint = first.to_string();
        let name_word = words.next()?.trim();
        if name_word.is_empty() {
            return None;
        }
        Some(DocProperty {
            type_hint,
            name: name_word.trim_start_matches('$').to_string(),
            read_only,
            write_only,
        })
    }
}

/// Parse `[static] [ReturnType] name(...)` for @method tags.
fn parse_method_line(s: &str) -> Option<DocMethod> {
    let mut words = s.splitn(4, char::is_whitespace);
    let first = words.next()?.trim();
    if first.is_empty() {
        return None;
    }
    let is_static = first.eq_ignore_ascii_case("static");
    let (return_type, name_part) = if is_static {
        let ret = words.next()?.trim().to_string();
        let nm = words.next()?.trim().to_string();
        (ret, nm)
    } else {
        // Check if next token looks like a method name (contains '(')
        let second = words
            .next()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        if second.is_empty() {
            // Only one word — treat as name with no return type
            let name = first.split('(').next().unwrap_or(first).to_string();
            return Some(DocMethod {
                return_type: String::new(),
                name,
                is_static: false,
            });
        }
        if first.contains('(') {
            // first word is `name(...)`, no return type
            let name = first.split('(').next().unwrap_or(first).to_string();
            return Some(DocMethod {
                return_type: String::new(),
                name,
                is_static: false,
            });
        }
        (first.to_string(), second)
    };
    let name = name_part
        .split('(')
        .next()
        .unwrap_or(&name_part)
        .to_string();
    Some(DocMethod {
        return_type,
        name,
        is_static,
    })
}

/// Parse `Alias = TypeExpr` for @psalm-type / @phpstan-type tags.
fn parse_type_alias_line(s: &str) -> Option<DocTypeAlias> {
    let (name_part, type_part) = if let Some(eq_pos) = s.find('=') {
        (&s[..eq_pos], &s[eq_pos + 1..])
    } else {
        // No `=` — just a name, no type expression
        return Some(DocTypeAlias {
            name: s.trim().to_string(),
            type_expr: String::new(),
        });
    };
    let name = name_part.trim().to_string();
    let type_expr = type_part.trim().to_string();
    if name.is_empty() {
        return None;
    }
    Some(DocTypeAlias { name, type_expr })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use mir_types::Atomic;

    #[test]
    fn parse_string() {
        let u = parse_type_string("string");
        assert_eq!(u.types.len(), 1);
        assert!(matches!(u.types[0], Atomic::TString));
    }

    #[test]
    fn parse_nullable_string() {
        let u = parse_type_string("?string");
        assert!(u.is_nullable());
        assert!(u.contains(|t| matches!(t, Atomic::TString)));
    }

    #[test]
    fn parse_union() {
        let u = parse_type_string("string|int|null");
        assert!(u.contains(|t| matches!(t, Atomic::TString)));
        assert!(u.contains(|t| matches!(t, Atomic::TInt)));
        assert!(u.is_nullable());
    }

    #[test]
    fn parse_array_of_string() {
        let u = parse_type_string("array<string>");
        assert!(u.contains(|t| matches!(t, Atomic::TArray { .. })));
    }

    #[test]
    fn parse_list_of_int() {
        let u = parse_type_string("list<int>");
        assert!(u.contains(|t| matches!(t, Atomic::TList { .. })));
    }

    #[test]
    fn parse_named_class() {
        let u = parse_type_string("Foo\\Bar");
        assert!(u.contains(
            |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Foo\\Bar")
        ));
    }

    #[test]
    fn parse_docblock_param_return() {
        let doc = r#"/**
         * @param string $name
         * @param int $age
         * @return bool
         */"#;
        let parsed = DocblockParser::parse(doc);
        assert_eq!(parsed.params.len(), 2);
        assert!(parsed.return_type.is_some());
        let ret = parsed.return_type.unwrap();
        assert!(ret.contains(|t| matches!(t, Atomic::TBool)));
    }

    #[test]
    fn parse_template() {
        let doc = "/** @template T of object */";
        let parsed = DocblockParser::parse(doc);
        assert_eq!(parsed.templates.len(), 1);
        assert_eq!(parsed.templates[0].0, "T");
        assert!(parsed.templates[0].1.is_some());
    }

    #[test]
    fn parse_deprecated() {
        let doc = "/** @deprecated use newMethod() instead */";
        let parsed = DocblockParser::parse(doc);
        assert!(parsed.is_deprecated);
        assert_eq!(
            parsed.deprecated.as_deref(),
            Some("use newMethod() instead")
        );
    }

    #[test]
    fn parse_description() {
        let doc = r#"/**
         * This is a description.
         * Spans two lines.
         * @param string $x
         */"#;
        let parsed = DocblockParser::parse(doc);
        assert!(parsed.description.contains("This is a description"));
        assert!(parsed.description.contains("Spans two lines"));
    }

    #[test]
    fn parse_see_and_link() {
        let doc = "/** @see SomeClass\n * @link https://example.com */";
        let parsed = DocblockParser::parse(doc);
        assert_eq!(parsed.see.len(), 2);
        assert!(parsed.see.contains(&"SomeClass".to_string()));
        assert!(parsed.see.contains(&"https://example.com".to_string()));
    }

    #[test]
    fn parse_mixin() {
        let doc = "/** @mixin SomeTrait */";
        let parsed = DocblockParser::parse(doc);
        assert_eq!(parsed.mixins, vec!["SomeTrait".to_string()]);
    }

    #[test]
    fn parse_property_tags() {
        let doc = r#"/**
         * @property string $name
         * @property-read int $id
         * @property-write bool $active
         */"#;
        let parsed = DocblockParser::parse(doc);
        assert_eq!(parsed.properties.len(), 3);
        let name_prop = parsed.properties.iter().find(|p| p.name == "name").unwrap();
        assert_eq!(name_prop.type_hint, "string");
        assert!(!name_prop.read_only);
        assert!(!name_prop.write_only);
        let id_prop = parsed.properties.iter().find(|p| p.name == "id").unwrap();
        assert!(id_prop.read_only);
        let active_prop = parsed
            .properties
            .iter()
            .find(|p| p.name == "active")
            .unwrap();
        assert!(active_prop.write_only);
    }

    #[test]
    fn parse_method_tag() {
        let doc = r#"/**
         * @method string getName()
         * @method static int create()
         */"#;
        let parsed = DocblockParser::parse(doc);
        assert_eq!(parsed.methods.len(), 2);
        let get_name = parsed.methods.iter().find(|m| m.name == "getName").unwrap();
        assert_eq!(get_name.return_type, "string");
        assert!(!get_name.is_static);
        let create = parsed.methods.iter().find(|m| m.name == "create").unwrap();
        assert!(create.is_static);
    }

    #[test]
    fn parse_type_alias_tag() {
        let doc = "/** @psalm-type MyAlias = string|int */";
        let parsed = DocblockParser::parse(doc);
        assert_eq!(parsed.type_aliases.len(), 1);
        assert_eq!(parsed.type_aliases[0].name, "MyAlias");
        assert_eq!(parsed.type_aliases[0].type_expr, "string|int");
    }
}
