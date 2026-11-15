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
    if let Some(reason) = insecure_session_setting(fn_name_lower, args) {
        ea.emit(
            mir_issues::IssueKind::InsecureSessionConfig { reason },
            mir_issues::Severity::Warning,
            span,
        );
    }
    if let Some(reason) = weak_cipher_use(fn_name_lower, args) {
        ea.emit(
            mir_issues::IssueKind::WeakCryptography { reason },
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

// ---------------------------------------------------------------------------
// Ciphers
// ---------------------------------------------------------------------------

fn weak_cipher_use(fn_name_lower: &str, args: &[Arg]) -> Option<String> {
    if !matches!(fn_name_lower, "openssl_encrypt" | "openssl_decrypt") {
        return None;
    }
    let cipher = literal_str(arg(args, 1, "cipher_algo")).map(str::to_ascii_lowercase);
    if let Some(cipher) = &cipher {
        let broken = ["des", "rc2", "rc4", "bf", "blowfish"]
            .iter()
            .any(|family| cipher.starts_with(family));
        if broken {
            return Some(format!("cipher '{cipher}' is broken"));
        }
        if cipher.ends_with("-ecb") {
            return Some(format!(
                "ECB mode of '{cipher}' leaks patterns of the plaintext"
            ));
        }
    }
    if literal_str(arg(args, 2, "passphrase")).is_some() {
        return Some("the key is hardcoded".to_string());
    }
    if literal_str(arg(args, 4, "iv")).is_some_and(|iv| !iv.is_empty()) {
        return Some("the IV is hardcoded; use a fresh random IV for every message".to_string());
    }
    let aead = cipher.as_deref().is_some_and(|c| {
        ["-gcm", "-ccm", "-ocb"]
            .iter()
            .any(|mode| c.ends_with(mode))
            || c == "chacha20-poly1305"
    });
    (fn_name_lower == "openssl_decrypt" && aead && arg(args, 5, "tag").is_none())
        .then(|| "an AEAD cipher is decrypted without its authentication tag".to_string())
}

// ---------------------------------------------------------------------------
// Session settings
// ---------------------------------------------------------------------------

fn insecure_session_setting(fn_name_lower: &str, args: &[Arg]) -> Option<String> {
    match fn_name_lower {
        "ini_set" => {
            let name = literal_str(arg(args, 0, "option"))?.to_ascii_lowercase();
            weakened_session_setting(name.strip_prefix("session.")?, arg(args, 1, "value")?)
        }
        "session_id" if arg(args, 0, "id").is_some_and(reads_request_parameter) => Some(
            "session_id() takes the ID from a request parameter (session fixation)".to_string(),
        ),
        "session_start" => {
            let ExprKind::Array(entries) = &arg(args, 0, "options")?.kind else {
                return None;
            };
            entries.iter().find_map(|entry| {
                let key = literal_str(entry.key.as_ref())?.to_ascii_lowercase();
                weakened_session_setting(&key, entry.value())
            })
        }
        _ => None,
    }
}

/// The reason a literal `value` for session setting `name` weakens the session.
fn weakened_session_setting(name: &str, value: &Expr) -> Option<String> {
    let enabled = match &value.kind {
        ExprKind::Bool(b) => *b,
        ExprKind::Int(i) => *i != 0,
        ExprKind::String(s) => match s.as_str().to_ascii_lowercase().as_str() {
            "1" | "on" | "true" | "yes" => true,
            "0" | "off" | "false" | "no" | "" => false,
            _ => return None,
        },
        _ => return None,
    };
    match (name, enabled) {
        ("use_strict_mode" | "use_only_cookies" | "cookie_httponly", false) => {
            Some(format!("session.{name} is turned off"))
        }
        ("use_trans_sid", true) => {
            Some("session.use_trans_sid puts the session ID in URLs".to_string())
        }
        _ => None,
    }
}

/// `$_GET['x']`, `$_POST['x']` or `$_REQUEST['x']`, possibly cast or with a `??` fallback.
fn reads_request_parameter(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::ArrayAccess(aa) => {
            matches!(&aa.array.kind, ExprKind::Variable(v) if matches!(&**v, "_GET" | "_POST" | "_REQUEST"))
                || reads_request_parameter(&aa.array)
        }
        ExprKind::Parenthesized(inner) | ExprKind::Cast(_, inner) => reads_request_parameter(inner),
        ExprKind::NullCoalesce(nc) => reads_request_parameter(&nc.left),
        _ => false,
    }
}
