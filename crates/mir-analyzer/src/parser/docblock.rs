use mir_types::{ArrayKey, Atomic, Type, Variance};
/// Docblock parser — delegates to `phpdoc_parser` for tag extraction,
/// then converts tags into mir's `ParsedDocblock` with resolved types.
use std::sync::Arc;

use indexmap::IndexMap;
use phpdoc_parser::{body_text, parse as parse_phpdoc};

// ---------------------------------------------------------------------------
// DocblockParser
// ---------------------------------------------------------------------------

pub struct DocblockParser;

impl DocblockParser {
    pub fn parse(text: &str) -> ParsedDocblock {
        let doc = parse_phpdoc(text);
        let mut result = ParsedDocblock {
            description: extract_description(text),
            ..Default::default()
        };

        for tag in &doc.tags {
            match tag.name.as_str() {
                "param" | "psalm-param" | "phpstan-param" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        if let Some((ty_s, name)) = parse_param_line(&body_str) {
                            // Check if the parsed type is valid
                            if is_inside_generics(&ty_s) {
                                // For unclosed generics, report the full body for context
                                if let Some(msg) = validate_type_str(&body_str, "param") {
                                    result.invalid_annotations.push(msg);
                                }
                            } else if let Some(msg) = validate_type_str(&ty_s, "param") {
                                // For other errors, report the parsed type
                                result.invalid_annotations.push(msg);
                            } else {
                                result.params.push((
                                    name.trim_start_matches('$').to_string(),
                                    parse_type_string(&ty_s),
                                ));
                            }
                        } else if let Some(msg) = validate_type_str(&body_str, "param") {
                            // If parsing failed, validate the full body to provide better error context
                            result.invalid_annotations.push(msg);
                        }
                    }
                }
                "return" | "psalm-return" | "phpstan-return" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        let ty_s = extract_return_type(&body_str);
                        if let Some(msg) = validate_type_str(&ty_s, "return") {
                            result.invalid_annotations.push(msg);
                        }
                        result.return_type = Some(parse_type_string(&ty_s));
                    }
                }
                "var" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        if let Some((ty_s, name)) = parse_param_line(&body_str) {
                            if let Some(msg) = validate_type_str(&ty_s, "var") {
                                result.invalid_annotations.push(msg);
                            }
                            result.var_type = Some(parse_type_string(&ty_s));
                            result.var_name = Some(name.trim_start_matches('$').to_string());
                        } else {
                            // Spaces inside PHP types only appear within <…> generics.
                            // Stop at top-level whitespace to exclude description text that
                            // follows the type in multi-line @var bodies.
                            let ty_s = extract_type_prefix(body_str.trim());
                            if let Some(msg) = validate_type_str(ty_s, "var") {
                                result.invalid_annotations.push(msg);
                            }
                            result.var_type = Some(parse_type_string(ty_s));
                        }
                    }
                }
                "throws" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        let class = body_str.split_whitespace().next().unwrap_or("").to_string();
                        if !class.is_empty() {
                            result.throws.push(class);
                        }
                    }
                }
                "deprecated" => {
                    result.is_deprecated = true;
                    result.deprecated = Some(body_text(&tag.body).unwrap_or_default().to_string());
                }
                "template" => {
                    if let Some((name, bound)) =
                        parse_template_line(tag.name.as_str(), body_text(&tag.body))
                    {
                        if let Some(b) = &bound {
                            if let Some(msg) = validate_type_str(b, "template") {
                                result.invalid_annotations.push(msg);
                            }
                        }
                        result.templates.push((
                            name,
                            bound.map(|b| parse_type_string(&b)),
                            Variance::Invariant,
                        ));
                    }
                }
                "template-covariant" => {
                    if let Some((name, bound)) =
                        parse_template_line(tag.name.as_str(), body_text(&tag.body))
                    {
                        if let Some(b) = &bound {
                            if let Some(msg) = validate_type_str(b, "template-covariant") {
                                result.invalid_annotations.push(msg);
                            }
                        }
                        result.templates.push((
                            name,
                            bound.map(|b| parse_type_string(&b)),
                            Variance::Covariant,
                        ));
                    }
                }
                "template-contravariant" => {
                    if let Some((name, bound)) =
                        parse_template_line(tag.name.as_str(), body_text(&tag.body))
                    {
                        if let Some(b) = &bound {
                            if let Some(msg) = validate_type_str(b, "template-contravariant") {
                                result.invalid_annotations.push(msg);
                            }
                        }
                        result.templates.push((
                            name,
                            bound.map(|b| parse_type_string(&b)),
                            Variance::Contravariant,
                        ));
                    }
                }
                "extends" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        result.extends = Some(parse_type_string(body_str.trim()));
                    }
                }
                "implements" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        result.implements.push(parse_type_string(body_str.trim()));
                    }
                }
                "assert" | "psalm-assert" | "phpstan-assert" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        if let Some((ty_str, name)) = parse_param_line(&body_str) {
                            result.assertions.push((name, parse_type_string(&ty_str)));
                        }
                    }
                }
                "suppress" | "psalm-suppress" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        for rule in body_str.split([',', ' ']) {
                            let rule = rule.trim().to_string();
                            if !rule.is_empty() {
                                result.suppressed_issues.push(rule);
                            }
                        }
                    }
                }
                "see" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        result.see.push(body_str.to_string());
                    }
                }
                "link" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        result.see.push(body_str.to_string());
                    }
                }
                "mixin" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        let base_class =
                            body_str.split('<').next().unwrap_or(&body_str).to_string();
                        result.mixins.push(base_class);
                    }
                }
                "property" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        if let Some((ty_str, name)) = parse_param_line(&body_str) {
                            result.properties.push(DocProperty {
                                type_hint: ty_str,
                                name: name.trim_start_matches('$').to_string(),
                                read_only: false,
                                write_only: false,
                            });
                        }
                    }
                }
                "property-read" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        if let Some((ty_str, name)) = parse_param_line(&body_str) {
                            result.properties.push(DocProperty {
                                type_hint: ty_str,
                                name: name.trim_start_matches('$').to_string(),
                                read_only: true,
                                write_only: false,
                            });
                        }
                    }
                }
                "property-write" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        if let Some((ty_str, name)) = parse_param_line(&body_str) {
                            result.properties.push(DocProperty {
                                type_hint: ty_str,
                                name: name.trim_start_matches('$').to_string(),
                                read_only: false,
                                write_only: true,
                            });
                        }
                    }
                }
                "method" | "psalm-method" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        if let Some(m) = parse_method_line(&body_str) {
                            result.methods.push(m);
                        }
                    }
                }
                "psalm-type" | "phpstan-type" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        if let Some((name, type_expr)) = body_str.split_once('=') {
                            result.type_aliases.push(DocTypeAlias {
                                name: name.trim().to_string(),
                                type_expr: type_expr.trim().to_string(),
                            });
                        }
                    }
                }
                "psalm-import-type" | "phpstan-import-type" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        if let Some(import) = parse_import_type(&body_str) {
                            result.import_types.push(import);
                        }
                    }
                }
                "since" if result.since.is_none() => {
                    if let Some(body_str) = body_text(&tag.body) {
                        let v = body_str.split_whitespace().next().unwrap_or("");
                        if !v.is_empty() {
                            result.since = Some(v.to_string());
                        }
                    }
                }
                "removed" if result.removed.is_none() => {
                    if let Some(body_str) = body_text(&tag.body) {
                        let v = body_str.split_whitespace().next().unwrap_or("");
                        if !v.is_empty() {
                            result.removed = Some(v.to_string());
                        }
                    }
                }
                "internal" => result.is_internal = true,
                "pure" => result.is_pure = true,
                "immutable" => result.is_immutable = true,
                "readonly" => result.is_readonly = true,
                "inheritDoc" | "inheritdoc" => result.is_inherit_doc = true,
                "api" | "psalm-api" => result.is_api = true,
                "psalm-assert-if-true" | "phpstan-assert-if-true" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        if let Some((ty_str, name)) = parse_param_line(&body_str) {
                            result
                                .assertions_if_true
                                .push((name, parse_type_string(&ty_str)));
                        }
                    }
                }
                "psalm-assert-if-false" | "phpstan-assert-if-false" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        if let Some((ty_str, name)) = parse_param_line(&body_str) {
                            result
                                .assertions_if_false
                                .push((name, parse_type_string(&ty_str)));
                        }
                    }
                }
                "psalm-property" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        if let Some((ty_str, name)) = parse_param_line(&body_str) {
                            result.properties.push(DocProperty {
                                type_hint: ty_str,
                                name,
                                read_only: false,
                                write_only: false,
                            });
                        }
                    }
                }
                "psalm-property-read" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        if let Some((ty_str, name)) = parse_param_line(&body_str) {
                            result.properties.push(DocProperty {
                                type_hint: ty_str,
                                name,
                                read_only: true,
                                write_only: false,
                            });
                        }
                    }
                }
                "psalm-property-write" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        if let Some((ty_str, name)) = parse_param_line(&body_str) {
                            result.properties.push(DocProperty {
                                type_hint: ty_str,
                                name,
                                read_only: false,
                                write_only: true,
                            });
                        }
                    }
                }
                "psalm-require-extends" | "phpstan-require-extends" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        let cls = body_str
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        if !cls.is_empty() {
                            result.require_extends.push(cls);
                        }
                    }
                }
                "psalm-require-implements" | "phpstan-require-implements" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        let cls = body_str
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        if !cls.is_empty() {
                            result.require_implements.push(cls);
                        }
                    }
                }
                "mir-check" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        if let Some((var_part, type_part)) = body_str.split_once(" is ") {
                            let var_name = var_part.trim().trim_start_matches('$').to_string();
                            let type_string = type_part.trim().to_string();
                            if !var_name.is_empty() && !type_string.is_empty() {
                                result.mir_checks.push((var_name, type_string));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if text.to_ascii_lowercase().contains("{@inheritdoc}") {
            result.is_inherit_doc = true;
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

#[derive(Debug, Default, Clone)]
pub struct DocImportType {
    /// The name exported by the source class (the original alias name).
    pub original: String,
    /// The local name to use in this class (`as LocalAlias`); defaults to `original`.
    pub local: String,
    /// The FQCN of the class to import the type from.
    pub from_class: String,
}

// ---------------------------------------------------------------------------
// ParsedDocblock
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct ParsedDocblock {
    /// `@param Type $name`
    pub params: Vec<(String, Type)>,
    /// `@return Type`
    pub return_type: Option<Type>,
    /// `@var Type` or `@var Type $name` — type and optional variable name
    pub var_type: Option<Type>,
    /// Optional variable name from `@var Type $name`
    pub var_name: Option<String>,
    /// `@template T` / `@template T of Bound` / `@template-covariant T` / `@template-contravariant T`
    pub templates: Vec<(String, Option<Type>, Variance)>,
    /// `@extends ClassName<T>`
    pub extends: Option<Type>,
    /// `@implements InterfaceName<T>`
    pub implements: Vec<Type>,
    /// `@throws ClassName`
    pub throws: Vec<String>,
    /// `@psalm-assert Type $var`
    pub assertions: Vec<(String, Type)>,
    /// `@psalm-assert-if-true Type $var`
    pub assertions_if_true: Vec<(String, Type)>,
    /// `@psalm-assert-if-false Type $var`
    pub assertions_if_false: Vec<(String, Type)>,
    /// `@psalm-suppress IssueName`
    pub suppressed_issues: Vec<String>,
    pub is_deprecated: bool,
    pub is_internal: bool,
    pub is_pure: bool,
    pub is_immutable: bool,
    pub is_readonly: bool,
    pub is_api: bool,
    /// `@inheritDoc` or `{@inheritDoc}` was present — documentation should be
    /// inherited from the nearest ancestor that has a real docblock.
    pub is_inherit_doc: bool,
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
    /// `@psalm-import-type Alias from SourceClass` / `@phpstan-import-type ...`
    pub import_types: Vec<DocImportType>,
    /// `@psalm-require-extends ClassName` / `@phpstan-require-extends ClassName`
    pub require_extends: Vec<String>,
    /// `@psalm-require-implements InterfaceName` / `@phpstan-require-implements InterfaceName`
    pub require_implements: Vec<String>,
    /// `@since X.Y` — first PHP version this symbol exists in.
    pub since: Option<String>,
    /// `@removed X.Y` — first PHP version this symbol no longer exists in.
    pub removed: Option<String>,
    /// Malformed type annotations detected during parsing.
    pub invalid_annotations: Vec<String>,
    /// `@mir-check $var is TYPE` — (var_name_without_dollar, type_string)
    pub mir_checks: Vec<(String, String)>,
}

impl ParsedDocblock {
    /// Returns the type for a given parameter name (strips leading `$`).
    ///
    /// Uses the **last** match so that `@psalm-param` / `@phpstan-param` (which
    /// php-rs-parser maps to the same `Param` variant as `@param`) overrides a
    /// preceding plain `@param` annotation.
    pub fn get_param_type(&self, name: &str) -> Option<&Type> {
        let name = name.trim_start_matches('$');
        self.params
            .iter()
            .rfind(|(n, _)| n.trim_start_matches('$') == name)
            .map(|(_, ty)| ty)
    }
}

// ---------------------------------------------------------------------------
// Type string parser
// ---------------------------------------------------------------------------

/// Parse a PHPDoc type expression string into a `Type`.
/// Handles: `string`, `int|null`, `array<string>`, `list<int>`,
/// `ClassName`, `?string` (nullable), `string[]` (array shorthand).
pub fn parse_type_string(s: &str) -> Type {
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
        "iterable" => Type::single(Atomic::TArray {
            key: Box::new(Type::single(Atomic::TMixed)),
            value: Box::new(Type::mixed()),
        }),
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

fn parse_generic(name: &str, inner: &str) -> Type {
    match name.to_lowercase().as_str() {
        "array" => {
            let params = split_generics(inner);
            let (key, value) = match params.len() {
                n if n >= 2 => (
                    parse_type_string(params[0].trim()),
                    parse_type_string(params[1].trim()),
                ),
                1 => (
                    Type::single(Atomic::TInt),
                    parse_type_string(params[0].trim()),
                ),
                _ => (Type::single(Atomic::TInt), Type::mixed()),
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
            let (key, value) = match params.len() {
                n if n >= 2 => (
                    parse_type_string(params[0].trim()),
                    parse_type_string(params[1].trim()),
                ),
                1 => (
                    Type::single(Atomic::TInt),
                    parse_type_string(params[0].trim()),
                ),
                _ => (Type::single(Atomic::TInt), Type::mixed()),
            };
            Type::single(Atomic::TNonEmptyArray {
                key: Box::new(key),
                value: Box::new(value),
            })
        }
        "iterable" => {
            let params = split_generics(inner);
            let value = match params.len() {
                n if n >= 2 => parse_type_string(params[1].trim()),
                1 => parse_type_string(params[0].trim()),
                _ => Type::mixed(),
            };
            Type::single(Atomic::TArray {
                key: Box::new(Type::single(Atomic::TMixed)),
                value: Box::new(value),
            })
        }
        "class-string" => Type::single(Atomic::TClassString(Some(
            normalize_fqcn(inner.trim()).into(),
        ))),
        "int" => {
            // int<min, max>
            Type::single(Atomic::TIntRange {
                min: None,
                max: None,
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

fn parse_keyed_array(inner: &str, is_list: bool) -> Type {
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

fn parse_callable_syntax(s: &str) -> Option<Type> {
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

fn find_matching_paren(s: &str) -> Option<usize> {
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

/// Parse template tag format: `T`, `T of Bound`, or `T as Bound`
fn parse_template_line(_tag_name: &str, body: Option<String>) -> Option<(String, Option<String>)> {
    let body = body?;
    if let Some((name, bound)) = body.split_once(" of ").or_else(|| body.split_once(" as ")) {
        Some((name.trim().to_string(), Some(bound.trim().to_string())))
    } else {
        Some((body.trim().to_string(), None))
    }
}

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

/// Parse `@psalm-import-type` body.
///
/// Formats:
/// - `AliasName from SourceClass`
/// - `AliasName as LocalAlias from SourceClass`
fn parse_import_type(body: &str) -> Option<DocImportType> {
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

fn parse_param_line(s: &str) -> Option<(String, String)> {
    // Formats: `Type $name`, `Type $name description`
    // Types can contain spaces (e.g., `array<string, int>`), so we need to find the variable name.
    // The variable name is the `$identifier` that comes after whitespace (not part of type syntax).

    // Strategy: find the last sequence of whitespace followed by `$identifier`
    // This handles both simple types and types with generics/spaces.
    let mut best_split: Option<(String, String)> = None;

    for (i, ch) in s.char_indices() {
        if ch.is_whitespace() {
            // Found whitespace; check what comes after it
            let after = &s[i..].trim_start();
            if after.starts_with('$') {
                // Found a `$` after whitespace
                let mut var_parts = after.split(char::is_whitespace);
                if let Some(name_with_dollar) = var_parts.next() {
                    let name = name_with_dollar.trim_start_matches('$').to_string();
                    if !name.is_empty() {
                        let type_part = s[..i].trim().to_string();
                        if !type_part.is_empty() {
                            // Keep this as a candidate; if there are more, the last one wins
                            best_split = Some((type_part, name));
                        }
                    }
                }
            }
        }
    }

    best_split
}

fn extract_return_type(s: &str) -> String {
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

/// Depth-aware split on `&` — does not break `&` inside `<>`, `()`, or `{}`.
fn split_intersection(s: &str) -> Vec<String> {
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
fn is_balanced_parens(s: &str) -> bool {
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

/// Return the leading type expression from `s`, stopping at top-level whitespace.
/// Spaces inside `<…>` brackets are kept so that `array<string, int>` is returned whole.
fn extract_type_prefix(s: &str) -> &str {
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

/// Parses `$param is TypeName ? TrueType : FalseType` into a `TConditional`.
fn parse_conditional_type(s: &str) -> Option<Type> {
    if !s.starts_with('$') {
        return None;
    }
    let is_pos = s.find(" is ")?;
    let param_raw = s[..is_pos].trim();
    let param_name = param_raw
        .strip_prefix('$')
        .filter(|n| !n.is_empty())
        .map(mir_types::Name::new);
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
fn find_char_at_depth(s: &str, target: char) -> Option<usize> {
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

fn normalize_fqcn(s: &str) -> String {
    // Strip leading backslash if present — we normalize all FQCNs without leading `\`
    s.trim_start_matches('\\').to_string()
}

/// Returns an error message if `s` is a malformed PHPDoc type expression, otherwise `None`.
///
/// Detects:
/// - unclosed generics (`array<`, `Foo<Bar`)
/// - `$variable` in type position (only `$this` is valid)
fn validate_type_str(s: &str, tag: &str) -> Option<String> {
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
    }
    None
}

fn has_empty_generics(s: &str) -> bool {
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
    fn parse_since_plain() {
        let parsed = DocblockParser::parse("/** @since 8.0 */");
        assert_eq!(parsed.since.as_deref(), Some("8.0"));
        assert_eq!(parsed.removed, None);
    }

    #[test]
    fn parse_since_strips_trailing_description() {
        // phpstorm-stubs commonly writes `@since X.Y description text`.
        // Only the leading version token must reach the version parser.
        let parsed = DocblockParser::parse("/** @since 1.4.0 added \\$options argument */");
        assert_eq!(parsed.since.as_deref(), Some("1.4.0"));
    }

    #[test]
    fn parse_removed_tag() {
        let parsed = DocblockParser::parse("/** @removed 8.0 use mb_convert_encoding */");
        assert_eq!(parsed.removed.as_deref(), Some("8.0"));
    }

    #[test]
    fn parse_since_empty_body_is_none() {
        let parsed = DocblockParser::parse("/** @since */");
        assert_eq!(parsed.since, None);
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
    fn parse_import_type_no_as() {
        let doc = "/** @psalm-import-type UserId from UserRepository */";
        let parsed = DocblockParser::parse(doc);
        assert_eq!(parsed.import_types.len(), 1);
        assert_eq!(parsed.import_types[0].original, "UserId");
        assert_eq!(parsed.import_types[0].local, "UserId");
        assert_eq!(parsed.import_types[0].from_class, "UserRepository");
    }

    #[test]
    fn parse_import_type_with_as() {
        let doc = "/** @psalm-import-type UserId as LocalId from UserRepository */";
        let parsed = DocblockParser::parse(doc);
        assert_eq!(parsed.import_types.len(), 1);
        assert_eq!(parsed.import_types[0].original, "UserId");
        assert_eq!(parsed.import_types[0].local, "LocalId");
        assert_eq!(parsed.import_types[0].from_class, "UserRepository");
    }

    #[test]
    fn parse_require_extends() {
        let doc = "/** @psalm-require-extends Model */";
        let parsed = DocblockParser::parse(doc);
        assert_eq!(parsed.require_extends, vec!["Model".to_string()]);
    }

    #[test]
    fn parse_require_implements() {
        let doc = "/** @psalm-require-implements Countable */";
        let parsed = DocblockParser::parse(doc);
        assert_eq!(parsed.require_implements, vec!["Countable".to_string()]);
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

    #[test]
    fn validate_unclosed_generic_return() {
        let parsed = DocblockParser::parse("/** @return array< */");
        assert_eq!(parsed.invalid_annotations.len(), 1);
        assert!(
            parsed.invalid_annotations[0].contains("unclosed generic"),
            "got: {}",
            parsed.invalid_annotations[0]
        );
    }

    #[test]
    fn parse_empty_generic_array_graceful() {
        let u = parse_type_string("array<>");
        assert!(u.contains(|t| matches!(t, Atomic::TArray { .. })));
    }

    #[test]
    fn parse_empty_generic_iterable_graceful() {
        let u = parse_type_string("iterable<>");
        assert!(u.contains(|t| matches!(t, Atomic::TArray { .. })));
    }

    #[test]
    fn parse_empty_generic_non_empty_array_graceful() {
        let u = parse_type_string("non-empty-array<>");
        assert!(u.contains(|t| matches!(t, Atomic::TNonEmptyArray { .. })));
    }

    #[test]
    fn validate_variable_in_type_position_param() {
        let parsed = DocblockParser::parse("/** @param Foo|$invalid $x */");
        assert_eq!(parsed.invalid_annotations.len(), 1);
        assert!(
            parsed.invalid_annotations[0].contains("$invalid"),
            "got: {}",
            parsed.invalid_annotations[0]
        );
    }

    #[test]
    fn validate_this_is_valid_in_type_position() {
        let parsed = DocblockParser::parse("/** @return $this */");
        assert!(
            parsed.invalid_annotations.is_empty(),
            "unexpected error: {:?}",
            parsed.invalid_annotations
        );
    }

    #[test]
    fn validate_unclosed_generic_var() {
        let parsed = DocblockParser::parse("/** @var array<string */");
        assert_eq!(parsed.invalid_annotations.len(), 1);
        assert!(parsed.invalid_annotations[0].contains("@var"));
    }

    #[test]
    fn validate_variable_in_template_bound() {
        let parsed = DocblockParser::parse("/** @template T of $invalid */");
        assert_eq!(parsed.invalid_annotations.len(), 1);
        assert!(parsed.invalid_annotations[0].contains("$invalid"));
    }
}
