//! Misuse of security-sensitive builtins that shows in constant arguments.
use php_ast::owned::{Arg, Expr, ExprKind};

use crate::ast_ext::ByteStringExt as _;
use crate::ast_ext::NameBareExt as _;
use crate::flow_state::{FlowState, ValueOrigin};

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
    ctx: &FlowState,
    span: php_ast::Span,
) {
    if fn_name_lower == "session_id" && arg(args, 0, "id").is_some_and(|id| is_weak_random(id, ctx))
    {
        ea.emit(
            mir_issues::IssueKind::InsecureRandomness {
                context: "the session ID".to_string(),
            },
            mir_issues::Severity::Warning,
            span,
        );
    }
    if fn_name_lower == "unserialize" && unserializes_any_class(ea, args, ctx) {
        ea.emit(
            mir_issues::IssueKind::UnsafeUnserialize,
            mir_issues::Severity::Warning,
            span,
        );
    }
    if matches!(
        fn_name_lower,
        "system" | "exec" | "shell_exec" | "passthru" | "popen" | "proc_open"
    ) {
        if let Some(command) = arg(args, 0, "command") {
            check_quoted_shell_escape(ea, command, ctx, span);
        }
    }
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

/// A request superglobal or a value read from one, possibly cast, trimmed or with a `??`
/// fallback, or a variable holding such a value.
pub(crate) fn is_request_input(expr: &Expr, ctx: &FlowState) -> bool {
    match &expr.kind {
        ExprKind::Variable(v) => {
            matches!(
                v.trim_start_matches('$'),
                "_GET" | "_POST" | "_REQUEST" | "_COOKIE"
            ) || ctx.has_value_origin(v, ValueOrigin::RequestInput)
        }
        ExprKind::ArrayAccess(aa) => is_request_input(&aa.array, ctx),
        ExprKind::Parenthesized(inner) | ExprKind::Cast(_, inner) => is_request_input(inner, ctx),
        ExprKind::NullCoalesce(nc) => is_request_input(&nc.left, ctx),
        ExprKind::FunctionCall(fc) => {
            named_one_of(
                &fc.name,
                &["trim", "intval", "strval", "wp_unslash", "stripslashes"],
            ) && fc
                .args
                .first()
                .and_then(|a| a.value())
                .is_some_and(|v| is_request_input(v, ctx))
        }
        _ => false,
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

// ---------------------------------------------------------------------------
// Randomness
// ---------------------------------------------------------------------------

/// Report a guessable random value returned from a function named for a secret.
pub(crate) fn check_weak_random_return(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    value: &Expr,
    ctx: &FlowState,
    span: php_ast::Span,
) {
    let name = match (&ctx.current_method_name, &ctx.current_function_fqn) {
        (Some(method), _) => method.as_ref(),
        (None, Some(function)) => function.rsplit('\\').next().unwrap_or(function),
        (None, None) => return,
    };
    if is_weak_random(value, ctx) && names_secret(name) {
        ea.emit(
            mir_issues::IssueKind::InsecureRandomness {
                context: format!("the value {name}() returns"),
            },
            mir_issues::Severity::Warning,
            span,
        );
    }
}

/// Whether a function name ends in a word for a secret: `generateToken`, `make_salt`, `newApiKey`.
fn names_secret(name: &str) -> bool {
    let mut words: Vec<String> = Vec::new();
    for (i, c) in name.char_indices() {
        if c == '_' {
            words.push(String::new());
        } else if words.is_empty() || (c.is_ascii_uppercase() && i > 0) {
            words.push(c.to_ascii_lowercase().to_string());
        } else if let Some(word) = words.last_mut() {
            word.push(c.to_ascii_lowercase());
        }
    }
    words.retain(|w| !w.is_empty());
    let last = words.last().map(String::as_str);
    let last_two =
        (words.len() >= 2).then(|| format!("{}{}", words[words.len() - 2], words[words.len() - 1]));
    matches!(
        last,
        Some("token" | "secret" | "password" | "passwd" | "nonce" | "salt" | "otp")
    ) || matches!(
        last_two.as_deref(),
        Some("apikey" | "resetcode" | "verificationcode" | "confirmationcode")
    )
}

/// A value from a non-cryptographic generator, possibly hashed, encoded or concatenated,
/// with no cryptographically secure part mixed in.
pub(crate) fn is_weak_random(expr: &Expr, ctx: &FlowState) -> bool {
    has_weak_random(expr, ctx) && !has_secure_random(expr)
}

fn has_weak_random(expr: &Expr, ctx: &FlowState) -> bool {
    match &expr.kind {
        ExprKind::Variable(name) => ctx.has_value_origin(name, ValueOrigin::WeakRandom),
        ExprKind::FunctionCall(fc) => match random_call(&fc.name) {
            Some(Generator::Weak) => true,
            Some(Generator::Secure) => false,
            None => {
                named_one_of(&fc.name, DERIVED_FROM_ARGS)
                    && fc
                        .args
                        .iter()
                        .filter_map(|a| a.value())
                        .any(|v| has_weak_random(v, ctx))
            }
        },
        ExprKind::ArrayAccess(aa) => aa.index.as_ref().is_some_and(|i| has_weak_random(i, ctx)),
        _ => any_string_part(expr, &mut |part| has_weak_random(part, ctx)),
    }
}

fn has_secure_random(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::FunctionCall(fc) => {
            random_call(&fc.name) == Some(Generator::Secure)
                || fc
                    .args
                    .iter()
                    .filter_map(|a| a.value())
                    .any(has_secure_random)
        }
        ExprKind::ArrayAccess(aa) => aa.index.as_ref().is_some_and(has_secure_random),
        _ => any_string_part(expr, &mut has_secure_random),
    }
}

#[derive(PartialEq)]
enum Generator {
    Weak,
    Secure,
}

const WEAK_GENERATORS: &[&str] = &[
    "rand",
    "mt_rand",
    "uniqid",
    "lcg_value",
    "str_shuffle",
    "array_rand",
];
const SECURE_GENERATORS: &[&str] = &["random_bytes", "random_int", "openssl_random_pseudo_bytes"];
/// Builtins whose result is a function of their arguments' content.
const DERIVED_FROM_ARGS: &[&str] = &[
    "md5",
    "sha1",
    "hash",
    "crc32",
    "base64_encode",
    "bin2hex",
    "dechex",
    "base_convert",
    "substr",
    "str_pad",
    "strtoupper",
    "strtolower",
    "sprintf",
    "strval",
    "implode",
    "str_replace",
];

fn named_one_of(name: &Expr, names: &[&str]) -> bool {
    name.kind.name_bare().is_some_and(|n| {
        names
            .iter()
            .any(|candidate| n.eq_ignore_ascii_case(candidate))
    })
}

fn random_call(name: &Expr) -> Option<Generator> {
    if named_one_of(name, WEAK_GENERATORS) {
        Some(Generator::Weak)
    } else if named_one_of(name, SECURE_GENERATORS) {
        Some(Generator::Secure)
    } else {
        None
    }
}

/// Whether `f` holds for a sub-expression a string-building expression copies into its result.
fn any_string_part(expr: &Expr, f: &mut dyn FnMut(&Expr) -> bool) -> bool {
    match &expr.kind {
        ExprKind::Parenthesized(inner) | ExprKind::Cast(_, inner) => f(inner),
        ExprKind::Binary(b) if b.op == php_ast::ast::BinaryOp::Concat => f(&b.left) || f(&b.right),
        ExprKind::InterpolatedString(_) | ExprKind::Heredoc(_) => {
            crate::ast_ext::StringPartsExt::string_parts(&expr.kind)
                .iter()
                .any(|part| matches!(part, php_ast::owned::StringPart::Expr(e) if f(e)))
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Unserialize
// ---------------------------------------------------------------------------

/// `unserialize()` of non-literal data that may create any class; tainted data is left to
/// `TaintedUnserialize`.
fn unserializes_any_class(
    ea: &crate::expr::ExpressionAnalyzer<'_>,
    args: &[Arg],
    ctx: &FlowState,
) -> bool {
    let Some(data) = arg(args, 0, "data") else {
        return false;
    };
    !matches!(data.kind, ExprKind::String(_) | ExprKind::Nowdoc(_))
        && !crate::taint::unserialize_disallows_classes(args, ctx)
        && !crate::taint::expr_taint(data, ctx, ea.db, &ea.file)
            .concrete()
            .intersects(crate::taint::Taint::UNSERIALIZE)
}

/// Report a magic method that runs on unserialized objects passing `$this->prop` to a sink.
pub(crate) fn check_gadget_sinks(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    sink_values: &[(&Expr, crate::taint::SinkKind)],
    ctx: &FlowState,
    span: php_ast::Span,
) {
    use crate::taint::SinkKind;
    let (Some(method), Some(class)) = (&ctx.current_method_name, &ctx.self_fqcn) else {
        return;
    };
    let runs_on_unserialized = ["__destruct", "__wakeup", "__unserialize", "__tostring"]
        .iter()
        .any(|m| method.eq_ignore_ascii_case(m));
    let reaches_gadget_sink = sink_values.iter().any(|(value, kind)| {
        matches!(
            kind,
            SinkKind::File
                | SinkKind::Shell
                | SinkKind::Callable
                | SinkKind::Unserialize
                | SinkKind::Ssrf
                | SinkKind::Sql
        ) && copies_this_property(value)
    });
    if runs_on_unserialized && reaches_gadget_sink {
        ea.emit(
            mir_issues::IssueKind::UnserializeGadget {
                class: class.to_string(),
                method: method.to_string(),
            },
            mir_issues::Severity::Warning,
            span,
        );
    }
}

/// Whether the value is, or is built by concatenation from, a `$this->prop` read.
fn copies_this_property(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa) => {
            matches!(&pa.object.kind, ExprKind::Variable(v) if &**v == "this")
                || copies_this_property(&pa.object)
        }
        ExprKind::ArrayAccess(aa) => copies_this_property(&aa.array),
        ExprKind::Array(elements) => elements.iter().any(|el| copies_this_property(el.value())),
        _ => any_string_part(expr, &mut copies_this_property),
    }
}

// ---------------------------------------------------------------------------
// Shell quoting
// ---------------------------------------------------------------------------

/// Report an `escapeshellarg()` result placed between quotes in a shell command:
/// `"ls '" . escapeshellarg($dir) . "'"`. Its own quotes then close the outer ones.
pub(crate) fn check_quoted_shell_escape(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    command: &Expr,
    ctx: &FlowState,
    span: php_ast::Span,
) {
    let mut parts = Vec::new();
    command_parts(command, &mut parts);
    let quoted = parts.windows(3).any(|w| match w {
        [CommandPart::Text(before), CommandPart::Value(value), CommandPart::Text(after)] => {
            let open = before.chars().last();
            matches!(open, Some('\'' | '"'))
                && after.chars().next() == open
                && is_shell_escaped(value, ctx)
        }
        _ => false,
    });
    if quoted {
        ea.emit(
            mir_issues::IssueKind::QuotedShellEscape,
            mir_issues::Severity::Warning,
            span,
        );
    }
}

enum CommandPart<'e> {
    Text(&'e str),
    Value(&'e Expr),
}

fn command_parts<'e>(expr: &'e Expr, out: &mut Vec<CommandPart<'e>>) {
    match &expr.kind {
        ExprKind::Parenthesized(inner) => command_parts(inner, out),
        ExprKind::Binary(b) if b.op == php_ast::ast::BinaryOp::Concat => {
            command_parts(&b.left, out);
            command_parts(&b.right, out);
        }
        ExprKind::String(s) => out.push(CommandPart::Text(s.as_str())),
        ExprKind::InterpolatedString(_) | ExprKind::Heredoc(_) | ExprKind::ShellExec(_) => {
            for part in crate::ast_ext::StringPartsExt::string_parts(&expr.kind) {
                match part {
                    php_ast::owned::StringPart::Literal(s) => {
                        out.push(CommandPart::Text(s.as_str()))
                    }
                    php_ast::owned::StringPart::Expr(e) => out.push(CommandPart::Value(e)),
                    _ => out.push(CommandPart::Text("")),
                }
            }
        }
        _ => out.push(CommandPart::Value(expr)),
    }
}

fn is_shell_escaped(expr: &Expr, ctx: &FlowState) -> bool {
    match &expr.kind {
        ExprKind::Variable(name) => ctx.has_value_origin(name, ValueOrigin::ShellEscaped),
        ExprKind::FunctionCall(fc) => named_one_of(&fc.name, &["escapeshellarg"]),
        _ => false,
    }
}
