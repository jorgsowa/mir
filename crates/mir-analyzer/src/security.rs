//! Misuse of security-sensitive builtins that shows in constant arguments.
use php_ast::owned::{Arg, Expr, ExprKind};

use crate::ast_ext::ByteStringExt as _;
use crate::ast_ext::NameBareExt as _;

/// The argument at `index`, or the one named `name`.
fn arg<'e>(args: &'e [Arg], index: usize, name: &str) -> Option<&'e Expr> {
    args.iter()
        .find(|a| a.name.as_ref().is_some_and(|n| n.last_part() == name))
        .or_else(|| args.get(index).filter(|a| a.name.is_none()))
        .and_then(|a| a.value())
}

fn literal_str(expr: Option<&Expr>) -> Option<&str> {
    match &expr?.kind {
        ExprKind::String(s) => Some(s.as_str()),
        _ => None,
    }
}

/// The value under string key `key` of a literal array.
fn array_entry<'e>(array: &'e Expr, key: &str) -> Option<&'e Expr> {
    let ExprKind::Array(entries) = &array.kind else {
        return None;
    };
    entries.iter().find_map(|entry| {
        matches!(
            entry.key.as_ref().map(|k| &k.kind),
            Some(ExprKind::String(k)) if k.as_str() == key
        )
        .then(|| entry.value())
    })
}

/// Report misuse of a security-sensitive builtin visible in its arguments.
pub(crate) fn check_builtin_call(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    fn_name_lower: &str,
    args: &[Arg],
    span: php_ast::Span,
) {
    if let Some(reason) = password_hash_misuse(fn_name_lower, args) {
        ea.emit(
            mir_issues::IssueKind::InsecurePasswordHash { reason },
            mir_issues::Severity::Warning,
            span,
        );
    }
}

// ---------------------------------------------------------------------------
// Password hashing
// ---------------------------------------------------------------------------

fn password_hash_misuse(fn_name_lower: &str, args: &[Arg]) -> Option<String> {
    if fn_name_lower != "password_hash" {
        return None;
    }
    let bcrypt = match arg(args, 1, "algo").map(|a| &a.kind) {
        None | Some(ExprKind::Null) => true,
        Some(ExprKind::String(s)) => s.as_str() == "2y",
        Some(algo) => matches!(
            algo.name_bare(),
            Some("PASSWORD_BCRYPT" | "PASSWORD_DEFAULT")
        ),
    };
    if bcrypt && arg(args, 0, "password").is_some_and(is_raw_digest) {
        return Some(
            "a raw binary digest can contain NUL bytes, where bcrypt stops reading".to_string(),
        );
    }
    let options = arg(args, 2, "options")?;
    if array_entry(options, "salt").is_some() {
        return Some("password_hash() ignores the 'salt' option since PHP 8.0".to_string());
    }
    match array_entry(options, "cost").map(|c| &c.kind) {
        Some(ExprKind::Int(cost)) if bcrypt && *cost < 10 => {
            Some(format!("bcrypt cost {cost} is below 10"))
        }
        _ => None,
    }
}

/// `md5($x, true)`, `sha1($x, true)`, `hash($algo, $x, true)` or `hash_hmac($algo, $x, $key, true)`.
fn is_raw_digest(expr: &Expr) -> bool {
    let ExprKind::FunctionCall(fc) = &expr.kind else {
        return false;
    };
    let Some(name) = fc.name.kind.name_bare().map(str::to_ascii_lowercase) else {
        return false;
    };
    let binary = match name.as_str() {
        "md5" | "sha1" => arg(&fc.args, 1, "binary"),
        "hash" => arg(&fc.args, 2, "binary"),
        "hash_hmac" => arg(&fc.args, 3, "binary"),
        _ => return false,
    };
    matches!(binary.map(|b| &b.kind), Some(ExprKind::Bool(true)))
}
