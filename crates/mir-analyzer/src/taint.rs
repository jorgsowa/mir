/// Taint analysis helpers (M19).
///
/// A value is "tainted" when it originates from user-controlled superglobals
/// (`$_GET`, `$_POST`, `$_REQUEST`, `$_COOKIE`, `$_FILES`, `$_SERVER`, `$_ENV`).
/// Taint propagates through assignments, string concatenation, and array access.
///
/// When tainted data reaches a sink (HTML output, SQL query, shell command),
/// the appropriate `TaintedHtml`, `TaintedSql`, or `TaintedShell` issue is emitted.
use php_ast::owned::{Expr, ExprKind, StringPart};

use crate::flow_state::FlowState;

// ---------------------------------------------------------------------------
// Superglobal names (without the $ prefix, as stored in FlowState::vars)
// ---------------------------------------------------------------------------

pub const SUPERGLOBALS: &[&str] = &[
    "_GET", "_POST", "_REQUEST", "_COOKIE", "_FILES", "_SERVER", "_ENV",
];

pub fn is_superglobal(name: &str) -> bool {
    SUPERGLOBALS.contains(&name)
}

// ---------------------------------------------------------------------------
// Sink classification
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinkKind {
    Html,        // echo / print → XSS
    Sql,         // DB query functions → SQL injection
    Shell,       // system / exec / shell_exec → command injection
    File,        // filesystem path args → path traversal / LFI / SSRF
    Unserialize, // unserialize → PHP object injection
}

impl SinkKind {
    /// The argument indices whose taint should be reported for this sink.
    ///
    /// `None` means "any argument" — used for output/query/command sinks where
    /// the dangerous payload may be in any position (e.g. `mysqli_query($db,
    /// $sql)` carries the query in arg 1). The path/payload sinks below name
    /// the single relevant argument so that, e.g., writing tainted *data* to a
    /// constant path is not flagged — only a tainted *path* is.
    pub fn tainted_arg_indices(self) -> Option<&'static [usize]> {
        match self {
            SinkKind::Html | SinkKind::Sql | SinkKind::Shell => None,
            // Path is the first argument for every File sink listed below.
            SinkKind::File => Some(&[0]),
            // unserialize($data) — the payload is the first argument.
            SinkKind::Unserialize => Some(&[0]),
        }
    }
}

/// Return the sink kind for a built-in function name, if it is a taint sink.
pub fn classify_sink(fn_name: &str) -> Option<SinkKind> {
    match fn_name.to_lowercase().as_str() {
        // HTML output
        "echo" | "print" | "printf" | "vprintf" | "fprintf" | "header" | "setcookie" => {
            Some(SinkKind::Html)
        }

        // SQL
        "mysql_query" | "mysqli_query" | "pg_query" | "pg_exec" | "sqlite_query"
        | "mssql_query" => Some(SinkKind::Sql),

        // Shell
        "system" | "exec" | "shell_exec" | "passthru" | "popen" | "proc_open" | "pcntl_exec" => {
            Some(SinkKind::Shell)
        }

        // Filesystem path (path traversal / local-file-inclusion / SSRF). Each
        // of these takes the path/URL as its first argument.
        "fopen" | "file_get_contents" | "file_put_contents" | "readfile" | "file" | "unlink" => {
            Some(SinkKind::File)
        }

        // Object injection.
        "unserialize" => Some(SinkKind::Unserialize),

        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Expression taint checker
// ---------------------------------------------------------------------------

/// Returns `true` if the expression could carry tainted data, given the
/// current `FlowState` taint state.
///
/// This is a conservative over-approximation:
/// - Any reference to a superglobal (or an array offset thereof) is tainted.
/// - Any variable that was previously marked tainted in `ctx.tainted_vars` is tainted.
/// - Binary string-concat or arithmetic on tainted operands propagates taint.
/// - Interpolated strings are tainted if any embedded variable is tainted.
pub fn is_expr_tainted(expr: &Expr, ctx: &FlowState) -> bool {
    match &expr.kind {
        ExprKind::Variable(name) => {
            let n = name.trim_start_matches('$');
            is_superglobal(n) || ctx.is_tainted(n)
        }

        ExprKind::ArrayAccess(aa) => {
            // $_GET['key'] — tainted if the array is tainted/superglobal
            is_expr_tainted(&aa.array, ctx)
        }

        ExprKind::Assign(a) => is_expr_tainted(&a.value, ctx),

        ExprKind::Binary(op) => is_expr_tainted(&op.left, ctx) || is_expr_tainted(&op.right, ctx),

        ExprKind::UnaryPrefix(u) => is_expr_tainted(&u.operand, ctx),

        ExprKind::InterpolatedString(parts) | ExprKind::Heredoc { parts, .. } => {
            parts.iter().any(|p| match p {
                StringPart::Expr(e) => is_expr_tainted(e, ctx),
                StringPart::Literal(_) => false,
            })
        }

        ExprKind::Ternary(t) => {
            t.then_expr
                .as_deref()
                .is_some_and(|e| is_expr_tainted(e, ctx))
                || is_expr_tainted(&t.else_expr, ctx)
        }

        ExprKind::Cast(_kind, inner) => is_expr_tainted(inner, ctx),

        // Conservative: function call results are not tracked as tainted
        // unless it's a known pass-through built-in (htmlspecialchars sanitizes)
        _ => false,
    }
}
