//! Misuse of security-sensitive builtins that shows in constant arguments.
use php_ast::owned::{Arg, Expr, ExprKind};

use crate::ast_ext::ArgExt as _;
use crate::ast_ext::ByteStringExt as _;
use crate::ast_ext::NameBareExt as _;
use crate::flow_state::{FlowState, OriginSet, ValueOrigin};
use crate::security_value::{Value, Values};

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

/// Report misuse of a security-sensitive builtin visible in its arguments.
pub(crate) fn check_builtin_call(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    fn_name_lower: &str,
    args: &[Arg],
    ctx: &mut FlowState,
    span: php_ast::Span,
) {
    let file = ea.file.clone();
    let values = Values {
        ctx: &*ctx,
        db: ea.db,
        file: &file,
    };
    let constant_findings: Vec<mir_issues::IssueKind> = [
        hardcoded_password(fn_name_lower, args, &values)
            .map(|what| mir_issues::IssueKind::HardcodedCredential { what }),
        world_writable_mode(fn_name_lower, args, &values)
            .map(|reason| mir_issues::IssueKind::InsecureFilePermissions { reason }),
        password_hash_misuse(fn_name_lower, args, &values)
            .map(|reason| mir_issues::IssueKind::InsecurePasswordHash { reason }),
        insecure_session_setting(fn_name_lower, args, &values)
            .map(|reason| mir_issues::IssueKind::InsecureSessionConfig { reason }),
        weak_cipher_use(fn_name_lower, args, &values)
            .map(|reason| mir_issues::IssueKind::WeakCryptography { reason }),
        disabled_tls_verification(fn_name_lower, args, &values)
            .filter(|_| !ctx.tls_opt_out)
            .map(|reason| mir_issues::IssueKind::InsecureTls { reason }),
    ]
    .into_iter()
    .flatten()
    .collect();
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
    if let Some(amount) = resource_amount_arg(fn_name_lower, args) {
        if is_request_value(ea, amount, ctx) {
            ea.emit(
                mir_issues::IssueKind::ResourceExhaustion {
                    function: fn_name_lower.to_string(),
                },
                mir_issues::Severity::Warning,
                span,
            );
        }
    }
    if matches!(fn_name_lower, "mail" | "mb_send_mail")
        && arg(args, 2, "message").is_some_and(|body| is_host_derived(body, ctx))
    {
        ea.emit(
            mir_issues::IssueKind::HostHeaderInjection,
            mir_issues::Severity::Warning,
            span,
        );
    }
    if checks_uploaded_file(fn_name_lower, args, ctx) {
        ctx.upload_checked = true;
    }
    if fn_name_lower == "move_uploaded_file"
        && arg(args, 1, "to").is_some_and(|to| is_client_file_name(to, ctx))
        && !ctx.upload_checked
    {
        ea.emit(
            mir_issues::IssueKind::UnrestrictedFileUpload,
            mir_issues::Severity::Warning,
            span,
        );
    }
    if fn_name_lower == "header" {
        check_cors_header(ea, args, ctx, span);
    }
    let code_arg = match fn_name_lower {
        "assert" => arg(args, 0, "assertion"),
        "create_function" => arg(args, 1, "code"),
        "call_user_func" | "call_user_func_array" => arg(args, 0, "callback"),
        _ => None,
    };
    if let Some(code) = code_arg {
        check_obfuscated_code(ea, code, ctx, span);
    }
    if matches!(fn_name_lower, "mt_srand" | "srand")
        && arg(args, 0, "seed").is_some_and(is_predictable_seed)
    {
        ea.emit(
            mir_issues::IssueKind::PredictableSeed {
                function: fn_name_lower.to_string(),
            },
            mir_issues::Severity::Warning,
            span,
        );
    }
    for kind in constant_findings {
        ea.emit(kind, mir_issues::Severity::Warning, span);
    }
    if matches!(fn_name_lower, "password_hash" | "password_verify") {
        ea.password_prehash = arg(args, 0, "password").map(|p| p.span);
    }
    let in_prehash = ea
        .password_prehash
        .is_some_and(|p| p.start <= span.start && span.end <= p.end);
    if let Some(reason) = fast_password_digest(fn_name_lower, args, in_prehash) {
        ea.emit(
            mir_issues::IssueKind::InsecurePasswordHash { reason },
            mir_issues::Severity::Warning,
            span,
        );
    }
    if fn_name_lower == "session_id"
        && arg(args, 0, "id").is_some_and(|id| reads_request_parameter(id, ctx))
    {
        ea.emit(
            mir_issues::IssueKind::InsecureSessionConfig {
                reason: "session_id() takes the ID from a request parameter (session fixation)"
                    .to_string(),
            },
            mir_issues::Severity::Warning,
            span,
        );
    }
}

// ---------------------------------------------------------------------------
// TLS verification
// ---------------------------------------------------------------------------

/// A curl option or `ssl` stream context option, written as a literal, that turns off checking
/// the server's certificate or host name.
fn disabled_tls_verification(fn_name_lower: &str, args: &[Arg], v: &Values) -> Option<String> {
    let value_of = |index: usize, name: &str| v.of(arg(args, index, name)?);
    match fn_name_lower {
        "curl_setopt" => curl_option_off(&value_of(1, "option")?, &value_of(2, "value")?),
        "curl_setopt_array" => value_of(1, "options")?
            .entries()
            .iter()
            .find_map(|(k, val)| curl_option_off(k.as_ref()?, val.as_ref()?)),
        "stream_context_create" => ssl_options_off(&value_of(0, "options")?),
        "stream_context_set_option" => {
            let second = value_of(1, "wrapper_or_options")?;
            if second
                .as_str()
                .is_some_and(|w| w.eq_ignore_ascii_case("ssl"))
            {
                let name = value_of(2, "option_name")?;
                ssl_option_off(name.as_str()?, &value_of(3, "value")?)
            } else {
                ssl_options_off(&second)
            }
        }
        "stream_context_set_options" => ssl_options_off(&value_of(1, "options")?),
        _ => None,
    }
}

/// A condition deciding on a TLS verification setting, such as `$options['verify'] === false`
/// or `$this->sslVerify`: the code under it honours a caller's opt-out.
pub(crate) fn mark_tls_opt_out(cond: &Expr, ctx: &mut FlowState) {
    if mentions_tls_setting(cond) {
        ctx.tls_opt_out = true;
    }
}

fn mentions_tls_setting(expr: &Expr) -> bool {
    let names_setting = |name: &str| {
        let lower = name.to_ascii_lowercase();
        ["verif", "ssl", "tls", "insecure", "cert"]
            .iter()
            .any(|w| lower.contains(w))
    };
    match &expr.kind {
        ExprKind::Variable(v) => names_setting(v),
        ExprKind::String(s) => names_setting(s.as_str()),
        ExprKind::Identifier(name) => names_setting(name),
        ExprKind::Name(_) => expr.kind.name_bare().is_some_and(names_setting),
        ExprKind::ArrayAccess(aa) => {
            mentions_tls_setting(&aa.array) || aa.index.as_ref().is_some_and(mentions_tls_setting)
        }
        ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa) => {
            mentions_tls_setting(&pa.object) || mentions_tls_setting(&pa.property)
        }
        ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => {
            mentions_tls_setting(&mc.method)
                || call_values(&mc.args).into_iter().any(mentions_tls_setting)
        }
        ExprKind::FunctionCall(fc) => {
            mentions_tls_setting(&fc.name)
                || call_values(&fc.args).into_iter().any(mentions_tls_setting)
        }
        ExprKind::Binary(b) => mentions_tls_setting(&b.left) || mentions_tls_setting(&b.right),
        ExprKind::UnaryPrefix(u) => mentions_tls_setting(&u.operand),
        ExprKind::Parenthesized(inner) | ExprKind::Empty(inner) => mentions_tls_setting(inner),
        ExprKind::Isset(items) => items.iter().any(mentions_tls_setting),
        _ => false,
    }
}

fn is_off(value: &Value) -> bool {
    matches!(value, Value::Bool(false) | Value::Int(0))
}

fn curl_option_off(option: &Value, value: &Value) -> Option<String> {
    let (name, what) = match option.as_int()? {
        64 => ("CURLOPT_SSL_VERIFYPEER", "any certificate is accepted"),
        81 => (
            "CURLOPT_SSL_VERIFYHOST",
            "a certificate for any host is accepted",
        ),
        248 => (
            "CURLOPT_PROXY_SSL_VERIFYPEER",
            "any proxy certificate is accepted",
        ),
        249 => (
            "CURLOPT_PROXY_SSL_VERIFYHOST",
            "a proxy certificate for any host is accepted",
        ),
        _ => return None,
    };
    is_off(value).then(|| format!("{name} is off, so {what}"))
}

fn ssl_options_off(options: &Value) -> Option<String> {
    options
        .entry("ssl")??
        .entries()
        .iter()
        .find_map(|(k, val)| ssl_option_off(k.as_ref()?.as_str()?, val.as_ref()?))
}

fn ssl_option_off(name: &str, value: &Value) -> Option<String> {
    let what = match name {
        "verify_peer" => "any certificate is accepted",
        "verify_peer_name" => "a certificate for any host is accepted",
        _ => return None,
    };
    is_off(value).then(|| format!("ssl option {name} is off, so {what}"))
}

// ---------------------------------------------------------------------------
// Password hashing
// ---------------------------------------------------------------------------

/// `md5($password)`, `sha1($_POST['pass'])`, `hash('sha256', $pwd)`: a fast digest of a password,
/// which a GPU guesses billions of times a second once the hash leaks.
fn fast_password_digest(fn_name_lower: &str, args: &[Arg], prehash: bool) -> Option<String> {
    let (data, binary) = match fn_name_lower {
        "md5" | "sha1" => (arg(args, 0, "string"), arg(args, 1, "binary")),
        "crc32" => (arg(args, 0, "string"), None),
        "hash" => (arg(args, 1, "data"), arg(args, 2, "binary")),
        _ => (None, None),
    };
    let data = data?;
    // A raw binary digest is a pre-hash for `password_hash()`; nobody stores raw bytes.
    let raw = matches!(binary.map(|b| &b.kind), Some(ExprKind::Bool(true)));
    if prehash || raw || !names_password(data) {
        return None;
    }
    Some(format!(
        "{fn_name_lower}() is a fast hash, so a leaked password hash is guessed quickly; use password_hash()"
    ))
}

/// A variable, property or array key whose name says it holds a password; a password mixed into
/// other text is a signature or key derivation, not a stored hash.
fn names_password(expr: &Expr) -> bool {
    let name = match &expr.kind {
        ExprKind::Variable(v) => v.trim_start_matches('$'),
        ExprKind::ArrayAccess(aa) => match literal_str(aa.index.as_ref()) {
            Some(key) => key,
            None => return false,
        },
        ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa) => {
            match &pa.property.kind {
                ExprKind::Identifier(name) => name.as_ref(),
                _ => return false,
            }
        }
        ExprKind::Parenthesized(inner) | ExprKind::Cast(_, inner) => return names_password(inner),
        _ => return false,
    };
    identifier_words(name).iter().any(|w| {
        matches!(
            w.as_str(),
            "password" | "passwd" | "pass" | "pwd" | "passphrase"
        )
    })
}

fn password_hash_misuse(fn_name_lower: &str, args: &[Arg], v: &Values) -> Option<String> {
    if fn_name_lower != "password_hash" {
        return None;
    }
    let bcrypt = match arg(args, 1, "algo") {
        None => true,
        Some(algo) => match v.of(algo) {
            Some(Value::Null) => true,
            Some(Value::Str(s)) => &*s == "2y",
            _ => matches!(
                algo.kind.name_bare(),
                Some("PASSWORD_BCRYPT" | "PASSWORD_DEFAULT")
            ),
        },
    };
    if bcrypt && arg(args, 0, "password").is_some_and(is_raw_digest) {
        return Some(
            "a raw binary digest can contain NUL bytes, where bcrypt stops reading".to_string(),
        );
    }
    let options = v.of(arg(args, 2, "options")?)?;
    if options.entry("salt").is_some() {
        return Some("password_hash() ignores the 'salt' option since PHP 8.0".to_string());
    }
    match options.entry("cost").flatten().and_then(Value::as_int) {
        Some(cost) if bcrypt && cost < 10 => Some(format!("bcrypt cost {cost} is below 10")),
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

fn weak_cipher_use(fn_name_lower: &str, args: &[Arg], v: &Values) -> Option<String> {
    if !matches!(fn_name_lower, "openssl_encrypt" | "openssl_decrypt") {
        return None;
    }
    let string_at = |index: usize, name: &str| match v.of(arg(args, index, name)?) {
        Some(Value::Str(s)) => Some(s),
        _ => None,
    };
    let cipher = string_at(1, "cipher_algo").map(|c| c.to_ascii_lowercase());
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
    if string_at(2, "passphrase").is_some() {
        return Some("the key is hardcoded".to_string());
    }
    if string_at(4, "iv").is_some_and(|iv| !iv.is_empty()) {
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

fn insecure_session_setting(fn_name_lower: &str, args: &[Arg], v: &Values) -> Option<String> {
    match fn_name_lower {
        "ini_set" => {
            let name = v
                .of(arg(args, 0, "option")?)?
                .as_str()?
                .to_ascii_lowercase();
            let value = v.of(arg(args, 1, "value")?)?;
            weakened_session_setting(name.strip_prefix("session.")?, &value)
        }
        "session_start" => v
            .of(arg(args, 0, "options")?)?
            .entries()
            .iter()
            .find_map(|(k, val)| {
                let key = k.as_ref()?.as_str()?.to_ascii_lowercase();
                weakened_session_setting(&key, val.as_ref()?)
            }),
        _ => None,
    }
}

/// The reason a constant `value` for session setting `name` weakens the session.
fn weakened_session_setting(name: &str, value: &Value) -> Option<String> {
    let enabled = match value {
        Value::Bool(b) => *b,
        Value::Int(i) => *i != 0,
        Value::Str(s) => match s.to_ascii_lowercase().as_str() {
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

type OriginCheck = fn(&Expr, &FlowState) -> bool;

/// Every origin a variable assigned `value` takes on.
pub(crate) fn assigned_origins(
    value: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> OriginSet {
    let mut origins = OriginSet::EMPTY;
    if reads_framework_request(value, ctx, db, file) {
        origins = origins.with(ValueOrigin::RequestInput);
    }
    if let Some(origin) = crate::taint::assigned_value_origin(value) {
        origins = origins.with(origin);
    }
    if reads_cookie(value) {
        origins = origins.with(ValueOrigin::CookieInput);
    }
    if crate::taint::has_dynamic_where_clause(value) {
        origins = origins.with(ValueOrigin::NoSqlWhere);
    }
    if let ExprKind::FunctionCall(fc) = &value.kind {
        match verifier_success(&fc.name) {
            Some("1") => origins = origins.with(ValueOrigin::VerifyResult),
            Some(_) => origins = origins.with(ValueOrigin::VerifyResultBool),
            None => {}
        }
    }
    let checks: [(OriginCheck, ValueOrigin); 8] = [
        (is_mac_digest, ValueOrigin::MacDigest),
        (is_weak_random, ValueOrigin::WeakRandom),
        (is_decoded, ValueOrigin::Decoded),
        (is_request_origin, ValueOrigin::OriginHeader),
        (is_spoofable_ip, ValueOrigin::SpoofableIp),
        (is_host_derived, ValueOrigin::HostHeader),
        (is_client_file_name, ValueOrigin::ClientFileName),
        (is_request_input, ValueOrigin::RequestInput),
    ];
    for (holds, origin) in checks {
        if holds(value, ctx) {
            origins = origins.with(origin);
        }
    }
    origins
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

/// Request data, from a superglobal or read through a framework's request object.
pub(crate) fn is_request_value(
    ea: &crate::expr::ExpressionAnalyzer<'_>,
    expr: &Expr,
    ctx: &FlowState,
) -> bool {
    is_request_input(expr, ctx) || reads_framework_request(expr, ctx, ea.db, &ea.file)
}

/// Request data read through a framework, possibly cast or trimmed: a Symfony `Request` accessor
/// or parameter bag (`$request->query->getInt('n')`), a Laravel request or `Request` facade
/// (`$request->input('n')`), or a PSR-7 server request (`$request->getQueryParams()['n']`).
pub(crate) fn reads_framework_request(
    expr: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> bool {
    use crate::ast_ext::NameResolveExt as _;
    let recurse = |e: &Expr| reads_framework_request(e, ctx, db, file);
    let receiver_is = |receiver: &Expr, test: &dyn Fn(&str) -> bool| {
        crate::expr::assignment::resolve_chained_receiver_type(receiver, ctx, db, file).is_some_and(
            |ty| {
                ty.types.iter().any(|a| {
                    matches!(a, mir_types::Atomic::TNamedObject { fqcn, .. } if test(fqcn.as_ref()))
                })
            },
        )
    };
    match &expr.kind {
        ExprKind::Parenthesized(inner) | ExprKind::Cast(_, inner) => recurse(inner),
        ExprKind::NullCoalesce(nc) => recurse(&nc.left),
        ExprKind::ArrayAccess(aa) => recurse(&aa.array),
        ExprKind::FunctionCall(fc) => {
            named_one_of(&fc.name, &["trim", "intval", "strval", "floatval"])
                && fc.args.first().and_then(|a| a.value()).is_some_and(recurse)
        }
        ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => {
            let ExprKind::Identifier(method) = &mc.method.kind else {
                return false;
            };
            let method = crate::util::php_ident_lowercase(method.as_ref());
            if receiver_is(&mc.object, &|fqcn| {
                crate::taint::is_framework_request_source(db, fqcn, &method)
            }) {
                return true;
            }
            let ExprKind::PropertyAccess(pa) = &mc.object.kind else {
                return false;
            };
            let ExprKind::Identifier(bag) = &pa.property.kind else {
                return false;
            };
            matches!(
                method.as_str(),
                "get"
                    | "getint"
                    | "getalpha"
                    | "getalnum"
                    | "getdigits"
                    | "getboolean"
                    | "getstring"
                    | "all"
                    | "filter"
            ) && receiver_is(&pa.object, &|fqcn| {
                crate::taint::is_request_bag_property(db, fqcn, bag)
            })
        }
        ExprKind::StaticMethodCall(sc) => {
            let ExprKind::Identifier(method) = &sc.method.kind else {
                return false;
            };
            sc.class
                .kind
                .resolve_name_ref(db, file)
                .is_some_and(|class| {
                    crate::taint::is_laravel_request_facade_source(
                        &class,
                        &crate::util::php_ident_lowercase(method.as_ref()),
                    )
                })
        }
        _ => false,
    }
}

/// `$_GET['x']`, `$_POST['x']` or `$_REQUEST['x']`, possibly cast, trimmed or with a `??`
/// fallback, `filter_input(INPUT_GET, ...)`, or a variable holding request data. A session ID
/// from `$_COOKIE` is where PHP itself reads it, so that isn't counted.
fn reads_request_parameter(expr: &Expr, ctx: &FlowState) -> bool {
    match &expr.kind {
        ExprKind::ArrayAccess(aa) => {
            matches!(&aa.array.kind, ExprKind::Variable(v) if matches!(&**v, "_GET" | "_POST" | "_REQUEST"))
                || reads_request_parameter(&aa.array, ctx)
        }
        ExprKind::Variable(v) => {
            let origins = ctx.value_origins(v);
            origins.contains(ValueOrigin::RequestInput)
                && !origins.contains(ValueOrigin::CookieInput)
        }
        ExprKind::Parenthesized(inner) | ExprKind::Cast(_, inner) => {
            reads_request_parameter(inner, ctx)
        }
        ExprKind::NullCoalesce(nc) => reads_request_parameter(&nc.left, ctx),
        ExprKind::FunctionCall(fc) if named_one_of(&fc.name, &["filter_input"]) => matches!(
            arg(&fc.args, 0, "type").and_then(|t| t.kind.name_bare()),
            Some("INPUT_GET" | "INPUT_POST" | "INPUT_REQUEST")
        ),
        ExprKind::FunctionCall(fc) if named_one_of(&fc.name, &["trim", "strval", "urldecode"]) => {
            fc.args
                .first()
                .and_then(|a| a.value())
                .is_some_and(|v| reads_request_parameter(v, ctx))
        }
        _ => false,
    }
}

/// A `$_COOKIE` entry, possibly cast, trimmed or with a `??` fallback.
fn reads_cookie(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::ArrayAccess(aa) => {
            matches!(&aa.array.kind, ExprKind::Variable(v) if v.trim_start_matches('$') == "_COOKIE")
        }
        ExprKind::Parenthesized(inner) | ExprKind::Cast(_, inner) => reads_cookie(inner),
        ExprKind::NullCoalesce(nc) => reads_cookie(&nc.left),
        ExprKind::FunctionCall(fc) if named_one_of(&fc.name, &["trim", "strval"]) => fc
            .args
            .first()
            .and_then(|a| a.value())
            .is_some_and(reads_cookie),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Randomness
// ---------------------------------------------------------------------------

/// A seed an attacker can guess: a literal, or built from the clock or process ID.
fn is_predictable_seed(seed: &Expr) -> bool {
    use php_ast::ast::BinaryOp;
    match &seed.kind {
        ExprKind::Int(_) => true,
        ExprKind::Parenthesized(inner) | ExprKind::Cast(_, inner) => is_predictable_seed(inner),
        ExprKind::FunctionCall(fc) => named_one_of(
            &fc.name,
            &["time", "microtime", "hrtime", "getmypid", "date", "mktime"],
        ),
        ExprKind::Binary(b)
            if matches!(
                b.op,
                BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div
                    | BinaryOp::Mod
                    | BinaryOp::BitwiseXor
                    | BinaryOp::BitwiseOr
                    | BinaryOp::BitwiseAnd
            ) =>
        {
            is_predictable_seed(&b.left) && is_predictable_seed(&b.right)
        }
        _ => false,
    }
}

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

/// The lowercase words of a camelCase or snake_case identifier: `newPassword` → new, password.
fn identifier_words(name: &str) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    for (i, c) in name.char_indices() {
        if c == '_' || c == '-' {
            words.push(String::new());
        } else if words.is_empty() || (c.is_ascii_uppercase() && i > 0) {
            words.push(c.to_ascii_lowercase().to_string());
        } else if let Some(word) = words.last_mut() {
            word.push(c.to_ascii_lowercase());
        }
    }
    words.retain(|w| !w.is_empty());
    words
}

/// Whether a function name ends in a word for a secret: `generateToken`, `make_salt`, `newApiKey`.
fn names_secret(name: &str) -> bool {
    let words = identifier_words(name);
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

// ---------------------------------------------------------------------------
// File uploads
// ---------------------------------------------------------------------------

/// A path whose last part is the name the client gave an uploaded file:
/// `'uploads/' . basename($_FILES['f']['name'])`, so the client picks its extension.
pub(crate) fn is_client_file_name(expr: &Expr, ctx: &FlowState) -> bool {
    match &expr.kind {
        ExprKind::Variable(name) => ctx.has_value_origin(name, ValueOrigin::ClientFileName),
        ExprKind::ArrayAccess(aa) => {
            let mut base = expr;
            let mut names_file = false;
            while let ExprKind::ArrayAccess(aa) = &base.kind {
                names_file |= literal_str(aa.index.as_ref()) == Some("name");
                base = &aa.array;
            }
            names_file
                && !matches!(literal_str(aa.index.as_ref()), Some(k) if k != "name")
                && matches!(&base.kind, ExprKind::Variable(v) if v.trim_start_matches('$') == "_FILES")
        }
        ExprKind::Parenthesized(inner) => is_client_file_name(inner, ctx),
        ExprKind::Binary(b) if b.op == php_ast::ast::BinaryOp::Concat => {
            is_client_file_name(&b.right, ctx)
        }
        ExprKind::InterpolatedString(parts) => matches!(
            parts.last(),
            Some(php_ast::owned::StringPart::Expr(e)) if is_client_file_name(e, ctx)
        ),
        ExprKind::FunctionCall(fc) => {
            named_one_of(
                &fc.name,
                &[
                    "basename",
                    "strtolower",
                    "trim",
                    "str_replace",
                    "preg_replace",
                    "sanitize_file_name",
                    "stripslashes",
                    "urldecode",
                ],
            ) && fc
                .args
                .iter()
                .filter_map(|a| a.value())
                .next_back()
                .is_some_and(|v| is_client_file_name(v, ctx))
        }
        _ => false,
    }
}

/// A call that checks an uploaded file's extension or content: a type-detecting builtin, or a
/// string function applied to the client's file name.
fn checks_uploaded_file(fn_name_lower: &str, args: &[Arg], ctx: &FlowState) -> bool {
    const DETECTORS: &[&str] = &[
        "pathinfo",
        "getimagesize",
        "getimagesizefromstring",
        "exif_imagetype",
        "finfo_file",
        "finfo_buffer",
        "mime_content_type",
        "wp_check_filetype",
        "wp_check_filetype_and_ext",
    ];
    const NAME_CHECKS: &[&str] = &[
        "substr",
        "strrpos",
        "strrchr",
        "explode",
        "preg_match",
        "str_ends_with",
        "in_array",
    ];
    DETECTORS.contains(&fn_name_lower)
        || fn_name_lower.starts_with("imagecreatefrom")
        || (NAME_CHECKS.contains(&fn_name_lower)
            && args
                .iter()
                .filter_map(|a| a.value())
                .any(|v| mentions_uploaded_file(v, ctx)))
}

/// Whether `expr` refers to an upload: its client file name, or a `$_FILES` entry as a whole.
fn mentions_uploaded_file(expr: &Expr, ctx: &FlowState) -> bool {
    if is_client_file_name(expr, ctx) {
        return true;
    }
    match &expr.kind {
        ExprKind::Variable(v) => v.trim_start_matches('$') == "_FILES",
        ExprKind::ArrayAccess(aa) => {
            !matches!(
                literal_str(aa.index.as_ref()),
                Some("type" | "size" | "error")
            ) && matches!(&aa.array.kind, ExprKind::Variable(v) if v.trim_start_matches('$') == "_FILES")
        }
        ExprKind::Parenthesized(inner) | ExprKind::Cast(_, inner) => {
            mentions_uploaded_file(inner, ctx)
        }
        ExprKind::FunctionCall(fc) => call_values(&fc.args)
            .into_iter()
            .any(|v| mentions_uploaded_file(v, ctx)),
        _ => false,
    }
}

/// Count a condition calling one of the program's own helpers or methods on an upload, such
/// as `isAllowedUpload($_FILES['f'])`, as a check of it.
pub(crate) fn mark_upload_checked(cond: &Expr, ctx: &mut FlowState) {
    let checks = match &cond.kind {
        ExprKind::UnaryPrefix(u) => return mark_upload_checked(&u.operand, ctx),
        ExprKind::Parenthesized(inner) => return mark_upload_checked(inner, ctx),
        ExprKind::FunctionCall(fc) => {
            fc.name.kind.name_bare().is_some_and(|n| {
                !crate::stubs::is_builtin_function(&n.trim_start_matches('\\').to_ascii_lowercase())
            }) && call_values(&fc.args)
                .into_iter()
                .any(|v| mentions_uploaded_file(v, ctx))
        }
        ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => call_values(&mc.args)
            .into_iter()
            .any(|v| mentions_uploaded_file(v, ctx)),
        ExprKind::StaticMethodCall(sc) => call_values(&sc.args)
            .into_iter()
            .any(|v| mentions_uploaded_file(v, ctx)),
        _ => false,
    };
    if checks {
        ctx.upload_checked = true;
    }
}

// ---------------------------------------------------------------------------
// Host header
// ---------------------------------------------------------------------------

/// A value built from the `Host` header the client sent: `$_SERVER['HTTP_HOST']`,
/// `$request->getSchemeAndHttpHost()`, or a string or variable made from one.
pub(crate) fn is_host_derived(expr: &Expr, ctx: &FlowState) -> bool {
    match &expr.kind {
        ExprKind::Variable(name) => ctx.has_value_origin(name, ValueOrigin::HostHeader),
        ExprKind::ArrayAccess(aa) => {
            matches!(&aa.array.kind, ExprKind::Variable(v) if v.trim_start_matches('$') == "_SERVER")
                && matches!(
                    literal_str(aa.index.as_ref()),
                    Some("HTTP_HOST" | "HTTP_X_FORWARDED_HOST")
                )
        }
        ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => matches!(
            &mc.method.kind,
            ExprKind::Identifier(m)
                if m.eq_ignore_ascii_case("getHttpHost") || m.eq_ignore_ascii_case("getSchemeAndHttpHost")
        ),
        ExprKind::NullCoalesce(nc) => is_host_derived(&nc.left, ctx),
        ExprKind::FunctionCall(fc) => {
            named_one_of(
                &fc.name,
                &["sprintf", "str_replace", "strtr", "rtrim", "trim"],
            ) && fc
                .args
                .iter()
                .filter_map(|a| a.value())
                .any(|v| is_host_derived(v, ctx))
        }
        _ => any_string_part(expr, &mut |part| is_host_derived(part, ctx)),
    }
}

/// Forget the origins a condition validates on the branch where it evaluates to `holds`:
/// equality with a value, an allowlist lookup or an anchored pattern clears request, host and
/// Origin header data; an upper bound clears request data used as an amount.
pub(crate) fn forget_checked_origins(cond: &Expr, holds: bool, ctx: &mut FlowState) {
    let Some((operands, bound_only)) = validated_operands(cond, holds) else {
        return;
    };
    for operand in operands {
        let operand = match &operand.kind {
            ExprKind::Cast(_, inner) | ExprKind::Parenthesized(inner) => inner,
            ExprKind::FunctionCall(fc) if !bound_only && named_one_of(&fc.name, &["trim"]) => {
                match fc.args.first().and_then(|a| a.value()) {
                    Some(inner) => inner,
                    None => continue,
                }
            }
            _ => operand,
        };
        match &operand.kind {
            ExprKind::Variable(var) => {
                ctx.forget_value_origin(var, ValueOrigin::RequestInput);
                if !bound_only {
                    ctx.forget_value_origin(var, ValueOrigin::HostHeader);
                    ctx.forget_value_origin(var, ValueOrigin::OriginHeader);
                }
            }
            _ if !bound_only && is_request_origin(operand, ctx) => ctx.origin_checked = true,
            _ => {}
        }
    }
}

/// The operands `cond` validates when it evaluates to `holds`, and whether it only bounds them
/// from above rather than pinning them to known values.
fn validated_operands(cond: &Expr, holds: bool) -> Option<(Vec<&Expr>, bool)> {
    use php_ast::ast::BinaryOp;
    let is_unset = |e: &Expr| match &e.kind {
        ExprKind::String(s) => s.is_empty(),
        ExprKind::Null | ExprKind::Bool(_) => true,
        _ => false,
    };
    match &cond.kind {
        ExprKind::Binary(b) => match b.op {
            BinaryOp::Identical | BinaryOp::Equal | BinaryOp::NotIdentical | BinaryOp::NotEqual => {
                let equal = matches!(b.op, BinaryOp::Identical | BinaryOp::Equal);
                (equal == holds && !is_unset(&b.left) && !is_unset(&b.right))
                    .then(|| (vec![&b.left, &b.right], false))
            }
            // `$n < 100` holding, or `$n > 100` failing, caps `$n`; the mirrored forms cap the
            // right operand.
            BinaryOp::Less | BinaryOp::LessOrEqual => {
                Some((vec![if holds { &b.left } else { &b.right }], true))
            }
            BinaryOp::Greater | BinaryOp::GreaterOrEqual => {
                Some((vec![if holds { &b.right } else { &b.left }], true))
            }
            _ => None,
        },
        ExprKind::FunctionCall(fc) if holds => {
            let index = if named_one_of(&fc.name, &["in_array", "array_key_exists", "array_search"])
            {
                0
            } else if named_one_of(&fc.name, &["preg_match"])
                && literal_str(arg(&fc.args, 0, "pattern")).is_none_or(is_anchored_pattern)
            {
                1
            } else if named_one_of(&fc.name, &["str_ends_with"])
                && literal_str(arg(&fc.args, 1, "needle")).is_some_and(|n| n.starts_with('.'))
            {
                0
            } else if fc.name.kind.name_bare().is_some_and(|n| {
                !crate::stubs::is_builtin_function(&n.trim_start_matches('\\').to_ascii_lowercase())
            }) {
                // A helper of the program's own, such as `isAllowedOrigin($origin)`, is taken
                // to check every value it's given.
                return Some((call_values(&fc.args), false));
            } else {
                return None;
            };
            Some((
                fc.args
                    .get(index)
                    .and_then(|a| a.value())
                    .into_iter()
                    .collect(),
                false,
            ))
        }
        ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) if holds => {
            Some((call_values(&mc.args), false))
        }
        ExprKind::StaticMethodCall(sc) if holds => Some((call_values(&sc.args), false)),
        ExprKind::Isset(items) if holds => Some((
            items
                .iter()
                .filter_map(|item| match &item.kind {
                    ExprKind::ArrayAccess(aa) => aa.index.as_ref(),
                    _ => None,
                })
                .collect(),
            false,
        )),
        _ => None,
    }
}

fn call_values(args: &[Arg]) -> Vec<&Expr> {
    args.iter().filter_map(|a| a.value()).collect()
}

/// A regex that must match the whole subject: `^...$` or `\A...\z`.
fn is_anchored_pattern(pattern: &str) -> bool {
    let Some(delimiter) = pattern.chars().next() else {
        return false;
    };
    let closing = match delimiter {
        '(' => ')',
        '{' => '}',
        '[' => ']',
        '<' => '>',
        other => other,
    };
    let Some(end) = pattern.rfind(closing).filter(|&end| end > 0) else {
        return false;
    };
    let body = &pattern[delimiter.len_utf8()..end];
    (body.starts_with('^') || body.starts_with("\\A"))
        && (body.ends_with('$') || body.ends_with("\\z") || body.ends_with("\\Z"))
}

/// Mark a variable a condition proves non-empty on the branch where it evaluates to `holds`:
/// a truthy or `empty()` check, a comparison with `''`, or a length of at least one.
pub(crate) fn mark_non_empty(cond: &Expr, holds: bool, ctx: &mut FlowState) {
    use php_ast::ast::BinaryOp;
    let var = match &cond.kind {
        ExprKind::Variable(var) if holds => Some(var),
        ExprKind::Empty(inner) if !holds => match &inner.kind {
            ExprKind::Variable(var) => Some(var),
            _ => None,
        },
        ExprKind::Binary(b) => {
            let empty_string = |e: &Expr| matches!(&e.kind, ExprKind::String(s) if s.is_empty());
            let equal = matches!(b.op, BinaryOp::Identical | BinaryOp::Equal);
            let unequal = matches!(b.op, BinaryOp::NotIdentical | BinaryOp::NotEqual);
            match (&b.left.kind, &b.right.kind) {
                (ExprKind::Variable(var), _) | (_, ExprKind::Variable(var))
                    if (equal || unequal)
                        && equal != holds
                        && (empty_string(&b.left) || empty_string(&b.right)) =>
                {
                    Some(var)
                }
                (ExprKind::FunctionCall(fc), ExprKind::Int(bound))
                    if named_one_of(&fc.name, &["strlen", "mb_strlen"])
                        && length_at_least_one(b.op, *bound, holds) =>
                {
                    match fc.args.first().and_then(|a| a.value()).map(|v| &v.kind) {
                        Some(ExprKind::Variable(var)) => Some(var),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => None,
    };
    if let Some(var) = var {
        ctx.mark_value_origin(var, ValueOrigin::NonEmpty);
    }
}

/// Whether `strlen($v) <op> bound` evaluating to `holds` means `$v` has at least one byte.
fn length_at_least_one(op: php_ast::ast::BinaryOp, bound: i64, holds: bool) -> bool {
    use php_ast::ast::BinaryOp;
    match (op, holds) {
        (BinaryOp::Greater, true) | (BinaryOp::LessOrEqual, false) => bound >= 0,
        (BinaryOp::GreaterOrEqual, true)
        | (BinaryOp::Less, false)
        | (BinaryOp::Identical | BinaryOp::Equal, true)
        | (BinaryOp::NotIdentical | BinaryOp::NotEqual, false) => bound >= 1,
        (BinaryOp::NotIdentical | BinaryOp::NotEqual, true)
        | (BinaryOp::Identical | BinaryOp::Equal, false) => bound == 0,
        _ => false,
    }
}

/// Whether `value` holds request data: by its taint when taint analysis runs, else by origin.
fn holds_request_data(
    ea: &crate::expr::ExpressionAnalyzer<'_>,
    value: &Expr,
    ctx: &FlowState,
) -> bool {
    if ea.db.taint_analysis() {
        !crate::taint::expr_taint(value, ctx, ea.db, &ea.file)
            .concrete()
            .is_empty()
    } else {
        is_request_input(value, ctx)
    }
}

// ---------------------------------------------------------------------------
// Resource exhaustion
// ---------------------------------------------------------------------------

/// The size, count or duration argument of a builtin whose cost grows with it.
fn resource_amount_arg<'e>(fn_name_lower: &str, args: &'e [Arg]) -> Option<&'e Expr> {
    match fn_name_lower {
        "str_repeat" => arg(args, 1, "times"),
        "str_pad" => arg(args, 1, "length"),
        "range" => arg(args, 1, "end"),
        "array_fill" => arg(args, 1, "count"),
        "array_pad" => arg(args, 1, "length"),
        "sleep" | "set_time_limit" => arg(args, 0, "seconds"),
        "usleep" => arg(args, 0, "microseconds"),
        "time_nanosleep" => arg(args, 0, "seconds"),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Conditions
// ---------------------------------------------------------------------------

/// Report security decisions an `if`, `elseif` or ternary condition gets wrong.
pub(crate) fn check_condition(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    cond: &Expr,
    ctx: &FlowState,
) {
    for part in condition_parts(cond) {
        if let Some(Verifier { label, success }) = error_value_passes(part, ctx) {
            ea.emit(
                mir_issues::IssueKind::InsecureCheckResult {
                    reason: format!(
                        "{label} is -1 on error, which passes this check; compare with === {success}"
                    ),
                },
                mir_issues::Severity::Warning,
                part.span,
            );
        }
        if let Some(ip) = spoofable_ip_check(part, ctx) {
            ea.emit(
                mir_issues::IssueKind::SpoofableClientIp,
                mir_issues::Severity::Warning,
                ip.span,
            );
        }
    }
}

/// Report a `==`/`===` comparison that leaks through its timing a `hash_hmac()` digest compared
/// with a non-literal value, or a value named for a secret compared with request data.
pub(crate) fn check_timing_unsafe_comparison(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    b: &php_ast::owned::BinaryExpr,
    ctx: &FlowState,
    span: php_ast::Span,
) {
    use php_ast::ast::BinaryOp;
    let op = match b.op {
        BinaryOp::Equal => "==",
        BinaryOp::Identical => "===",
        BinaryOp::NotEqual => "!=",
        BinaryOp::NotIdentical => "!==",
        _ => return,
    };
    if let Some(what) = timing_unsafe_operand(ea, b, ctx) {
        ea.emit(
            mir_issues::IssueKind::TimingUnsafeComparison {
                what: format!("{what} is compared with {op}"),
            },
            mir_issues::Severity::Warning,
            span,
        );
    }
}

fn timing_unsafe_operand(
    ea: &crate::expr::ExpressionAnalyzer<'_>,
    b: &php_ast::owned::BinaryExpr,
    ctx: &FlowState,
) -> Option<String> {
    let is_literal = |e: &Expr| {
        matches!(
            e.kind,
            ExprKind::String(_) | ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::Null
        )
    };
    for (one, other) in [(&b.left, &b.right), (&b.right, &b.left)] {
        if is_mac_digest(one, ctx) && !is_literal(other) {
            return Some("a hash_hmac() digest".to_string());
        }
        if is_request_value(ea, other, ctx) && !is_request_value(ea, one, ctx) {
            if let Some(name) = stored_secret_name(one, ctx) {
                return Some(format!("the stored secret '{name}'"));
            }
        }
    }
    None
}

/// A `hash_hmac()` call, possibly hex- or base64-encoded, or a variable holding one.
fn is_mac_digest(expr: &Expr, ctx: &FlowState) -> bool {
    match &expr.kind {
        ExprKind::Variable(v) => ctx.has_value_origin(v, ValueOrigin::MacDigest),
        ExprKind::Parenthesized(inner) => is_mac_digest(inner, ctx),
        ExprKind::FunctionCall(fc) => {
            named_one_of(&fc.name, &["hash_hmac"])
                || named_one_of(
                    &fc.name,
                    &[
                        "bin2hex",
                        "base64_encode",
                        "strtolower",
                        "strtoupper",
                        "rtrim",
                        "strtr",
                    ],
                ) && fc
                    .args
                    .first()
                    .and_then(|a| a.value())
                    .is_some_and(|v| is_mac_digest(v, ctx))
        }
        _ => false,
    }
}

/// The variable, property or literal array key naming a stored secret: `$_SESSION['csrf_token']`,
/// `$this->apiKey`, `$secret`. A parameter usually carries the submitted value, not the stored one.
fn stored_secret_name(expr: &Expr, ctx: &FlowState) -> Option<String> {
    let name = match &expr.kind {
        ExprKind::Variable(v) => {
            let v = v.trim_start_matches('$');
            if ctx.param_names.contains(&mir_types::Name::from(v)) {
                return None;
            }
            v.to_string()
        }
        ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa) => {
            match &pa.property.kind {
                ExprKind::Identifier(name) => name.to_string(),
                _ => return None,
            }
        }
        ExprKind::ArrayAccess(aa) => literal_str(aa.index.as_ref())?.to_string(),
        ExprKind::Parenthesized(inner) => return stored_secret_name(inner, ctx),
        _ => return None,
    };
    names_secret(&name).then_some(name)
}

/// A verification result and the value that alone means success.
struct Verifier {
    label: String,
    success: &'static str,
}

/// The verification result whose `-1` error value a condition lets through, as it treats any
/// truthy result as success: `if (openssl_verify(...))`, `openssl_verify(...) != 0`.
fn error_value_passes(check: &Expr, ctx: &FlowState) -> Option<Verifier> {
    if let Some(found) = verifier(check, ctx) {
        return Some(found);
    }
    let ExprKind::Binary(b) = &check.kind else {
        return None;
    };
    let (call, literal) = match (verifier(&b.left, ctx), verifier(&b.right, ctx)) {
        (Some(found), _) => (found, &b.right.kind),
        (_, Some(found)) => (found, &b.left.kind),
        _ => return None,
    };
    use php_ast::ast::BinaryOp;
    let passes = matches!(
        (b.op, literal),
        (BinaryOp::Equal, ExprKind::Bool(true))
            | (
                BinaryOp::NotEqual | BinaryOp::NotIdentical,
                ExprKind::Bool(false) | ExprKind::Int(0)
            )
    );
    passes.then_some(call)
}

fn verifier(e: &Expr, ctx: &FlowState) -> Option<Verifier> {
    match &e.kind {
        ExprKind::FunctionCall(fc) => Some(Verifier {
            success: verifier_success(&fc.name)?,
            label: format!("{}()", fc.name.kind.name_bare()?.trim_start_matches('\\')),
        }),
        ExprKind::Variable(v) => {
            let origins = ctx.value_origins(v);
            let success = if origins.contains(ValueOrigin::VerifyResult) {
                "1"
            } else if origins.contains(ValueOrigin::VerifyResultBool) {
                "true"
            } else {
                return None;
            };
            Some(Verifier {
                label: format!("${}, a verification result,", v.trim_start_matches('$')),
                success,
            })
        }
        _ => None,
    }
}

/// The result of a verifying function that means success, for a function that returns -1 on
/// error.
fn verifier_success(name: &Expr) -> Option<&'static str> {
    if named_one_of(name, &["openssl_verify", "openssl_x509_verify"]) {
        Some("1")
    } else if named_one_of(name, &["openssl_pkcs7_verify", "ldap_compare"]) {
        Some("true")
    } else {
        None
    }
}

/// The operands `&&`, `||`, `!` and parentheses combine into a condition.
fn condition_parts(cond: &Expr) -> Vec<&Expr> {
    match &cond.kind {
        ExprKind::Parenthesized(inner) => condition_parts(inner),
        ExprKind::UnaryPrefix(u) if u.op == php_ast::ast::UnaryPrefixOp::BooleanNot => {
            condition_parts(&u.operand)
        }
        ExprKind::Binary(b)
            if matches!(
                b.op,
                php_ast::ast::BinaryOp::BooleanAnd
                    | php_ast::ast::BinaryOp::BooleanOr
                    | php_ast::ast::BinaryOp::LogicalAnd
                    | php_ast::ast::BinaryOp::LogicalOr
            ) =>
        {
            let mut parts = condition_parts(&b.left);
            parts.extend(condition_parts(&b.right));
            parts
        }
        _ => vec![cond],
    }
}

/// A client IP from a forgeable header that a comparison, list lookup or pattern match decides
/// on: `in_array($_SERVER['HTTP_X_FORWARDED_FOR'], $allowed)`.
fn spoofable_ip_check<'e>(check: &'e Expr, ctx: &FlowState) -> Option<&'e Expr> {
    let is_empty_literal = |e: &Expr| match &e.kind {
        ExprKind::String(s) => s.is_empty(),
        ExprKind::Null | ExprKind::Bool(_) => true,
        _ => false,
    };
    match &check.kind {
        ExprKind::Binary(b)
            if matches!(
                b.op,
                php_ast::ast::BinaryOp::Identical
                    | php_ast::ast::BinaryOp::NotIdentical
                    | php_ast::ast::BinaryOp::Equal
                    | php_ast::ast::BinaryOp::NotEqual
            ) =>
        {
            [(&b.left, &b.right), (&b.right, &b.left)]
                .into_iter()
                .find(|(ip, other)| is_spoofable_ip(ip, ctx) && !is_empty_literal(other))
                .map(|(ip, _)| ip)
        }
        ExprKind::FunctionCall(fc) => {
            let index = if named_one_of(
                &fc.name,
                &[
                    "in_array",
                    "array_search",
                    "array_key_exists",
                    "str_starts_with",
                    "strpos",
                    "strncmp",
                ],
            ) {
                0
            } else if named_one_of(&fc.name, &["preg_match"]) {
                1
            } else {
                return None;
            };
            fc.args
                .get(index)
                .and_then(|a| a.value())
                .filter(|ip| is_spoofable_ip(ip, ctx))
        }
        _ => None,
    }
}

/// A client IP taken from `X-Forwarded-For`, `Client-IP` or `X-Real-IP`, which the client sets.
pub(crate) fn is_spoofable_ip(expr: &Expr, ctx: &FlowState) -> bool {
    let names_header = |key: Option<&str>| {
        matches!(
            key,
            Some("HTTP_X_FORWARDED_FOR" | "HTTP_CLIENT_IP" | "HTTP_X_REAL_IP")
        )
    };
    match &expr.kind {
        ExprKind::Variable(name) => ctx.has_value_origin(name, ValueOrigin::SpoofableIp),
        ExprKind::ArrayAccess(aa) => match &aa.array.kind {
            ExprKind::Variable(v) if v.trim_start_matches('$') == "_SERVER" => {
                names_header(literal_str(aa.index.as_ref()))
            }
            _ => is_spoofable_ip(&aa.array, ctx),
        },
        ExprKind::Parenthesized(inner) | ExprKind::Cast(_, inner) => is_spoofable_ip(inner, ctx),
        ExprKind::NullCoalesce(nc) => is_spoofable_ip(&nc.left, ctx),
        ExprKind::Ternary(t) => {
            t.then_expr
                .as_ref()
                .is_some_and(|e| is_spoofable_ip(e, ctx))
                || is_spoofable_ip(&t.else_expr, ctx)
        }
        ExprKind::FunctionCall(fc) if named_one_of(&fc.name, &["getenv"]) => {
            names_header(literal_str(arg(&fc.args, 0, "name")))
        }
        ExprKind::FunctionCall(fc) => {
            named_one_of(&fc.name, &["trim", "explode", "current", "reset", "strtok"])
                && fc
                    .args
                    .iter()
                    .filter_map(|a| a.value())
                    .any(|v| is_spoofable_ip(v, ctx))
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Redirects
// ---------------------------------------------------------------------------

/// `header('Location: ...')`, whose literal text starts the header.
pub(crate) fn is_location_header(args: &[Arg]) -> bool {
    let Some(mut first) = arg(args, 0, "header") else {
        return false;
    };
    while let ExprKind::Binary(b) = &first.kind {
        if b.op != php_ast::ast::BinaryOp::Concat {
            break;
        }
        first = &b.left;
    }
    let text = match &first.kind {
        ExprKind::String(s) => s.as_str(),
        ExprKind::InterpolatedString(parts) => match parts.first() {
            Some(php_ast::owned::StringPart::Literal(s)) => s.as_str(),
            _ => return false,
        },
        _ => return false,
    };
    text.trim_start()
        .to_ascii_lowercase()
        .starts_with("location:")
}

/// Report `if (preg_match('/evil/u', $input)) exit;` on tainted input: `preg_match()` returns
/// false when matching fails, so input that breaks the pattern skips the rejection.
pub(crate) fn check_deny_list_match(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    cond: &Expr,
    ctx: &FlowState,
) {
    for part in either_parts(cond) {
        let Some(fc) = matched_call(part) else {
            continue;
        };
        let (Some(pattern), Some(subject)) = (
            literal_str(arg(&fc.args, 0, "pattern")),
            arg(&fc.args, 1, "subject"),
        ) else {
            continue;
        };
        let Some(failure) = pattern_failure(pattern) else {
            continue;
        };
        if !holds_request_data(ea, subject, ctx) {
            continue;
        }
        ea.emit(
            mir_issues::IssueKind::InsecureCheckResult {
                reason: format!(
                    "preg_match() returns false {failure}, which skips this rejection; compare with !== 0"
                ),
            },
            mir_issues::Severity::Warning,
            part.span,
        );
    }
}

/// The operands `||` joins: any of them holding takes the branch.
fn either_parts(cond: &Expr) -> Vec<&Expr> {
    match &cond.kind {
        ExprKind::Parenthesized(inner) => either_parts(inner),
        ExprKind::Binary(b)
            if matches!(
                b.op,
                php_ast::ast::BinaryOp::BooleanOr | php_ast::ast::BinaryOp::LogicalOr
            ) =>
        {
            let mut parts = either_parts(&b.left);
            parts.extend(either_parts(&b.right));
            parts
        }
        _ => vec![cond],
    }
}

/// A `preg_match()` call in a check that a `false` result fails: used as a condition, or
/// compared with `== 1`, `=== 1`, `> 0`, `>= 1`, `== true` or `!= 0`.
fn matched_call(check: &Expr) -> Option<&php_ast::owned::FunctionCallExpr> {
    if let Some(fc) = preg_match_call(check) {
        return Some(fc);
    }
    let ExprKind::Binary(b) = &check.kind else {
        return None;
    };
    use php_ast::ast::BinaryOp;
    let fc = preg_match_call(&b.left)?;
    let fails_on_false = matches!(
        (b.op, &b.right.kind),
        (
            BinaryOp::Equal | BinaryOp::Identical | BinaryOp::GreaterOrEqual,
            ExprKind::Int(1)
        ) | (BinaryOp::Greater | BinaryOp::NotEqual, ExprKind::Int(0))
            | (BinaryOp::Equal, ExprKind::Bool(true))
    );
    fails_on_false.then_some(fc)
}

fn preg_match_call(e: &Expr) -> Option<&php_ast::owned::FunctionCallExpr> {
    match &e.kind {
        ExprKind::FunctionCall(fc) if named_one_of(&fc.name, &["preg_match"]) => Some(fc),
        _ => None,
    }
}

/// How a literal pattern can make `preg_match()` fail on attacker input.
fn pattern_failure(pattern: &str) -> Option<&'static str> {
    let delimiter = pattern.chars().next()?;
    let closer = match delimiter {
        '(' => ')',
        '{' => '}',
        '[' => ']',
        '<' => '>',
        c => c,
    };
    let end = pattern.rfind(closer).filter(|&end| end > 0)?;
    let (body, flags) = (&pattern[1..end], &pattern[end + 1..]);
    if flags.contains('u') {
        return Some("on invalid UTF-8");
    }
    let bytes = body.as_bytes();
    let quantified_group = bytes.windows(2).enumerate().any(|(i, w)| {
        w[0] == b')'
            && (i == 0 || bytes[i - 1] != b'\\')
            && (w[1] == b'*'
                || w[1] == b'+'
                || (w[1] == b'{'
                    && body[i + 1..]
                        .split('}')
                        .next()
                        .is_some_and(|q| q.contains(','))))
    });
    quantified_group.then_some("when a long input hits the backtracking limit")
}

// ---------------------------------------------------------------------------
// LDAP
// ---------------------------------------------------------------------------

/// Report `ldap_bind()` with a request password not checked to be non-empty: LDAP servers treat
/// a bind with a DN and an empty password as an anonymous bind, which succeeds.
pub(crate) fn check_ldap_bind(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    args: &[Arg],
    arg_types: &[mir_types::Type],
    ctx: &FlowState,
    span: php_ast::Span,
) {
    let Some(index) = args
        .iter()
        .position(|a| a.name.as_ref().is_some_and(|n| n.last_part() == "password"))
        .or_else(|| {
            (args.len() > 2 && args[..3].iter().all(|a| a.name.is_none() && !a.unpack()))
                .then_some(2)
        })
    else {
        return;
    };
    let (Some(password), Some(ty)) = (args[index].value(), arg_types.get(index)) else {
        return;
    };
    let checked_non_empty = matches!(&password.kind, ExprKind::Variable(v) if ctx.has_value_origin(v, ValueOrigin::NonEmpty));
    if ty.can_be_falsy()
        && !checked_non_empty
        && is_request_input(password, ctx)
        && holds_request_data(ea, password, ctx)
    {
        ea.emit(
            mir_issues::IssueKind::LdapAnonymousBind,
            mir_issues::Severity::Warning,
            span,
        );
    }
}

// ---------------------------------------------------------------------------
// File permissions
// ---------------------------------------------------------------------------

/// `chmod()` to a literal mode, or `umask()` to a literal mask, that leaves files writable by
/// every user. `mkdir()` isn't checked: the umask applies to its mode, which defaults to 0777.
fn world_writable_mode(fn_name_lower: &str, args: &[Arg], v: &Values) -> Option<String> {
    const OTHERS_WRITE: i64 = 0o002;
    let literal = |e: Option<&Expr>| v.of(e?)?.as_int();
    match fn_name_lower {
        "chmod" => literal(arg(args, 1, "permissions"))
            .filter(|mode| mode & OTHERS_WRITE != 0)
            .map(|mode| format!("chmod() mode {mode:04o} lets every user on the system write the file")),
        "umask" => literal(arg(args, 0, "mask"))
            .filter(|mask| mask & OTHERS_WRITE == 0)
            .map(|mask| {
                format!("umask({mask:04o}) lets every user on the system write the files and directories created after it")
            }),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Hard-coded credentials
// ---------------------------------------------------------------------------

/// Report misuse visible in the arguments of `new Class(...)`.
pub(crate) fn check_constructor_call(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    fqcn: &str,
    args: &[Arg],
    ctx: &FlowState,
    span: php_ast::Span,
) {
    let class = fqcn.trim_start_matches('\\');
    let file = ea.file.clone();
    let values = Values {
        ctx,
        db: ea.db,
        file: &file,
    };
    let takes_password = class.eq_ignore_ascii_case("PDO") || class.eq_ignore_ascii_case("mysqli");
    if takes_password && is_password_literal(arg(args, 2, "password"), &values) {
        ea.emit(
            mir_issues::IssueKind::HardcodedCredential {
                what: format!("the password passed to new {class}()"),
            },
            mir_issues::Severity::Warning,
            span,
        );
    }
}

/// A password written in the source: a non-empty constant string that isn't a placeholder such as
/// `%env(DB_PASS)%`, `${DB_PASS}`, `{{ password }}`, `<password>` or `****`.
fn is_password_literal(expr: Option<&Expr>, v: &Values) -> bool {
    let Some(Value::Str(s)) = expr.and_then(|e| v.of(e)) else {
        return false;
    };
    !s.is_empty() && !is_placeholder(&s)
}

fn is_placeholder(s: &str) -> bool {
    let wrapped = |open: &str, close: &str| s.starts_with(open) && s.ends_with(close);
    let mut chars = s.chars();
    let first = chars.next();
    wrapped("%", "%")
        || wrapped("<", ">")
        || s.contains("${")
        || s.contains("{{")
        || (s.len() >= 3 && chars.all(|c| Some(c) == first))
}

/// A literal password at a builtin's password parameter, or in a `pg_connect()` string.
fn hardcoded_password(fn_name_lower: &str, args: &[Arg], v: &Values) -> Option<String> {
    let password = match fn_name_lower {
        "mysqli_connect" | "ldap_bind" => arg(args, 2, "password"),
        "mysqli_real_connect" => arg(args, 3, "password"),
        "oci_connect" | "oci_pconnect" | "oci_new_connect" => arg(args, 1, "password"),
        "pg_connect" | "pg_pconnect" => {
            let conn = v.of(arg(args, 0, "connection_string")?)?;
            let conn = conn.as_str()?;
            let value = conn
                .split_whitespace()
                .find_map(|p| p.strip_prefix("password="))?;
            return (!value.is_empty() && value != "''" && !is_placeholder(value))
                .then(|| format!("the password in the {fn_name_lower}() connection string"));
        }
        _ => return None,
    };
    is_password_literal(password, v).then(|| format!("the password passed to {fn_name_lower}()"))
}

/// Report a string literal shaped like a live secret: an AWS access key ID, a private key or a
/// Stripe live secret key.
pub(crate) fn check_secret_literal(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    text: &str,
    span: php_ast::Span,
) {
    if text.len() < 20 {
        return;
    }
    if let Some(what) = secret_shape(text) {
        ea.emit(
            mir_issues::IssueKind::HardcodedCredential {
                what: what.to_string(),
            },
            mir_issues::Severity::Warning,
            span,
        );
    }
}

fn secret_shape(text: &str) -> Option<&'static str> {
    let alnum = |s: &str| s.bytes().all(|b| b.is_ascii_alphanumeric());
    let upper_alnum = |s: &str| {
        s.bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
    };
    if text.len() == 20
        && (text.starts_with("AKIA") || text.starts_with("ASIA"))
        && upper_alnum(text)
        && !text.contains("EXAMPLE")
    {
        return Some("an AWS access key ID");
    }
    for prefix in ["sk_live_", "rk_live_"] {
        if let Some(rest) = text.strip_prefix(prefix) {
            if rest.len() >= 24 && alnum(rest) {
                return Some("a Stripe live secret key");
            }
        }
    }
    if let Some(start) = text.find("-----BEGIN ") {
        let header_end = text[start..].find("PRIVATE KEY-----")? + start + "PRIVATE KEY-----".len();
        let body: usize = text[header_end..]
            .bytes()
            .take_while(|b| *b != b'-')
            .filter(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'='))
            .count();
        if body >= 64 {
            return Some("a private key");
        }
    }
    None
}

// ---------------------------------------------------------------------------
// CORS
// ---------------------------------------------------------------------------

/// The request's `Origin` header: `$_SERVER['HTTP_ORIGIN']` or a variable holding it.
pub(crate) fn is_request_origin(expr: &Expr, ctx: &FlowState) -> bool {
    match &expr.kind {
        ExprKind::Variable(name) => ctx.has_value_origin(name, ValueOrigin::OriginHeader),
        ExprKind::ArrayAccess(aa) => {
            matches!(&aa.array.kind, ExprKind::Variable(v) if v.trim_start_matches('$') == "_SERVER")
                && literal_str(aa.index.as_ref()) == Some("HTTP_ORIGIN")
        }
        ExprKind::Parenthesized(inner) | ExprKind::Cast(_, inner) => is_request_origin(inner, ctx),
        ExprKind::NullCoalesce(nc) => is_request_origin(&nc.left, ctx),
        _ => false,
    }
}

/// `Access-Control-Allow-Origin: ` followed by the request's `Origin` header.
fn reflects_origin(header: &Expr, ctx: &FlowState) -> bool {
    let mut parts = Vec::new();
    command_parts(header, &mut parts);
    let mut text = String::new();
    for part in parts {
        match part {
            CommandPart::Text(t) => text.push_str(t),
            CommandPart::Value(v) => {
                return text
                    .trim()
                    .eq_ignore_ascii_case("access-control-allow-origin:")
                    && is_request_origin(v, ctx);
            }
        }
    }
    false
}

/// Report a reflected `Access-Control-Allow-Origin` together with
/// `Access-Control-Allow-Credentials: true`, at the origin header, whichever is sent first.
fn check_cors_header(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    args: &[Arg],
    ctx: &mut FlowState,
    span: php_ast::Span,
) {
    let Some(header) = arg(args, 0, "header") else {
        return;
    };
    let emit = |ea: &mut crate::expr::ExpressionAnalyzer<'_>, at| {
        ea.emit(
            mir_issues::IssueKind::InsecureCors,
            mir_issues::Severity::Warning,
            at,
        );
    };
    if allows_credentials(header) {
        if let Some(reflection) = ctx.cors_reflection.take() {
            emit(ea, reflection);
        }
        ctx.cors_credentials = true;
    } else if !ctx.origin_checked && reflects_origin(header, ctx) {
        if ctx.cors_credentials {
            emit(ea, span);
        } else {
            ctx.cors_reflection = Some(span);
        }
    }
}

/// `Access-Control-Allow-Credentials: true`, in any case and spacing.
fn allows_credentials(header: &Expr) -> bool {
    let mut parts = Vec::new();
    command_parts(header, &mut parts);
    let mut text = String::new();
    for part in parts {
        match part {
            CommandPart::Text(t) => text.push_str(t),
            CommandPart::Value(_) => return false,
        }
    }
    let compact: String = text
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    compact == "access-control-allow-credentials:true"
}

// ---------------------------------------------------------------------------
// Obfuscation
// ---------------------------------------------------------------------------

const DECODERS: &[&str] = &[
    "base64_decode",
    "gzinflate",
    "gzuncompress",
    "gzdecode",
    "str_rot13",
    "strrev",
    "hex2bin",
    "convert_uudecode",
];

/// Report code to run, or a function to call, that comes out of a decoder: the shape of a web
/// shell or backdoor, `eval(gzinflate(base64_decode('...')))`.
pub(crate) fn check_obfuscated_code(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    code: &Expr,
    ctx: &FlowState,
    span: php_ast::Span,
) {
    if is_decoded(code, ctx) {
        ea.emit(
            mir_issues::IssueKind::ObfuscatedCode,
            mir_issues::Severity::Warning,
            span,
        );
    }
}

/// A decoder's output, possibly concatenated, or a string spelled with three or more `chr()` calls.
pub(crate) fn is_decoded(expr: &Expr, ctx: &FlowState) -> bool {
    has_decoded_part(expr, ctx) || chr_calls(expr) >= 3
}

fn has_decoded_part(expr: &Expr, ctx: &FlowState) -> bool {
    match &expr.kind {
        ExprKind::Variable(name) => ctx.has_value_origin(name, ValueOrigin::Decoded),
        ExprKind::FunctionCall(fc) => named_one_of(&fc.name, DECODERS),
        _ => any_string_part(expr, &mut |part| has_decoded_part(part, ctx)),
    }
}

fn chr_calls(expr: &Expr) -> usize {
    match &expr.kind {
        ExprKind::FunctionCall(fc) if named_one_of(&fc.name, &["chr"]) => 1,
        ExprKind::Parenthesized(inner) => chr_calls(inner),
        ExprKind::Binary(b) if b.op == php_ast::ast::BinaryOp::Concat => {
            chr_calls(&b.left) + chr_calls(&b.right)
        }
        _ => 0,
    }
}
