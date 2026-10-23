use super::*;
use crate::ast_ext::ArgExt as _;
use crate::ast_ext::ByteStringExt as _;

// ---------------------------------------------------------------------------
// Superglobal names (without the $ prefix, as stored in FlowState::vars)
// ---------------------------------------------------------------------------

/// Superglobals holding request data; `$_ENV` comes from the process environment.
pub const SUPERGLOBALS: &[&str] = &["_GET", "_POST", "_REQUEST", "_COOKIE", "_FILES", "_SERVER"];

pub fn is_superglobal(name: &str) -> bool {
    SUPERGLOBALS.contains(&name)
}

/// `$_SERVER` keys a client controls; the rest come from the server or environment.
pub(super) fn is_request_server_key(key: &str) -> bool {
    key.starts_with("HTTP_")
        || matches!(
            key,
            "REQUEST_URI"
                | "QUERY_STRING"
                | "PHP_SELF"
                | "PATH_INFO"
                | "PATH_TRANSLATED"
                | "ORIG_PATH_INFO"
                | "REQUEST_METHOD"
                | "CONTENT_TYPE"
                | "CONTENT_LENGTH"
                | "PHP_AUTH_USER"
                | "PHP_AUTH_PW"
                | "PHP_AUTH_DIGEST"
                | "REMOTE_USER"
                | "REDIRECT_URL"
                | "REDIRECT_QUERY_STRING"
                | "UNENCODED_URL"
                | "SERVER_NAME"
        )
}

/// The key of a `$_SERVER[$key]` read whose entry isn't known statically.
pub fn server_dynamic_key(aa: &php_ast::owned::ArrayAccessExpr) -> Option<&Expr> {
    let key = aa.index.as_deref()?;
    (is_superglobal_var(&aa.array, "_SERVER")
        && !matches!(key.kind, ExprKind::String(_) | ExprKind::Int(_)))
    .then_some(key)
}

/// Whether a key expression starts with the `HTTP_` header prefix.
pub fn starts_with_header_prefix(key: &Expr) -> bool {
    match &key.kind {
        ExprKind::String(s) => s.as_str().starts_with("HTTP_"),
        ExprKind::Parenthesized(inner) => starts_with_header_prefix(inner),
        ExprKind::Binary(b) if matches!(b.op, php_ast::ast::BinaryOp::Concat) => {
            starts_with_header_prefix(&b.left)
        }
        ExprKind::InterpolatedString(parts) => matches!(
            parts.first(),
            Some(StringPart::Literal(l)) if l.as_str().starts_with("HTTP_")
        ),
        _ => false,
    }
}

/// Taint of `$_SERVER[$key]`: request entries when the key can name one,
/// otherwise whatever the key itself carries.
pub(super) fn server_dynamic_read_taint(
    key: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    if starts_with_header_prefix(key) {
        return Taint::ALL;
    }
    if let ExprKind::Variable(name) = &unwrap_parens(key).kind {
        let ty = ctx.get_var(name);
        let literals: Option<Vec<&str>> = ty
            .types
            .iter()
            .map(|a| match a {
                mir_types::Atomic::TLiteralString(s) => Some(s.as_ref()),
                _ => None,
            })
            .collect();
        if let Some(literals) = literals.filter(|l| !l.is_empty()) {
            return if literals.iter().any(|l| is_request_server_key(l)) {
                Taint::ALL
            } else {
                Taint::NONE
            };
        }
    }
    expr_taint(key, ctx, db, file)
}

/// Key argument of a Symfony `$request->server->get($key)` read, when the receiver is the `server` bag.
pub fn server_bag_read_key<'a>(
    object: &Expr,
    method: &Expr,
    args: &'a [php_ast::owned::Arg],
) -> Option<&'a Expr> {
    let first = args.first().filter(|a| !a.unpack() && a.name.is_none())?;
    let (ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa)) =
        &unwrap_parens(object).kind
    else {
        return None;
    };
    let prop = match &pa.property.kind {
        ExprKind::Identifier(prop) => &**prop,
        ExprKind::String(prop) => prop.as_str(),
        _ => return None,
    };
    let ExprKind::Identifier(method) = &method.kind else {
        return None;
    };
    (prop == "server"
        && ["get", "getstring", "filter"]
            .iter()
            .any(|name| method.eq_ignore_ascii_case(name)))
    .then_some(first.value()?)
}

/// Whether a key expression names a client-controlled `$_SERVER` entry.
pub fn is_request_server_key_expr(key: &Expr) -> bool {
    starts_with_header_prefix(key)
        || matches!(&unwrap_parens(key).kind, ExprKind::String(k) if is_request_server_key(k.as_str()))
}

/// Taint of a `$request->server->get($key, $default)` read.
pub(super) fn server_bag_read_taint(
    key: &Expr,
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    let entry = match &unwrap_parens(key).kind {
        ExprKind::String(k) if is_request_server_key(k.as_str()) => Taint::ALL,
        ExprKind::String(_) => Taint::NONE,
        _ => server_dynamic_read_taint(key, ctx, db, file),
    };
    entry | args_taint(&args[1..], ctx, db, file)
}

pub(super) fn is_superglobal_var(expr: &Expr, name: &str) -> bool {
    matches!(&expr.kind, ExprKind::Variable(v) if v.trim_start_matches('$') == name)
}

/// A superglobal entry PHP, the server or the environment sets rather than the
/// client: `$_SERVER['DOCUMENT_ROOT']`, `$_FILES[x]['tmp_name']`.
pub fn is_trusted_superglobal_field(aa: &php_ast::owned::ArrayAccessExpr) -> bool {
    let Some(ExprKind::String(key)) = aa.index.as_deref().map(|i| &i.kind) else {
        return false;
    };
    if is_superglobal_var(&aa.array, "_SERVER") {
        return !is_request_server_key(key.as_str());
    }
    let ExprKind::ArrayAccess(file_entry) = &aa.array.kind else {
        return false;
    };
    matches!(key.as_str(), "tmp_name" | "size" | "error")
        && is_superglobal_var(&file_entry.array, "_FILES")
}

/// The global variable `$GLOBALS['name']` reads.
pub(super) fn globals_entry(aa: &php_ast::owned::ArrayAccessExpr) -> Option<&str> {
    let Some(ExprKind::String(key)) = aa.index.as_deref().map(|i| &i.kind) else {
        return None;
    };
    is_superglobal_var(&aa.array, "GLOBALS").then_some(key.as_str())
}

/// `$GLOBALS['_GET']` reads the superglobal itself.
pub fn is_globals_superglobal_alias(aa: &php_ast::owned::ArrayAccessExpr) -> bool {
    matches!(aa.index.as_deref().map(|i| &i.kind), Some(ExprKind::String(key)) if is_superglobal(key.as_str()))
        && is_superglobal_var(&aa.array, "GLOBALS")
}

pub(super) const SYMFONY_REQUEST: &str = "Symfony\\Component\\HttpFoundation\\Request";

/// `Request` properties holding client data (`$request->query->get('q')`).
/// `attributes` is left out: the router fills it (`_controller`, `_route`).
pub(super) const REQUEST_BAG_PROPERTIES: &[&str] =
    &["query", "request", "cookies", "headers", "files", "server"];

/// Request accessors returning client data: Symfony's `Request`, Laravel's
/// subclass, and PSR-7 server requests.
pub fn is_framework_request_source(
    db: &dyn crate::db::MirDatabase,
    fqcn: &str,
    method_lower: &str,
) -> bool {
    let is_a = |base: &str| {
        fqcn.eq_ignore_ascii_case(base) || crate::db::extends_or_implements(db, fqcn, base)
    };
    let symfony_accessor = matches!(
        method_lower,
        "get"
            | "getcontent"
            | "getpayload"
            | "toarray"
            | "geturi"
            | "getrequesturi"
            | "getpathinfo"
            | "getquerystring"
            | "gethost"
            | "gethttphost"
            | "getschemeandhttphost"
            | "getbaseurl"
            | "getbasepath"
            | "getuserinfo"
            | "getuser"
            | "getpassword"
            | "getclientip"
            | "getclientips"
            | "getpreferredlanguage"
            | "getlanguages"
            | "getcharsets"
            | "getencodings"
            | "getacceptablecontenttypes"
            | "getetags"
            | "getrequestformat"
    ) || is_laravel_request_accessor(method_lower);
    let server_request_accessor = matches!(
        method_lower,
        "getqueryparams"
            | "getparsedbody"
            | "getcookieparams"
            | "getuploadedfiles"
            | "getserverparams"
            | "getheader"
            | "getheaderline"
            | "getheaders"
            | "getbody"
            | "geturi"
            | "getrequesttarget"
    );
    // Most method names are no accessor of any request type: skip the hierarchy walks.
    if !(symfony_accessor || server_request_accessor) {
        return false;
    }
    if is_a(SYMFONY_REQUEST) {
        return symfony_accessor;
    }
    // A plain `RequestInterface` is an outgoing client request: its data is the caller's own.
    server_request_accessor && is_a("Psr\\Http\\Message\\ServerRequestInterface")
}

/// A branch merge can leave `Sub|Base` in a union; `Sub` adds nothing over `Base`.
pub(super) fn absorbed_by_supertype(
    db: &dyn crate::db::MirDatabase,
    fqcn: &str,
    ty: &mir_types::Type,
) -> bool {
    ty.types.iter().any(|atom| {
        matches!(atom, mir_types::Atomic::TNamedObject { fqcn: other, .. }
            if !other.eq_ignore_ascii_case(fqcn) && crate::db::extends_or_implements(db, fqcn, other))
    })
}

pub(super) fn is_laravel_request_accessor(method_lower: &str) -> bool {
    matches!(
        method_lower,
        "input"
            | "query"
            | "post"
            | "cookie"
            | "header"
            | "server"
            | "all"
            | "only"
            | "except"
            | "json"
            | "string"
            | "str"
            | "keys"
            | "collect"
            | "route"
            | "fullurl"
            | "fullurlwithquery"
            | "fullurlwithoutquery"
            | "url"
            | "path"
            | "decodedpath"
            | "segment"
            | "segments"
            | "ip"
            | "ips"
            | "useragent"
            | "bearertoken"
            | "validated"
            | "safe"
            | "old"
            | "file"
            | "allfiles"
    )
}

/// `Request::input('q')` through Laravel's facades.
pub fn is_laravel_request_facade_source(class: &str, method_lower: &str) -> bool {
    matches!(
        class.trim_start_matches('\\'),
        "Request"
            | "Input"
            | "Illuminate\\Support\\Facades\\Request"
            | "Illuminate\\Support\\Facades\\Input"
    ) && (method_lower == "get" || is_laravel_request_accessor(method_lower))
}

/// `$request->query` and the other client-data bags of a Symfony request.
pub fn is_request_bag_property(db: &dyn crate::db::MirDatabase, fqcn: &str, prop: &str) -> bool {
    REQUEST_BAG_PROPERTIES.contains(&prop)
        && (fqcn.eq_ignore_ascii_case(SYMFONY_REQUEST)
            || crate::db::extends_or_implements(db, fqcn, SYMFONY_REQUEST))
}

/// Laravel's `request('q')` helper reads input; `request()` returns the request object.
pub fn is_request_helper_source(fn_name_lower: &str, args: &[php_ast::owned::Arg]) -> bool {
    fn_name_lower == "request" && !args.is_empty()
}

/// Builtins whose result is request data.
pub fn is_source_function(fn_name_lower: &str) -> bool {
    matches!(
        fn_name_lower,
        "getallheaders" | "apache_request_headers" | "filter_input" | "filter_input_array"
    )
}

/// `file_get_contents('php://input')` and friends read the raw request body.
pub(super) fn reads_raw_request_body(fn_name_lower: &str, args: &[php_ast::owned::Arg]) -> bool {
    matches!(
        fn_name_lower,
        "file_get_contents" | "fopen" | "file" | "readfile" | "stream_get_contents"
    ) && matches!(
        args.first().and_then(|a| a.value()).map(|v| &v.kind),
        Some(ExprKind::String(path))
            if path.as_str().eq_ignore_ascii_case("php://input")
    )
}
