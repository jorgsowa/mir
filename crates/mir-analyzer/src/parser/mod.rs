pub mod docblock;
pub(crate) mod type_from_hint;

use std::sync::Arc;

use php_ast::Span;

pub use docblock::{DocblockParser, ParsedDocblock};
pub use type_from_hint::{type_from_hint, type_from_hint_owned};

// ---------------------------------------------------------------------------
// Parse-error → Issue conversion
// ---------------------------------------------------------------------------

/// Convert a parser diagnostic to a [`mir_issues::Issue`], using the source
/// and source map to derive a precise location.
pub(crate) fn parse_error_to_issue(
    err: &php_rs_parser::diagnostics::ParseError,
    file: &Arc<str>,
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
) -> mir_issues::Issue {
    let span = err.span();
    let (line, col_start) = crate::diagnostics::offset_to_line_col(source, span.start, source_map);
    let (line_end, col_end) = crate::diagnostics::offset_to_line_col(source, span.end, source_map);

    mir_issues::Issue::new(
        mir_issues::IssueKind::ParseError {
            message: err.to_string(),
        },
        mir_issues::Location {
            file: file.clone(),
            line,
            line_end,
            col_start,
            col_end,
        },
    )
}

/// Compile-time rules the parser checks that the analyzer reports itself with a
/// dedicated issue kind; the parser's diagnostic is dropped as a duplicate.
const ANALYZER_COVERED_MESSAGES: &[&str] = &[
    "when current class scope has no parent",
    "when no class scope is active",
    "(): Return type must be string when declared",
    "Cannot apply #[\\Attribute] to",
    "Cannot apply #[\\Deprecated] to",
    "cannot target",
    "must return a value",
    "Generator return type must be a supertype of Generator",
    "Readonly property cannot have default value",
    "Redefinition of parameter",
    "Cannot re-assign $this",
];

fn is_analyzer_covered_error(err: &php_rs_parser::diagnostics::ParseError) -> bool {
    matches!(
        err,
        php_rs_parser::diagnostics::ParseError::Forbidden { .. }
    ) && {
        let msg = err.to_string();
        ANALYZER_COVERED_MESSAGES.iter().any(|m| msg.contains(m))
    }
}

/// Parser diagnostics that are dropped from the issue stream and never block
/// analysis: the ones the analyzer reports itself.
pub(crate) fn is_redundant_parse_error(err: &php_rs_parser::diagnostics::ParseError) -> bool {
    is_analyzer_covered_error(err)
}

/// Returns `true` for parser diagnostics that should block semantic analysis.
pub(crate) fn is_hard_parse_error(err: &php_rs_parser::diagnostics::ParseError) -> bool {
    !is_redundant_parse_error(err)
}

/// Returns `true` for parser diagnostics that leave the AST incomplete.
/// Narrower than [`is_hard_parse_error`]: `Forbidden` diagnostics (redeclared
/// method, reserved function name, valued case on a pure enum, ...) are
/// emitted after the node is fully built.
fn leaves_ast_incomplete(err: &php_rs_parser::diagnostics::ParseError) -> bool {
    match err {
        // Past the nesting limit the subtree becomes `ExprKind::Error`.
        php_rs_parser::diagnostics::ParseError::NestingTooDeep { .. } => true,
        php_rs_parser::diagnostics::ParseError::Forbidden { .. } => false,
        _ => is_hard_parse_error(err),
    }
}

/// Whether `errors` keep the definition slice collected from `path` out of
/// the parse and stub caches. Bundled stubs only need a complete AST: several
/// (`Core.php`, `intl.php`, ...) trip `Forbidden` checks because they declare
/// per-version variants side by side, and would otherwise be re-parsed every
/// session. Every other file keeps the stricter [`is_hard_parse_error`] rule.
pub(crate) fn parse_errors_block_cache(
    path: &str,
    errors: &[php_rs_parser::diagnostics::ParseError],
) -> bool {
    if crate::stubs::is_stub_path(path) {
        errors.iter().any(leaves_ast_incomplete)
    } else {
        errors.iter().any(is_hard_parse_error)
    }
}

// ---------------------------------------------------------------------------
// Source location helpers
// ---------------------------------------------------------------------------

/// Extract the exact source text covered by a span.
pub(crate) fn span_text(src: &str, span: Span) -> Option<String> {
    if span.start >= span.end {
        return None;
    }
    let s = span.start as usize;
    let e = (span.end as usize).min(src.len());
    src.get(s..e)
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
}

// ---------------------------------------------------------------------------
// Docblock extraction from source text
// ---------------------------------------------------------------------------

/// Scan backwards from `offset` and return the `/** ... */` docblock comment
/// that immediately precedes the token at that position, if any.
///
/// Whitespace and class-level modifier keywords (`final`, `abstract`,
/// `readonly`) between the docblock and the declaration are skipped — the
/// php-rs-parser places `span.start` at the `class`/`interface`/`trait`
/// keyword, after any modifiers.
///
/// A `?>`/inline-HTML/`<?php` block boundary is also skipped: a Yii2-style
/// view template routinely writes `/** @var Post $model */` immediately
/// before closing the PHP block, then reopens it later to actually use the
/// variable (php-lsp#235). The docblock is textually adjacent to the
/// declaration in every sense that matters here — nothing but tag
/// punctuation and inert markup sits between them.
pub(crate) fn find_preceding_docblock(source: &str, offset: u32) -> Option<String> {
    let offset = (offset as usize).min(source.len());
    if offset == 0 {
        return None;
    }
    let mut trimmed = source[..offset].trim_end();
    loop {
        let after_ws = trimmed.trim_end();

        if let Some(before_tag) = strip_trailing_reopen_tag(after_ws) {
            let before_html = before_tag.trim_end();
            trimmed = match before_html.rfind("?>") {
                Some(close_idx) => &before_html[..close_idx],
                // A re-opening tag with nothing before it to close — this is
                // the file's very first `<?php`, not a split block boundary.
                None => before_html,
            };
            continue;
        }

        // Strip trailing modifier keywords like `final` or `abstract readonly`.
        let last_word_start = after_ws
            .char_indices()
            .rfind(|(_, c)| !c.is_ascii_alphabetic())
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        let word = &after_ws[last_word_start..];
        if matches!(word, "final" | "abstract" | "readonly") {
            trimmed = &after_ws[..last_word_start];
        } else {
            trimmed = after_ws;
            break;
        }
    }
    if !trimmed.ends_with("*/") {
        return None;
    }
    let end = trimmed.rfind("*/")?;
    // Prefer /** docblocks; fall back to /* for inline @var annotations (e.g. Yii2 view files).
    let start = trimmed[..end]
        .rfind("/**")
        .or_else(|| trimmed[..end].rfind("/*"))?;
    Some(trimmed[start..end + 2].to_string())
}

/// Strip a trailing PHP re-opening tag (`<?php`, `<?=`, or short-open `<?`),
/// returning the text before it. Used by [`find_preceding_docblock`] to look
/// past a `?> ... <?php` gap for the docblock that precedes it.
fn strip_trailing_reopen_tag(s: &str) -> Option<&str> {
    for tag in ["<?php", "<?=", "<?"] {
        if let Some(stripped) = s.strip_suffix(tag) {
            return Some(stripped);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Name resolution helper — join Name parts to a string
// ---------------------------------------------------------------------------

pub(crate) fn name_to_string(name: &php_ast::ast::Name<'_, '_>) -> String {
    name.to_string_repr().into_owned()
}

/// Same as [`name_to_string`] but for the owned (lifetime-free) AST.
pub(crate) fn name_to_string_owned(name: &php_ast::owned::Name) -> String {
    let joined = name
        .parts
        .iter()
        .map(|p| p.as_ref())
        .collect::<Vec<&str>>()
        .join("\\");
    if name.kind == php_ast::ast::NameKind::FullyQualified {
        format!("\\{}", joined)
    } else {
        joined
    }
}

#[cfg(test)]
mod ast_incomplete_tests {
    use super::{is_hard_parse_error, leaves_ast_incomplete, parse_errors_block_cache};

    fn classify(src: &str) -> (bool, bool) {
        let parsed = php_rs_parser::parse(src);
        (
            parsed.errors.iter().any(is_hard_parse_error),
            parsed.errors.iter().any(leaves_ast_incomplete),
        )
    }

    #[test]
    fn redeclared_method_is_hard_but_ast_complete() {
        let src = "<?php\nclass A { function f() {} function f() {} }\n";
        assert_eq!(classify(src), (true, false));
    }

    #[test]
    fn reserved_function_name_is_hard_but_ast_complete() {
        assert_eq!(classify("<?php\nfunction exit() {}\n"), (true, false));
    }

    #[test]
    fn syntax_error_leaves_ast_incomplete() {
        assert_eq!(classify("<?php\nfunction f( {}\n"), (true, true));
    }

    #[test]
    fn forbidden_errors_block_caching_only_outside_bundled_stubs() {
        let parsed = php_rs_parser::parse("<?php\nclass A { function f() {} function f() {} }\n");
        assert!(!parse_errors_block_cache(
            "stubs/Core/Core.php",
            &parsed.errors
        ));
        assert!(parse_errors_block_cache("src/A.php", &parsed.errors));
        // A user file that merely mirrors a stub's virtual path is still user code.
        assert!(parse_errors_block_cache(
            "/app/stubs/Core/Core.php",
            &parsed.errors
        ));
    }

    #[test]
    fn syntax_errors_block_caching_for_bundled_stubs() {
        let parsed = php_rs_parser::parse("<?php\nfunction f( {}\n");
        assert!(parse_errors_block_cache(
            "stubs/Core/Core.php",
            &parsed.errors
        ));
    }

    /// The depth diagnostic must block caching on its own, so a slice built
    /// from a truncated AST is never cached.
    #[test]
    fn expression_depth_limit_leaves_ast_incomplete() {
        let src = format!("<?php\n$x = {}1{};\n", "(".repeat(1000), ")".repeat(1000));
        let parsed = php_rs_parser::parse(&src);
        let depth_err = parsed
            .errors
            .iter()
            .find(|e| {
                matches!(
                    e,
                    php_rs_parser::diagnostics::ParseError::NestingTooDeep { .. }
                )
            })
            .expect("parser no longer emits the depth-limit diagnostic");
        assert!(leaves_ast_incomplete(depth_err));
    }
}

#[cfg(test)]
mod docblock_lookup_tests {
    use super::find_preceding_docblock;

    #[test]
    fn plain_docblock_immediately_before_offset() {
        let src = "<?php\n/** @var Post $model */\n$x = 1;\n";
        let offset = src.find("$x").unwrap() as u32;
        assert_eq!(
            find_preceding_docblock(src, offset),
            Some("/** @var Post $model */".to_string())
        );
    }

    #[test]
    fn docblock_survives_a_split_php_html_block() {
        let src = "<?php\nclass Post { public string $title = ''; }\n\
                   /** @var Post $model */\n?>\n<div>\n<?php if ($model->title !== ''): ?>\n";
        let offset = src.rfind("if (").unwrap() as u32;
        assert_eq!(
            find_preceding_docblock(src, offset),
            Some("/** @var Post $model */".to_string())
        );
    }

    #[test]
    fn docblock_survives_a_split_block_via_short_echo_reopen() {
        let src = "<?php\n/** @var Post $model */\n?>\n<div>\n<?= $model->title ?>\n";
        let offset = src.rfind("$model->title").unwrap() as u32;
        assert_eq!(
            find_preceding_docblock(src, offset),
            Some("/** @var Post $model */".to_string())
        );
    }

    /// A re-opening `<?php` with no preceding `?>` is the file's very first
    /// tag, not a split block — there's nothing to skip past, and (since
    /// nothing precedes it) no docblock to find either.
    #[test]
    fn leading_reopen_tag_with_no_prior_close_finds_nothing() {
        let src = "<?php\n$x = 1;\n";
        let offset = src.find("$x").unwrap() as u32;
        assert_eq!(find_preceding_docblock(src, offset), None);
    }

    /// Guards against treating unrelated HTML/PHP alternation as a docblock
    /// boundary when there genuinely is no docblock to find.
    #[test]
    fn split_block_with_no_docblock_finds_nothing() {
        let src = "<?php\n$unrelated = 1;\n?>\n<div>\n<?php $x = 1;\n";
        let offset = src.rfind("$x").unwrap() as u32;
        assert_eq!(find_preceding_docblock(src, offset), None);
    }

    #[test]
    fn modifier_keywords_still_skip_after_a_split_block() {
        // Not a realistic PHP construct (a docblock + modifier can't
        // actually precede a re-opening tag), but exercises that the two
        // skip loops compose without infinite-looping or misordering.
        let src = "<?php\n/** doc */\nfinal\n?>\n<?php\n";
        let offset = src.len() as u32;
        assert_eq!(
            find_preceding_docblock(src, offset),
            Some("/** doc */".to_string())
        );
    }
}
