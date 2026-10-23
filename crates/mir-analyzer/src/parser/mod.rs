pub mod docblock;
pub(crate) mod type_from_hint;

use std::borrow::Cow;
use std::sync::Arc;

use php_ast::Span;
use php_rs_parser::diagnostics::{ForbiddenRule, Phase};

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

    let message = err.to_string();
    let code = err.code().to_owned();
    let rule = err.forbidden_rule();
    let kind = if rule == Some(ForbiddenRule::ParentWithoutParentClass) {
        mir_issues::IssueKind::ParentNotFound
    } else if rule.map(ForbiddenRule::phase) == Some(Phase::Compile) {
        mir_issues::IssueKind::CompileError { message, code }
    } else {
        mir_issues::IssueKind::ParseError { message, code }
    };
    mir_issues::Issue::new(
        kind,
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
const ANALYZER_COVERED_RULES: &[ForbiddenRule] = &[
    ForbiddenRule::ToStringReturnType,
    ForbiddenRule::AttributeOnInvalidClass,
    ForbiddenRule::DeprecatedOnInvalidTarget,
    ForbiddenRule::AttributeTargetMismatch,
    ForbiddenRule::MissingReturnValue,
    ForbiddenRule::VoidReturnValue,
    ForbiddenRule::NeverReturn,
    ForbiddenRule::ReadonlyPropertyDefault,
];

fn is_analyzer_covered_error(err: &php_rs_parser::diagnostics::ParseError) -> bool {
    err.forbidden_rule()
        .is_some_and(|rule| ANALYZER_COVERED_RULES.contains(&rule))
}

/// Parser diagnostics that are dropped from the issue stream and never block
/// analysis: the ones the analyzer reports itself.
pub(crate) fn is_redundant_parse_error(err: &php_rs_parser::diagnostics::ParseError) -> bool {
    is_analyzer_covered_error(err)
}

/// Whether `errors` leave a recovery guess in the AST, which blocks body
/// analysis. `Forbidden` and `VersionTooLow` are raised on a fully built node.
pub(crate) fn blocks_body_analysis(errors: &[php_rs_parser::diagnostics::ParseError]) -> bool {
    errors.iter().any(|err| err.leaves_ast_incomplete())
}

/// Whether `errors` keep the definition slice out of the parse and stub caches.
/// The parser's diagnostics are cached with the slice, so only an incomplete
/// AST blocks: `Forbidden` and `VersionTooLow` files (bundled stubs declaring
/// per-version variants, polyfills behind `PHP_VERSION_ID` checks) are cached.
pub(crate) fn parse_errors_block_cache(errors: &[php_rs_parser::diagnostics::ParseError]) -> bool {
    blocks_body_analysis(errors)
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

pub(crate) fn name_to_string<'src>(name: &php_ast::ast::Name<'_, 'src>) -> Cow<'src, str> {
    name.to_string_repr()
}

/// Same as [`name_to_string`] but for the owned (lifetime-free) AST.
pub(crate) fn name_to_string_owned(name: &php_ast::owned::Name) -> Cow<'_, str> {
    let fully_qualified = name.kind == php_ast::ast::NameKind::FullyQualified;
    if let [single] = &*name.parts {
        if !fully_qualified {
            return Cow::Borrowed(single);
        }
    }
    let len = name.parts.iter().map(|p| p.len() + 1).sum::<usize>();
    let mut out = String::with_capacity(len);
    for (i, part) in name.parts.iter().enumerate() {
        if fully_qualified || i > 0 {
            out.push('\\');
        }
        out.push_str(part);
    }
    Cow::Owned(out)
}

#[cfg(test)]
mod ast_incomplete_tests {
    use super::{blocks_body_analysis, parse_errors_block_cache};

    fn blocks(src: &str) -> bool {
        let parsed = php_rs_parser::parse(src);
        assert!(!parsed.errors.is_empty());
        parse_errors_block_cache(&parsed.errors)
    }

    #[test]
    fn redeclared_method_leaves_ast_complete() {
        assert!(!blocks(
            "<?php\nclass A { function f() {} function f() {} }\n"
        ));
    }

    #[test]
    fn reserved_function_name_leaves_ast_complete() {
        assert!(!blocks("<?php\nfunction exit() {}\n"));
    }

    #[test]
    fn version_too_low_leaves_ast_complete() {
        let mut options = php_rs_parser::ParseOptions::default();
        options.version = php_rs_parser::PhpVersion::Php74;
        let parsed = php_rs_parser::parse_with("<?php\n$x = match (1) { 1 => 2 };\n", &options);
        assert!(!parsed.errors.is_empty());
        assert!(!blocks_body_analysis(&parsed.errors));
        assert!(!parse_errors_block_cache(&parsed.errors));
    }

    #[test]
    fn syntax_error_leaves_ast_incomplete() {
        assert!(blocks("<?php\nfunction f( {}\n"));
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
        assert!(depth_err.leaves_ast_incomplete());
    }
}

#[cfg(test)]
mod analyzer_covered_tests {
    use super::{is_redundant_parse_error, ANALYZER_COVERED_RULES};

    /// One source per rule in `ANALYZER_COVERED_RULES`, in the same order.
    const SOURCES: &[&str] = &[
        "<?php\nclass A { function __toString(): int { return 1; } }\n",
        "<?php\n#[Attribute] trait T {}\n",
        "<?php\n#[Deprecated] class A {}\n",
        "<?php\n#[Override] class A {}\n",
        "<?php\nfunction f(): int { return; }\n",
        "<?php\nfunction f(): void { return 1; }\n",
        "<?php\nfunction f(): never { return; }\n",
        "<?php\nclass A { public readonly int $x = 1; }\n",
    ];

    #[test]
    fn every_covered_rule_matches_its_parser_diagnostic() {
        assert_eq!(SOURCES.len(), ANALYZER_COVERED_RULES.len());
        for (src, rule) in SOURCES.iter().zip(ANALYZER_COVERED_RULES) {
            let parsed = php_rs_parser::parse(src);
            let rules: Vec<_> = parsed.errors.iter().map(|e| e.forbidden_rule()).collect();
            assert_eq!(rules, [Some(*rule)], "{src:?}: {:?}", parsed.errors);
            assert!(parsed.errors.iter().all(is_redundant_parse_error));
        }
    }

    #[test]
    fn neighbouring_rules_are_not_covered() {
        for src in [
            "<?php\nclass A { function __serialize(): int { return 1; } }\n",
            "<?php\nfunction f($_GET) {}\n",
            "<?php\nfunction f() { unset($this); }\n",
            "<?php\nclass A { function f() { parent::f(); } }\n",
            "<?php\nfunction f() { self::g(); }\n",
            "<?php\nfunction f(): int { yield 1; }\n",
            "<?php\nfunction f($a, $a) {}\n",
            "<?php\nfunction f() { $this = 1; }\n",
        ] {
            let parsed = php_rs_parser::parse(src);
            assert!(
                !parsed.errors.is_empty() && !parsed.errors.iter().any(is_redundant_parse_error),
                "{src:?}: {:?}",
                parsed.errors
            );
        }
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
