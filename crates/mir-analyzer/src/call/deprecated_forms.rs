//! Calls of built-in functions in a form PHP 8.6 deprecates.

use mir_issues::{IssueKind, Severity};
use mir_types::Type;
use php_ast::owned::{Arg, ArgKind, Expr, ExprKind};
use php_ast::Span;

use crate::ast_ext::ByteStringExt as _;

use crate::expr::ExpressionAnalyzer;
use crate::php_version::PhpVersion;

/// The argument bound to parameter `name` at position `pos`, with its index in `args`.
fn arg_at<'a>(args: &'a [Arg], pos: usize, name: &str) -> Option<(usize, &'a Expr)> {
    let named = args.iter().enumerate().find(|(_, a)| {
        a.name
            .as_ref()
            .is_some_and(|n| n.last_part().eq_ignore_ascii_case(name))
    });
    let (index, arg) = named.or_else(|| {
        args.iter()
            .enumerate()
            .filter(|(_, a)| a.name.is_none())
            .nth(pos)
    })?;
    Some((index, arg.value()?))
}

fn literal_string(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::String(s) => Some(s.as_str()),
        _ => None,
    }
}

fn is_string_type(ty: &Type) -> bool {
    !ty.types.is_empty() && ty.types.iter().all(|a| a.is_string())
}

/// PHP's message for a constant name it reserves from 8.6; a namespaced short name matches only in lowercase.
fn reserved_constant(name: &str) -> Option<&'static str> {
    let (namespaced, short) = match name.trim_start_matches('\\').rsplit_once('\\') {
        Some((_, short)) => (true, short),
        None => (false, name),
    };
    if short == "_" {
        return Some("_");
    }
    let matches = |word: &str| {
        if namespaced {
            short == word
        } else {
            short.eq_ignore_ascii_case(word)
        }
    };
    ["let", "is"].into_iter().find(|w| matches(w))
}

impl ExpressionAnalyzer<'_> {
    /// `define()` with a third argument or a reserved name, `class_alias()` to a reserved
    /// name, and `is_a()`/`is_subclass_of()` of a string with `$allow_string` false.
    pub(super) fn check_deprecated_call_form(
        &mut self,
        fn_name_lower: &str,
        args: &[Arg],
        arg_types: &[Type],
        span: Span,
    ) {
        if self.php_version < PhpVersion::new(8, 6)
            || args.iter().any(|a| !matches!(a.kind, ArgKind::Value(_)))
        {
            return;
        }
        match fn_name_lower {
            "define" => {
                if arg_at(args, 2, "case_insensitive").is_some() {
                    self.emit(
                        IssueKind::DeprecatedCall {
                            name: "define".to_string(),
                            message: Some(
                                "define(): Argument #3 ($case_insensitive) is ignored and treated as false since declaration of case-insensitive constants is no longer supported, passing the argument explicitly is unnecessary".into(),
                            ),
                        },
                        Severity::Info,
                        span,
                    );
                }
                let name = arg_at(args, 0, "constant_name").and_then(|(_, e)| literal_string(e));
                if let Some(word) = name.and_then(reserved_constant) {
                    self.emit(
                        IssueKind::DeprecatedSyntax {
                            message: format!(
                                "Calling a constant \"{word}\" is deprecated since 8.6"
                            ),
                        },
                        Severity::Info,
                        span,
                    );
                }
            }
            "class_alias" => {
                let alias = arg_at(args, 1, "alias").and_then(|(_, e)| literal_string(e));
                let short = alias.map(|a| a.rsplit('\\').next().unwrap_or(a));
                if let Some(word) = short.and_then(|s| {
                    ["let", "is"]
                        .into_iter()
                        .find(|w| s.eq_ignore_ascii_case(w))
                }) {
                    self.emit(
                        IssueKind::DeprecatedSyntax {
                            message: format!(
                                "Using \"{word}\" as a class alias is deprecated since 8.6"
                            ),
                        },
                        Severity::Info,
                        span,
                    );
                }
            }
            "is_a" | "is_subclass_of" => {
                let Some((object_index, _)) = arg_at(args, 0, "object_or_class") else {
                    return;
                };
                if !arg_types.get(object_index).is_some_and(is_string_type) {
                    return;
                }
                // `is_a()` defaults `$allow_string` to false, `is_subclass_of()` to true.
                let allow_string = arg_at(args, 2, "allow_string").map(|(_, e)| &e.kind);
                let disallowed = match allow_string {
                    None => fn_name_lower == "is_a",
                    Some(ExprKind::Bool(allow)) => !allow,
                    Some(_) => false,
                };
                if disallowed {
                    self.emit(
                        IssueKind::DeprecatedCall {
                            name: fn_name_lower.to_string(),
                            message: Some(
                                format!(
                                    "Calling {fn_name_lower}() with a string when $allow_string is false"
                                )
                                .into(),
                            ),
                        },
                        Severity::Info,
                        span,
                    );
                }
            }
            _ => {}
        }
    }
}
