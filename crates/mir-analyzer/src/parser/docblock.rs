use mir_types::{Atomic, Union, Variance};
/// Docblock parser — delegates to `php_rs_parser::phpdoc` for tag extraction,
/// then converts `PhpDocTag`s into mir's `ParsedDocblock` with resolved types.
use std::sync::Arc;

use php_rs_parser::phpdoc::PhpDocTag;

// ---------------------------------------------------------------------------
// DocblockParser
// ---------------------------------------------------------------------------

pub struct DocblockParser;

impl DocblockParser {
    pub fn parse(text: &str) -> ParsedDocblock {
        let doc = php_rs_parser::phpdoc::parse(text);
        let mut result = ParsedDocblock {
            description: extract_description(text),
            ..Default::default()
        };

        for tag in &doc.tags {
            match tag {
                PhpDocTag::Param {
                    type_str: Some(ty_s),
                    name: Some(n),
                    ..
                } => {
                    result.params.push((
                        n.trim_start_matches('$').to_string(),
                        parse_type_string(ty_s),
                    ));
                }
                PhpDocTag::Return {
                    type_str: Some(ty_s),
                    ..
                } => {
                    result.return_type = Some(parse_type_string(ty_s));
                }
                PhpDocTag::Var { type_str, name, .. } => {
                    if let Some(ty_s) = type_str {
                        result.var_type = Some(parse_type_string(ty_s));
                    }
                    if let Some(n) = name {
                        result.var_name = Some(n.trim_start_matches('$').to_string());
                    }
                }
                PhpDocTag::Throws {
                    type_str: Some(ty_s),
                    ..
                } => {
                    let class = ty_s.split_whitespace().next().unwrap_or("").to_string();
                    if !class.is_empty() {
                        result.throws.push(class);
                    }
                }
                PhpDocTag::Deprecated { description } => {
                    result.is_deprecated = true;
                    result.deprecated = Some(
                        description
                            .as_ref()
                            .map(|d| d.to_string())
                            .unwrap_or_default(),
                    );
                }
                PhpDocTag::Template { name, bound } => {
                    result.templates.push((
                        name.to_string(),
                        bound.map(parse_type_string),
                        Variance::Invariant,
                    ));
                }
                PhpDocTag::TemplateCovariant { name, bound } => {
                    result.templates.push((
                        name.to_string(),
                        bound.map(parse_type_string),
                        Variance::Covariant,
                    ));
                }
                PhpDocTag::TemplateContravariant { name, bound } => {
                    result.templates.push((
                        name.to_string(),
                        bound.map(parse_type_string),
                        Variance::Contravariant,
                    ));
                }
                PhpDocTag::Extends { type_str } => {
                    result.extends = Some(parse_type_string(type_str));
                }
                PhpDocTag::Implements { type_str } => {
                    result.implements.push(parse_type_string(type_str));
                }
                PhpDocTag::Assert {
                    type_str: Some(ty_s),
                    name: Some(n),
                } => {
                    result.assertions.push((
                        n.trim_start_matches('$').to_string(),
                        parse_type_string(ty_s),
                    ));
                }
                PhpDocTag::Suppress { rules } => {
                    for rule in rules.split([',', ' ']) {
                        let rule = rule.trim().to_string();
                        if !rule.is_empty() {
                            result.suppressed_issues.push(rule);
                        }
                    }
                }
                PhpDocTag::See { reference } => result.see.push(reference.to_string()),
                PhpDocTag::Link { url } => result.see.push(url.to_string()),
                PhpDocTag::Mixin { class } => result.mixins.push(class.to_string()),
                PhpDocTag::Property {
                    type_str,
                    name: Some(n),
                    ..
                } => result.properties.push(DocProperty {
                    type_hint: type_str.unwrap_or("").to_string(),
                    name: n.trim_start_matches('$').to_string(),
                    read_only: false,
                    write_only: false,
                }),
                PhpDocTag::PropertyRead {
                    type_str,
                    name: Some(n),
                    ..
                } => result.properties.push(DocProperty {
                    type_hint: type_str.unwrap_or("").to_string(),
                    name: n.trim_start_matches('$').to_string(),
                    read_only: true,
                    write_only: false,
                }),
                PhpDocTag::PropertyWrite {
                    type_str,
                    name: Some(n),
                    ..
                } => result.properties.push(DocProperty {
                    type_hint: type_str.unwrap_or("").to_string(),
                    name: n.trim_start_matches('$').to_string(),
                    read_only: false,
                    write_only: true,
                }),
                PhpDocTag::Method { signature } => {
                    if let Some(m) = parse_method_line(signature) {
                        result.methods.push(m);
                    }
                }
                PhpDocTag::TypeAlias {
                    name: Some(n),
                    type_str,
                } => result.type_aliases.push(DocTypeAlias {
                    name: n.to_string(),
                    type_expr: type_str.unwrap_or("").to_string(),
                }),
                PhpDocTag::Internal => result.is_internal = true,
                PhpDocTag::Pure => result.is_pure = true,
                PhpDocTag::Immutable => result.is_immutable = true,
                PhpDocTag::Readonly => result.is_readonly = true,
                PhpDocTag::Generic { tag, body } => match *tag {
                    "api" | "psalm-api" => result.is_api = true,
                    "psalm-assert" | "phpstan-assert" => {
                        if let Some((ty_str, name)) = body.as_deref().and_then(parse_param_line) {
                            result.assertions.push((name, parse_type_string(&ty_str)));
                        }
                    }
                    "psalm-assert-if-true" | "phpstan-assert-if-true" => {
                        if let Some((ty_str, name)) = body.as_deref().and_then(parse_param_line) {
                            result
                                .assertions_if_true
                                .push((name, parse_type_string(&ty_str)));
                        }
                    }
                    "psalm-assert-if-false" | "phpstan-assert-if-false" => {
                        if let Some((ty_str, name)) = body.as_deref().and_then(parse_param_line) {
                            result
                                .assertions_if_false
                                .push((name, parse_type_string(&ty_str)));
                        }
                    }
                    "psalm-property" => {
                        if let Some((ty_str, name)) = body.as_deref().and_then(parse_param_line) {
                            result.properties.push(DocProperty {
                                type_hint: ty_str,
                                name,
                                read_only: false,
                                write_only: false,
                            });
                        }
                    }
                    "psalm-property-read" => {
                        if let Some((ty_str, name)) = body.as_deref().and_then(parse_param_line) {
                            result.properties.push(DocProperty {
                                type_hint: ty_str,
                                name,
                                read_only: true,
                                write_only: false,
                            });
                        }
                    }
                    "psalm-property-write" => {
                        if let Some((ty_str, name)) = body.as_deref().and_then(parse_param_line) {
                            result.properties.push(DocProperty {
                                type_hint: ty_str,
                                name,
                                read_only: false,
                                write_only: true,
                            });
                        }
                    }
                    "psalm-method" => {
                        if let Some(method) = body.as_deref().and_then(parse_method_line) {
                            result.methods.push(method);
                        }
                    }
                    _ => {}
                },
                _ => {}
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
    pub params: Vec<DocMethodParam>,
}

#[derive(Debug, Default, Clone)]
pub struct DocMethodParam {
    pub name: String,
    pub type_hint: String,
    pub is_variadic: bool,
    pub is_byref: bool,
    pub is_optional: bool,
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
    /// `@template T` / `@template T of Bound` / `@template-covariant T` / `@template-contravariant T`
    pub templates: Vec<(String, Option<Union>, Variance)>,
    /// `@extends ClassName<T>`
    pub extends: Option<Union>,
    /// `@implements InterfaceName<T>`
    pub implements: Vec<Union>,
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

    // Intersection: `A&B&C` — PHP 8.1+ pure intersection type
    if s.contains('&') && !is_inside_generics(s) {
        let parts: Vec<Union> = s.split('&').map(|p| parse_type_string(p.trim())).collect();
        return Union::single(Atomic::TIntersection { parts });
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

/// Extract the description text (all prose before the first `@` tag) from a raw docblock.
fn extract_description(text: &str) -> String {
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

/// Parse `[static] [ReturnType] name(...)` for @method tags.
fn parse_method_line(s: &str) -> Option<DocMethod> {
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

fn parse_method_params(name_part: &str) -> Vec<DocMethodParam> {
    let Some(open) = name_part.find('(') else {
        return vec![];
    };
    let Some(close) = name_part.rfind(')') else {
        return vec![];
    };
    let inner = name_part[open + 1..close].trim();
    if inner.is_empty() {
        return vec![];
    }

    split_generics(inner)
        .into_iter()
        .filter_map(|param| parse_method_param(&param))
        .collect()
}

fn parse_method_param(param: &str) -> Option<DocMethodParam> {
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
        assert_eq!(parsed.templates[0].2, Variance::Invariant);
    }

    #[test]
    fn parse_template_covariant() {
        let doc = "/** @template-covariant T */";
        let parsed = DocblockParser::parse(doc);
        assert_eq!(parsed.templates.len(), 1);
        assert_eq!(parsed.templates[0].0, "T");
        assert_eq!(parsed.templates[0].2, Variance::Covariant);
    }

    #[test]
    fn parse_template_contravariant() {
        let doc = "/** @template-contravariant T */";
        let parsed = DocblockParser::parse(doc);
        assert_eq!(parsed.templates.len(), 1);
        assert_eq!(parsed.templates[0].0, "T");
        assert_eq!(parsed.templates[0].2, Variance::Contravariant);
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

    #[test]
    fn parse_intersection_two_parts() {
        let u = parse_type_string("Iterator&Countable");
        assert_eq!(u.types.len(), 1);
        assert!(matches!(u.types[0], Atomic::TIntersection { ref parts } if parts.len() == 2));
        if let Atomic::TIntersection { parts } = &u.types[0] {
            assert!(parts[0].contains(
                |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Iterator")
            ));
            assert!(parts[1].contains(
                |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Countable")
            ));
        }
    }

    #[test]
    fn parse_intersection_three_parts() {
        let u = parse_type_string("Iterator&Countable&Stringable");
        assert_eq!(u.types.len(), 1);
        let Atomic::TIntersection { parts } = &u.types[0] else {
            panic!("expected TIntersection");
        };
        assert_eq!(parts.len(), 3);
        assert!(parts[0].contains(
            |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Iterator")
        ));
        assert!(parts[1].contains(
            |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Countable")
        ));
        assert!(parts[2].contains(
            |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Stringable")
        ));
    }

    #[test]
    fn parse_intersection_in_union_with_null() {
        let u = parse_type_string("Iterator&Countable|null");
        assert!(u.is_nullable());
        let intersection = u
            .types
            .iter()
            .find_map(|t| {
                if let Atomic::TIntersection { parts } = t {
                    Some(parts)
                } else {
                    None
                }
            })
            .expect("expected TIntersection");
        assert_eq!(intersection.len(), 2);
        assert!(intersection[0].contains(
            |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Iterator")
        ));
        assert!(intersection[1].contains(
            |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Countable")
        ));
    }

    #[test]
    fn parse_intersection_in_union_with_scalar() {
        let u = parse_type_string("Iterator&Countable|string");
        assert!(u.contains(|t| matches!(t, Atomic::TString)));
        let intersection = u
            .types
            .iter()
            .find_map(|t| {
                if let Atomic::TIntersection { parts } = t {
                    Some(parts)
                } else {
                    None
                }
            })
            .expect("expected TIntersection");
        assert!(intersection[0].contains(
            |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Iterator")
        ));
        assert!(intersection[1].contains(
            |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Countable")
        ));
    }
}
