pub mod docblock;
pub mod type_from_hint;

use std::sync::Arc;

use php_ast::Span;
use php_rs_parser::parse;
use thiserror::Error;

pub use docblock::{DocblockParser, ParsedDocblock};
pub use type_from_hint::type_from_hint;

// ---------------------------------------------------------------------------
// ParseError
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("PHP parse error in {file}: {message}")]
    SyntaxError { file: Arc<str>, message: String },
}

// ---------------------------------------------------------------------------
// ParsedFile — result of parsing a single PHP file
// ---------------------------------------------------------------------------

pub struct ParsedFile<'arena, 'src> {
    pub program: php_ast::ast::Program<'arena, 'src>,
    pub errors: Vec<ParseError>,
    pub file: Arc<str>,
}

// ---------------------------------------------------------------------------
// FileParser
// ---------------------------------------------------------------------------

pub struct FileParser {
    pub arena: bumpalo::Bump,
}

impl FileParser {
    pub fn new() -> Self {
        Self {
            arena: bumpalo::Bump::new(),
        }
    }

    /// Parse a PHP source string.
    /// The returned `ParsedFile` borrows from both `self.arena` and `src`.
    /// The arena must outlive the parsed file.
    pub fn parse<'arena, 'src>(
        &'arena self,
        src: &'src str,
        file: Arc<str>,
    ) -> ParsedFile<'arena, 'src> {
        let result = parse(&self.arena, src);
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

/// Convert a byte-offset `Span` to a `(line, col)` pair given the source.
pub fn span_to_line_col(src: &str, span: Span) -> (u32, u16) {
    let offset = span.start as usize;
    let before = &src[..offset.min(src.len())];
    let line = before.bytes().filter(|&b| b == b'\n').count() as u32 + 1;
    let col = before.rfind('\n').map(|p| offset - p - 1).unwrap_or(offset) as u16;
    (line, col)
}

/// Extract the exact source text covered by a span.
pub fn span_text(src: &str, span: Span) -> Option<String> {
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
pub fn span_snippet(src: &str, span: Span) -> String {
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
/// The docblock must be separated from the declaration only by whitespace.
pub fn find_preceding_docblock(source: &str, offset: u32) -> Option<String> {
    let offset = (offset as usize).min(source.len());
    if offset == 0 {
        return None;
    }
    let before = &source[..offset];
    let trimmed = before.trim_end();
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

pub fn name_to_string(name: &php_ast::ast::Name<'_, '_>) -> String {
    name.to_string_repr().into_owned()
}
