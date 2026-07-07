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
                "param-out" | "psalm-param-out" | "phpstan-param-out" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        if let Some((ty_s, name)) = parse_param_line(&body_str) {
                            if let Some(msg) = validate_type_str(&ty_s, "param-out") {
                                result.invalid_annotations.push(msg);
                            } else {
                                result.out_params.push((
                                    name.trim_start_matches('$').to_string(),
                                    parse_type_string(&ty_s),
                                ));
                            }
                        }
                    }
                }
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
                        if let Some(msg) = validate_type_str(&name, "template") {
                            result.invalid_annotations.push(msg);
                        }
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
                        if let Some(msg) = validate_type_str(&name, "template-covariant") {
                            result.invalid_annotations.push(msg);
                        }
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
                        if let Some(msg) = validate_type_str(&name, "template-contravariant") {
                            result.invalid_annotations.push(msg);
                        }
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
                "extends" | "template-extends" | "phpstan-extends" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        let trimmed = body_str.trim();
                        if let Some(msg) = validate_type_str(trimmed, "extends") {
                            result.invalid_annotations.push(msg);
                        }
                        result.extends.push(parse_type_string(trimmed));
                    }
                }
                "implements" | "template-implements" | "phpstan-implements" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        let trimmed = body_str.trim();
                        if let Some(msg) = validate_type_str(trimmed, "implements") {
                            result.invalid_annotations.push(msg);
                        }
                        result.implements.push(parse_type_string(trimmed));
                    }
                }
                "assert" | "psalm-assert" | "phpstan-assert" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        if let Some((ty_str, name)) = parse_param_line(&body_str) {
                            result.assertions.push((name, parse_type_string(&ty_str)));
                        }
                    }
                }
                "if-this-is" | "psalm-if-this-is" | "phpstan-if-this-is" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        let trimmed = body_str.trim();
                        if !trimmed.is_empty() {
                            result.if_this_is = Some(parse_type_string(trimmed));
                        }
                    }
                }
                "self-out" | "psalm-self-out" | "phpstan-self-out" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        let trimmed = body_str.trim();
                        if !trimmed.is_empty() {
                            result.self_out = Some(parse_type_string(trimmed));
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
                    let body_str = body_text(&tag.body).unwrap_or_default().trim().to_string();
                    if let Some(err) = validate_method_body(&body_str) {
                        result.invalid_annotations.push(err);
                    } else if let Some(m) = parse_method_line(&body_str) {
                        result.methods.push(m);
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
                "seal-properties" | "psalm-seal-properties" => result.seal_properties = true,
                "no-named-arguments" => result.no_named_arguments = true,
                "mutation-free" | "psalm-mutation-free" | "phpstan-mutation-free" => {
                    result.is_mutation_free = true
                }
                "psalm-external-mutation-free" => result.is_external_mutation_free = true,
                "immutable" | "psalm-immutable" => result.is_immutable = true,
                "readonly" => result.is_readonly = true,
                "final" => result.is_final = true,
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
                "trace" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        // Support both comma-separated and space-separated variable names
                        for part in body_str.split([',', ' ']) {
                            let var_name = part.trim().trim_start_matches('$').to_string();
                            if !var_name.is_empty() {
                                result.trace_vars.push(var_name);
                            }
                        }
                    }
                }
                "taint-sink" => {
                    if let Some(body_str) = body_text(&tag.body) {
                        // Format: `kind $param` or `kind $param1 $param2`
                        let mut tokens = body_str.split_whitespace();
                        if let Some(kind) = tokens.next() {
                            let kind = kind.to_string();
                            for param_token in tokens {
                                let param = param_token.trim_start_matches('$').to_string();
                                if !param.is_empty() {
                                    result.taint_sinks.push((param, kind.clone()));
                                }
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
    /// `@param-out Type $name` / `@psalm-param-out Type $name` — the type written
    /// back to the caller's by-ref argument after the call.
    pub out_params: Vec<(String, Type)>,
    /// `@return Type`
    pub return_type: Option<Type>,
    /// `@var Type` or `@var Type $name` — type and optional variable name
    pub var_type: Option<Type>,
    /// Optional variable name from `@var Type $name`
    pub var_name: Option<String>,
    /// `@template T` / `@template T of Bound` / `@template-covariant T` / `@template-contravariant T`
    pub templates: Vec<(String, Option<Type>, Variance)>,
    /// `@extends ClassName<T>` — a class has at most one entry (its single
    /// parent); an interface may have several, one per base interface named
    /// in its native `extends A, B` clause.
    pub extends: Vec<Type>,
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
    pub is_mutation_free: bool,
    pub is_external_mutation_free: bool,
    pub no_named_arguments: bool,
    pub is_immutable: bool,
    pub is_readonly: bool,
    pub is_api: bool,
    /// `@final` — class should be treated as final even without the PHP `final` keyword.
    pub is_final: bool,
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
    /// `@trace $var1, $var2` or `@trace $var1 $var2` — variable names to trace
    pub trace_vars: Vec<String>,
    /// `@taint-sink <kind> $param` — (param_name_without_dollar, sink_kind_string)
    pub taint_sinks: Vec<(String, String)>,
    /// `@seal-properties` / `@psalm-seal-properties` — disallows undeclared property access.
    pub seal_properties: bool,
    /// `@if-this-is Type` / `@psalm-if-this-is Type` — the method may only be
    /// called when `$this` satisfies this type. Stored as the raw parsed type;
    /// class names are resolved later by the collector.
    pub if_this_is: Option<Type>,
    /// `@psalm-self-out Type` / `@phpstan-self-out Type` — the receiver's type
    /// after this call returns. Stored as the raw parsed type; class names
    /// (and `self`/`static`) are resolved later by the collector.
    pub self_out: Option<Type>,
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

    /// Returns the `@param-out` / `@psalm-param-out` type for a given parameter
    /// name, if declared. Uses the **last** match.
    pub fn get_out_param_type(&self, name: &str) -> Option<&Type> {
        let name = name.trim_start_matches('$');
        self.out_params
            .iter()
            .rfind(|(n, _)| n.trim_start_matches('$') == name)
            .map(|(_, ty)| ty)
    }
}

// ---------------------------------------------------------------------------
// Type string parser
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests;
/// Parse a PHPDoc type expression string into a `Type`.
/// Handles: `string`, `int|null`, `array<string>`, `list<int>`,
/// `ClassName`, `?string` (nullable), `string[]` (array shorthand).
mod types;
mod validate;

pub(crate) use types::SelfIntConstantsGuard;
use types::*;
use validate::*;

pub(crate) use types::parse_type_string;
