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
    match crate::util::php_ident_lowercase(fn_name).as_str() {
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

/// Return the sink kind for a *method* call on a known OOP database wrapper —
/// `$pdo->query($sql)`, `$mysqli->prepare($sql)` — the dominant modern PHP
/// idiom over the procedural `mysqli_query()` style `classify_sink` covers.
/// Matches the class itself or any subclass (a custom `Database extends PDO`
/// is just as much a sink).
pub fn classify_method_sink(
    db: &dyn crate::db::MirDatabase,
    fqcn: &str,
    method_name: &str,
) -> Option<SinkKind> {
    let method = crate::util::php_ident_lowercase(method_name);
    let is_a = |base: &str| fqcn.eq_ignore_ascii_case(base) || crate::db::extends_or_implements(db, fqcn, base);

    if is_a("PDO") && matches!(method.as_str(), "query" | "exec" | "prepare") {
        return Some(SinkKind::Sql);
    }
    if is_a("mysqli")
        && matches!(
            method.as_str(),
            "query" | "prepare" | "real_query" | "multi_query"
        )
    {
        return Some(SinkKind::Sql);
    }
    if is_a("SQLite3") && matches!(method.as_str(), "query" | "exec" | "prepare") {
        return Some(SinkKind::Sql);
    }
    None
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

        // $obj->prop — tainted if this property was previously assigned a
        // tainted value (see FlowState::taint_prop, set on property writes
        // in expr/assignment.rs). Only a simple-variable receiver is tracked,
        // matching prop_refined's narrowing scope.
        ExprKind::PropertyAccess(pa) => {
            if let ExprKind::Variable(obj_var) = &pa.object.kind {
                if let Some(prop_name) =
                    crate::expr::helpers::extract_string_from_expr(&pa.property)
                {
                    return ctx.is_prop_tainted(obj_var.trim_start_matches('$'), &prop_name);
                }
            }
            false
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

        // Numeric/boolean casts sanitize: PHP coerces the value to that scalar
        // type, so a subsequent SQL/shell/HTML sink can no longer receive
        // arbitrary attacker-controlled text through it. String/array/object
        // casts don't remove the payload, so taint still propagates.
        ExprKind::Cast(kind, inner) => match kind {
            php_ast::ast::CastKind::Int
            | php_ast::ast::CastKind::Float
            | php_ast::ast::CastKind::Bool => false,
            _ => is_expr_tainted(inner, ctx),
        },

        ExprKind::Match(m) => m.arms.iter().any(|arm| is_expr_tainted(&arm.body, ctx)),

        ExprKind::Array(elements) => elements.iter().any(|el| is_expr_tainted(&el.value, ctx)),

        // Conservative: function call results are not tracked as tainted
        // unless it's a known pass-through built-in (htmlspecialchars sanitizes)
        _ => false,
    }
}
