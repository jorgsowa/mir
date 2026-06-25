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
/// and source map to derive a precise location. `ForbiddenWarning` diagnostics
/// become `Severity::Warning`; all other variants become `Severity::Error`.
pub(crate) fn parse_error_to_issue(
    err: &php_rs_parser::diagnostics::ParseError,
    file: &Arc<str>,
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
) -> mir_issues::Issue {
    let span = err.span();
    let (line, col_start) = crate::diagnostics::offset_to_line_col(source, span.start, source_map);
    let (line_end, col_end) = crate::diagnostics::offset_to_line_col(source, span.end, source_map);

    let mut issue = mir_issues::Issue::new(
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
    );
    if matches!(
        err.severity(),
        php_rs_parser::diagnostics::Severity::Warning
    ) {
        issue.severity = mir_issues::Severity::Warning;
    }
    issue
}

/// php-rs-parser 0.17 over-broadly rejects `numeric` and `resource` as reserved
/// class names, but PHP permits `class Numeric {}` / `class Resource {}` — only
/// `int`/`float`/`bool`/`string`/`true`/`false`/`null`/`void`/`iterable`/
/// `object`/`mixed`/`never` (plus `self`/`parent`/`static`) are truly reserved
/// as type/class names. Recognize that single spurious diagnostic so it can be
/// dropped from the issue stream and ignored when deciding whether to block
/// analysis. Matches on the parser's Display message
/// (`Cannot use "<name>" as a class name as it is reserved`).
pub(crate) fn is_spurious_reserved_class_error(
    err: &php_rs_parser::diagnostics::ParseError,
) -> bool {
    let msg = err.to_string();
    let Some(rest) = msg.strip_prefix("Cannot use \"") else {
        return false;
    };
    let Some(end) = rest.find('"') else {
        return false;
    };
    let name = &rest[..end];
    rest[end..].contains("as a class name as it is reserved")
        && matches!(name.to_ascii_lowercase().as_str(), "numeric" | "resource")
}

/// Returns `true` for parser diagnostics that should block semantic analysis.
/// `ForbiddenWarning` diagnostics are non-fatal (PHP only warns) and leave the
/// AST complete, so they do not block analysis. The spurious
/// reserved-class-name diagnostic (see [`is_spurious_reserved_class_error`]) is
/// likewise treated as non-blocking — the declaration it flags is valid PHP.
pub(crate) fn is_hard_parse_error(err: &php_rs_parser::diagnostics::ParseError) -> bool {
    matches!(err.severity(), php_rs_parser::diagnostics::Severity::Error)
        && !is_spurious_reserved_class_error(err)
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
pub(crate) fn find_preceding_docblock(source: &str, offset: u32) -> Option<String> {
    let offset = (offset as usize).min(source.len());
    if offset == 0 {
        return None;
    }
    let mut trimmed = source[..offset].trim_end();
    // Strip trailing modifier keywords like `final` or `abstract readonly`.
    loop {
        let after_ws = trimmed.trim_end();
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
