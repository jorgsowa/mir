use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// Source location: file path and a Unicode code-point span.
///
/// Columns are 0-based Unicode scalar value (code-point) counts, equivalent to
/// LSP `utf-32` position encoding. Convert to UTF-16 code units at the LSP
/// boundary for clients that do not advertise `utf-32` support.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub file: Arc<str>,
    /// 1-based start line.
    pub line: u32,
    /// 1-based end line (inclusive). Equal to `line` for single-line spans.
    pub line_end: u32,
    /// 0-based Unicode code-point column of the span start.
    pub col_start: u16,
    /// 0-based Unicode code-point column of the span end (exclusive).
    pub col_end: u16,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.col_start)
    }
}
