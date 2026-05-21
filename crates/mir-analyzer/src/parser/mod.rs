pub mod docblock;
pub(crate) mod type_from_hint;

use std::sync::Arc;

use php_ast::Span;
use php_rs_parser::ParserContext;
use thiserror::Error;

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

/// Returns `true` for parser diagnostics that should block semantic analysis.
/// `ForbiddenWarning` diagnostics are non-fatal (PHP only warns) and leave the
/// AST complete, so they do not block analysis.
pub(crate) fn is_hard_parse_error(err: &php_rs_parser::diagnostics::ParseError) -> bool {
    matches!(err.severity(), php_rs_parser::diagnostics::Severity::Error)
}

// ---------------------------------------------------------------------------
// ParseError
// ---------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Debug, Error)]
pub(crate) enum ParseError {
    #[error("PHP parse error in {file}: {message}")]
    SyntaxError { file: Arc<str>, message: String },
}

// ---------------------------------------------------------------------------
// ParsedFile — result of parsing a single PHP file
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub(crate) struct ParsedFile<'arena, 'src> {
    pub(crate) program: php_ast::ast::Program<'arena, 'src>,
    pub(crate) errors: Vec<ParseError>,
    pub(crate) file: Arc<str>,
}

// ---------------------------------------------------------------------------
// FileParser
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub(crate) struct FileParser {
    ctx: ParserContext,
}

impl FileParser {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self {
            ctx: ParserContext::new(),
        }
    }

    /// Parse a PHP source string, reusing the internal arena (O(1) reset).
    /// The returned `ParsedFile` borrows from both `self` and `src`.
    /// The previous `ParsedFile` must be dropped before calling `parse` again.
    #[allow(dead_code)]
    pub(crate) fn parse<'arena, 'src>(
        &'arena mut self,
        src: &'src str,
        file: Arc<str>,
    ) -> ParsedFile<'arena, 'src> {
        let result = self.ctx.reparse(src);
        let errors = result
            .errors
            .iter()
            .map(|e| ParseError::SyntaxError {
                file: file.clone(),
                message: e.to_string(),
            })
            .collect();

        ParsedFile {
            program: result.program,
            errors,
            file,
        }
    }
}

impl Default for FileParser {
    fn default() -> Self {
        Self::new()
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

/// Extract the source line containing a span.
#[allow(dead_code)]
pub(crate) fn span_snippet(src: &str, span: Span) -> String {
    let offset = span.start as usize;
    let line_start = src[..offset].rfind('\n').map(|p| p + 1).unwrap_or(0);
    let line_end = src[offset..]
        .find('\n')
        .map(|p| offset + p)
        .unwrap_or(src.len());
    src[line_start..line_end].to_string()
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
    let start = trimmed[..end].rfind("/**")?;
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
