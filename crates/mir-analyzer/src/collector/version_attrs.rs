//! Decoder for phpstorm-stubs version attributes.
//!
//! phpstorm-stubs annotates ~2,400 stub sites with two JetBrains attributes that
//! make a single verbatim stub serve every PHP version:
//!
//! - `#[PhpStormStubsElementAvailable($from, $to = null)]` — the
//!   function/method/parameter exists only within the **inclusive** version
//!   range `[from, to]`.
//! - `#[LanguageLevelTypeAware(array $map, string $default)]` — the type of the
//!   param/return is `map[highest key ≤ target]`, or `default` below the lowest
//!   key. An empty `default` (`default: ''`) means "no type override".
//!
//! These functions are pure over `&[Attribute]` + `use_aliases` + a target
//! [`PhpVersion`], so they are unit-testable in isolation with no collector
//! state. They are gated by the caller on `php_version == Some(_)`; user code
//! never reaches them.
//!
//! See `ROADMAP.md` §5 for the confirmed upstream semantics.

use rustc_hash::FxHashMap;

use php_ast::owned::{Arg, Attribute, Expr, ExprKind};

use super::resolution::resolve_alias_only;
use crate::parser::name_to_string_owned;
use crate::php_version::PhpVersion;

/// Canonical FQN of `#[LanguageLevelTypeAware]`.
const LLTA_FQN: &str = "JetBrains\\PhpStorm\\Internal\\LanguageLevelTypeAware";
/// Canonical FQN of `#[PhpStormStubsElementAvailable]`.
const PSEA_FQN: &str = "JetBrains\\PhpStorm\\Internal\\PhpStormStubsElementAvailable";

/// Resolve an attribute's name to a canonical FQN (leading `\` stripped, `use`
/// aliases applied). Never matches on the bare short name, so user attributes
/// that merely share the suffix do not collide.
fn canonical_attr_name(attr: &Attribute, use_aliases: &FxHashMap<String, String>) -> String {
    resolve_alias_only(&name_to_string_owned(&attr.name), use_aliases)
}

/// Find the attribute whose canonical FQN equals `fqn`.
fn find_attr<'a>(
    attrs: &'a [Attribute],
    use_aliases: &FxHashMap<String, String>,
    fqn: &str,
) -> Option<&'a Attribute> {
    attrs
        .iter()
        .find(|a| canonical_attr_name(a, use_aliases) == fqn)
}

/// Whether an attribute argument's name (e.g. `from:`) matches `target`.
fn arg_name_is(arg: &Arg, target: &str) -> bool {
    arg.name
        .as_ref()
        .and_then(|n| n.parts.last())
        .is_some_and(|p| p.as_ref() == target)
}

/// Locate the value for an argument that is either the `pos`-th positional
/// (unnamed) argument or a named argument `name:`. Named takes precedence.
fn arg_value<'a>(args: &'a [Arg], pos: usize, name: &str) -> Option<&'a Expr> {
    if let Some(a) = args.iter().find(|a| arg_name_is(a, name)) {
        return Some(&a.value);
    }
    args.iter()
        .filter(|a| a.name.is_none())
        .nth(pos)
        .map(|a| &a.value)
}

/// Read a string-literal argument (positional `pos` or named `name`).
fn string_arg(args: &[Arg], pos: usize, name: &str) -> Option<String> {
    match arg_value(args, pos, name).map(|e| &e.kind) {
        Some(ExprKind::String(s)) => Some(s.to_string()),
        _ => None,
    }
}

/// Whether the element carrying `attrs` is available at `target`.
///
/// Reads `#[PhpStormStubsElementAvailable(from, to)]`; both bounds inclusive.
/// An absent attribute means "always available".
pub(super) fn is_available(
    attrs: &[Attribute],
    use_aliases: &FxHashMap<String, String>,
    target: PhpVersion,
) -> bool {
    let Some(attr) = find_attr(attrs, use_aliases, PSEA_FQN) else {
        return true;
    };
    let from = string_arg(&attr.args, 0, "from");
    let to = string_arg(&attr.args, 1, "to");
    target.in_range(from.as_deref(), to.as_deref())
}

/// The `#[LanguageLevelTypeAware]` type override for `target`, if any.
///
/// The first positional argument (or named `type:`) is a `version => type` map;
/// the resolved type is the value at the highest key `≤ target`, else the
/// `default` (positional arg 1 or named `default:`). An empty resolved string
/// (`default: ''`) yields `None` — it must never be parsed into a type. An
/// absent attribute also yields `None`.
pub(super) fn type_aware(
    attrs: &[Attribute],
    use_aliases: &FxHashMap<String, String>,
    target: PhpVersion,
) -> Option<String> {
    let attr = find_attr(attrs, use_aliases, LLTA_FQN)?;

    // Collect (version, type) pairs from the map literal.
    let mut chosen: Option<(PhpVersion, &str)> = None;
    if let Some(Expr {
        kind: ExprKind::Array(elems),
        ..
    }) = arg_value(&attr.args, 0, "type")
    {
        for el in elems.iter() {
            let (Some(key), value) = (&el.key, &el.value) else {
                continue;
            };
            let (ExprKind::String(k), ExprKind::String(v)) = (&key.kind, &value.kind) else {
                continue;
            };
            let Ok(ver) = k.parse::<PhpVersion>() else {
                continue;
            };
            // Highest key ≤ target wins.
            if ver <= target && chosen.is_none_or(|(best, _)| ver > best) {
                chosen = Some((ver, v));
            }
        }
    }

    let resolved = match chosen {
        Some((_, ty)) => ty.to_string(),
        None => string_arg(&attr.args, 1, "default")?,
    };
    // Empty `default` means "no override".
    if resolved.is_empty() {
        None
    } else {
        Some(resolved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use php_ast::ast::NameKind;
    use php_ast::owned::{ArrayElement, ExprKind, Name};
    use php_ast::Span;

    fn span() -> Span {
        Span::default()
    }

    fn name(parts: &[&str], fq: bool) -> Name {
        Name {
            parts: parts.iter().map(|p| Box::from(*p)).collect(),
            kind: if fq {
                NameKind::FullyQualified
            } else {
                NameKind::Unqualified
            },
            span: span(),
        }
    }

    fn expr(kind: ExprKind) -> Expr {
        Expr { kind, span: span() }
    }

    fn str_expr(s: &str) -> Expr {
        expr(ExprKind::String(Box::from(s)))
    }

    fn positional(value: Expr) -> Arg {
        Arg {
            name: None,
            value,
            unpack: false,
            by_ref: false,
            span: span(),
        }
    }

    fn named(n: &str, value: Expr) -> Arg {
        Arg {
            name: Some(name(&[n], false)),
            value,
            unpack: false,
            by_ref: false,
            span: span(),
        }
    }

    fn array(pairs: &[(&str, &str)]) -> Expr {
        expr(ExprKind::Array(
            pairs
                .iter()
                .map(|(k, v)| ArrayElement {
                    key: Some(str_expr(k)),
                    value: str_expr(v),
                    unpack: false,
                    by_ref: false,
                    span: span(),
                })
                .collect(),
        ))
    }

    /// `use ... as Short` style alias map mapping the short name to the FQN.
    fn aliases(short: &str, fqn: &str) -> FxHashMap<String, String> {
        let mut m = FxHashMap::default();
        m.insert(short.to_string(), fqn.to_string());
        m
    }

    fn attr(name_parts: &[&str], fq: bool, args: Vec<Arg>) -> Attribute {
        Attribute {
            name: name(name_parts, fq),
            args: args.into_boxed_slice(),
            span: span(),
        }
    }

    fn v(major: u8, minor: u8) -> PhpVersion {
        PhpVersion::new(major, minor)
    }

    #[test]
    fn type_aware_multi_threshold_map() {
        // ['8.0' => 'int', '8.5' => 'int|null'], default: ''
        let a = aliases("LanguageLevelTypeAware", LLTA_FQN);
        let attrs = vec![attr(
            &["LanguageLevelTypeAware"],
            false,
            vec![
                positional(array(&[("8.0", "int"), ("8.5", "int|null")])),
                named("default", str_expr("")),
            ],
        )];
        assert_eq!(type_aware(&attrs, &a, v(7, 4)), None); // below lowest key, empty default
        assert_eq!(type_aware(&attrs, &a, v(8, 0)), Some("int".to_string()));
        assert_eq!(type_aware(&attrs, &a, v(8, 2)), Some("int".to_string())); // highest ≤ 8.2 is 8.0
        assert_eq!(
            type_aware(&attrs, &a, v(8, 5)),
            Some("int|null".to_string())
        );
    }

    #[test]
    fn type_aware_uses_nonempty_default_below_lowest_key() {
        let a = aliases("LanguageLevelTypeAware", LLTA_FQN);
        let attrs = vec![attr(
            &["LanguageLevelTypeAware"],
            false,
            vec![
                positional(array(&[("8.0", "int")])),
                named("default", str_expr("string")),
            ],
        )];
        assert_eq!(type_aware(&attrs, &a, v(7, 4)), Some("string".to_string()));
        assert_eq!(type_aware(&attrs, &a, v(8, 0)), Some("int".to_string()));
    }

    #[test]
    fn type_aware_empty_default_is_none() {
        let a = aliases("LanguageLevelTypeAware", LLTA_FQN);
        let attrs = vec![attr(
            &["LanguageLevelTypeAware"],
            false,
            vec![positional(array(&[])), positional(str_expr(""))],
        )];
        assert_eq!(type_aware(&attrs, &a, v(8, 2)), None);
    }

    #[test]
    fn type_aware_positional_default() {
        let a = aliases("LanguageLevelTypeAware", LLTA_FQN);
        let attrs = vec![attr(
            &["LanguageLevelTypeAware"],
            false,
            vec![
                positional(array(&[("8.0", "int")])),
                positional(str_expr("mixed")),
            ],
        )];
        assert_eq!(type_aware(&attrs, &a, v(7, 4)), Some("mixed".to_string()));
    }

    #[test]
    fn type_aware_absent_attribute_is_none() {
        let a = aliases("LanguageLevelTypeAware", LLTA_FQN);
        assert_eq!(type_aware(&[], &a, v(8, 2)), None);
    }

    #[test]
    fn type_aware_aliased_name() {
        // `use JetBrains\PhpStorm\Internal\LanguageLevelTypeAware as TA;`
        let a = aliases("TA", LLTA_FQN);
        let attrs = vec![attr(
            &["TA"],
            false,
            vec![
                positional(array(&[("8.0", "int")])),
                named("default", str_expr("")),
            ],
        )];
        assert_eq!(type_aware(&attrs, &a, v(8, 1)), Some("int".to_string()));
    }

    #[test]
    fn type_aware_fully_qualified_name_no_alias() {
        // `#[\JetBrains\PhpStorm\Internal\LanguageLevelTypeAware(...)]`
        let a = FxHashMap::default();
        let attrs = vec![attr(
            &[
                "JetBrains",
                "PhpStorm",
                "Internal",
                "LanguageLevelTypeAware",
            ],
            true,
            vec![
                positional(array(&[("8.0", "int")])),
                named("default", str_expr("")),
            ],
        )];
        assert_eq!(type_aware(&attrs, &a, v(8, 1)), Some("int".to_string()));
    }

    #[test]
    fn bare_short_name_never_matches() {
        // A user attribute named LanguageLevelTypeAware with no matching `use`
        // must not be decoded as the JetBrains one.
        let a = FxHashMap::default();
        let attrs = vec![attr(
            &["LanguageLevelTypeAware"],
            false,
            vec![positional(array(&[("8.0", "int")]))],
        )];
        assert_eq!(type_aware(&attrs, &a, v(8, 1)), None);
    }

    #[test]
    fn is_available_inclusive_to_boundary() {
        let a = aliases("PhpStormStubsElementAvailable", PSEA_FQN);
        let attrs = vec![attr(
            &["PhpStormStubsElementAvailable"],
            false,
            vec![positional(str_expr("7.0")), positional(str_expr("8.0"))],
        )];
        assert!(is_available(&attrs, &a, v(7, 0)));
        assert!(is_available(&attrs, &a, v(8, 0))); // inclusive upper bound
        assert!(!is_available(&attrs, &a, v(8, 1)));
    }

    #[test]
    fn is_available_named_from_and_to() {
        let a = aliases("PhpStormStubsElementAvailable", PSEA_FQN);
        let attrs = vec![attr(
            &["PhpStormStubsElementAvailable"],
            false,
            vec![named("from", str_expr("8.0")), named("to", str_expr("8.3"))],
        )];
        assert!(!is_available(&attrs, &a, v(7, 4)));
        assert!(is_available(&attrs, &a, v(8, 0)));
        assert!(is_available(&attrs, &a, v(8, 3)));
        assert!(!is_available(&attrs, &a, v(8, 4)));
    }

    #[test]
    fn is_available_from_only() {
        let a = aliases("PhpStormStubsElementAvailable", PSEA_FQN);
        let attrs = vec![attr(
            &["PhpStormStubsElementAvailable"],
            false,
            vec![positional(str_expr("8.0"))],
        )];
        assert!(!is_available(&attrs, &a, v(7, 4)));
        assert!(is_available(&attrs, &a, v(8, 0)));
    }

    #[test]
    fn is_available_absent_attribute() {
        let a = FxHashMap::default();
        assert!(is_available(&[], &a, v(8, 2)));
    }
}
