/// Taint analysis helpers (M19).
///
/// A value is "tainted" when it originates from user-controlled superglobals
/// (`$_GET`, `$_POST`, `$_REQUEST`, `$_COOKIE`, `$_FILES`, `$_SERVER`, `$_ENV`).
/// Taint propagates through assignments, string concatenation, and array access.
///
/// When tainted data reaches a sink (HTML output, SQL query, shell command),
/// the appropriate `TaintedHtml`, `TaintedSql`, or `TaintedShell` issue is emitted.
use php_ast::ast::{Expr, ExprKind, StringPart};

use crate::context::Context;

// ---------------------------------------------------------------------------
// Superglobal names (without the $ prefix, as stored in Context::vars)
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
    Html,  // echo / print → XSS
    Sql,   // DB query functions → SQL injection
    Shell, // system / exec / shell_exec → command injection
}

/// Return the sink kind for a built-in function name, if it is a taint sink.
pub fn classify_sink(fn_name: &str) -> Option<SinkKind> {
    match fn_name.to_lowercase().as_str() {
        // HTML output
        "echo" | "print" | "printf" | "vprintf" | "fprintf"
        | "header" | "setcookie" => Some(SinkKind::Html),

        // SQL
        "mysql_query" | "mysqli_query" | "pg_query" | "pg_exec"
        | "sqlite_query" | "mssql_query" => Some(SinkKind::Sql),

        // Shell
        "system" | "exec" | "shell_exec" | "passthru" | "popen"
        | "proc_open" | "pcntl_exec" => Some(SinkKind::Shell),

        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Expression taint checker
// ---------------------------------------------------------------------------

/// Returns `true` if the expression could carry tainted data, given the
/// current `Context` taint state.
///
/// This is a conservative over-approximation:
/// - Any reference to a superglobal (or an array offset thereof) is tainted.
/// - Any variable that was previously marked tainted in `ctx.tainted_vars` is tainted.
/// - Binary string-concat or arithmetic on tainted operands propagates taint.
/// - Interpolated strings are tainted if any embedded variable is tainted.
pub fn is_expr_tainted<'arena, 'src>(expr: &Expr<'arena, 'src>, ctx: &Context) -> bool {
    match &expr.kind {
        ExprKind::Variable(name) => {
            let n = name.as_ref().trim_start_matches('$');
            is_superglobal(n) || ctx.is_tainted(n)
        }

        ExprKind::ArrayAccess(aa) => {
            // $_GET['key'] — tainted if the array is tainted/superglobal
            is_expr_tainted(aa.array, ctx)
        }

        ExprKind::Assign(a) => is_expr_tainted(a.value, ctx),

        ExprKind::Binary(op) => {
            is_expr_tainted(op.left, ctx) || is_expr_tainted(op.right, ctx)
        }

        ExprKind::UnaryPrefix(u) => is_expr_tainted(u.operand, ctx),

        ExprKind::InterpolatedString(parts) | ExprKind::Heredoc { parts, .. } => {
            parts.iter().any(|p| match p {
                StringPart::Expr(e) => is_expr_tainted(e, ctx),
                StringPart::Literal(_) => false,
            })
        }

        ExprKind::Ternary(t) => {
            t.then_expr.is_some_and(|e| is_expr_tainted(e, ctx))
                || is_expr_tainted(t.else_expr, ctx)
        }

        ExprKind::Cast(_kind, inner) => is_expr_tainted(inner, ctx),

        // Conservative: function call results are not tracked as tainted
        // unless it's a known pass-through built-in (htmlspecialchars sanitizes)
        _ => false,
    }
}
