use super::*;
use crate::ast_ext::ByteStringExt as _;
use crate::ast_ext::NameBareExt as _;

/// A value of this type can't carry an injection payload: numbers, booleans,
/// numeric strings, or string literals the program itself spells out (e.g. after
/// `is_numeric($x)` or `in_array($x, ['a', 'b'], true)` narrowing).
pub fn is_injection_safe_type(ty: &mir_types::Type) -> bool {
    use mir_types::Atomic;
    !ty.types.is_empty()
        && ty.types.iter().all(|a| {
            matches!(
                a,
                Atomic::TInt
                    | Atomic::TLiteralInt(_)
                    | Atomic::TIntRange { .. }
                    | Atomic::TPositiveInt
                    | Atomic::TNegativeInt
                    | Atomic::TNonNegativeInt
                    | Atomic::TFloat
                    | Atomic::TIntegralFloat
                    | Atomic::TLiteralFloat(..)
                    | Atomic::TBool
                    | Atomic::TTrue
                    | Atomic::TFalse
                    | Atomic::TNull
                    | Atomic::TNumeric
                    | Atomic::TNumericString
                    | Atomic::TLiteralString(_)
            )
        })
}

/// Whether every array shape `ty` can be holds an injection-safe value at `key`.
pub(super) fn shape_entry_is_safe(ty: &mir_types::Type, key: mir_types::Name) -> bool {
    use mir_types::atomic::ArrayKey;
    !ty.types.is_empty()
        && ty.types.iter().all(|atom| match atom {
            mir_types::Atomic::TKeyedArray { properties, .. } => properties
                .iter()
                .find(|(k, _)| match k {
                    ArrayKey::String(s) => s.as_ref() == key.as_str(),
                    ArrayKey::Int(i) => i.to_string() == key.as_str(),
                })
                .is_some_and(|(_, prop)| is_injection_safe_type(&prop.ty)),
            _ => false,
        })
}

/// Whether a flags expression includes the constant `flag` (`A | B`).
pub(super) fn has_flag(expr: &Expr, flag: &str) -> bool {
    match &expr.kind {
        ExprKind::Name(_) => expr.kind.name_bare() == Some(flag),
        ExprKind::Binary(op) if op.op == php_ast::ast::BinaryOp::BitwiseOr => {
            has_flag(&op.left, flag) || has_flag(&op.right, flag)
        }
        ExprKind::Parenthesized(inner) => has_flag(inner, flag),
        _ => false,
    }
}

/// Taint of a `filter_var()`/`filter_input()` result: validating filters
/// (`FILTER_VALIDATE_INT`, …) leave nothing injectable, escaping ones remove
/// their own kind, everything else passes the input through.
pub(super) fn filter_call_taint(
    fn_name_lower: &str,
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<Taint> {
    let arg = |i: usize| args.get(i).and_then(|a| a.value());
    let (input, filter) = match fn_name_lower {
        "filter_var" => (
            arg(0).map_or(Taint::NONE, |v| expr_taint(v, ctx, db, file)),
            arg(1),
        ),
        "filter_input" => (Taint::ALL, arg(2)),
        "filter_input_array" => return Some(Taint::ALL),
        _ => return None,
    };
    let Some(filter_name) = filter
        .and_then(|f| f.kind.name_bare())
        .map(str::to_ascii_uppercase)
    else {
        return Some(input);
    };
    Some(match filter_name.as_str() {
        "FILTER_VALIDATE_INT"
        | "FILTER_VALIDATE_FLOAT"
        | "FILTER_VALIDATE_BOOLEAN"
        | "FILTER_VALIDATE_BOOL"
        | "FILTER_VALIDATE_IP"
        | "FILTER_VALIDATE_MAC"
        | "FILTER_SANITIZE_NUMBER_INT"
        | "FILTER_SANITIZE_NUMBER_FLOAT" => Taint::NONE,
        "FILTER_SANITIZE_SPECIAL_CHARS" | "FILTER_SANITIZE_FULL_SPECIAL_CHARS" => {
            input.without(Taint::HTML)
        }
        "FILTER_SANITIZE_ENCODED" => input.without(sanitized_kinds("urlencode")?),
        _ => input,
    })
}

/// Taint kinds a sanitizing builtin removes from its arguments' taint.
pub(super) fn sanitized_kinds(fn_name_lower: &str) -> Option<Taint> {
    Some(match fn_name_lower {
        "htmlspecialchars" | "htmlentities" | "strip_tags" | "e" => Taint::HTML,
        "addslashes"
        | "mysqli_real_escape_string"
        | "mysqli_escape_string"
        | "mysql_real_escape_string"
        | "mysql_escape_string"
        | "pg_escape_string"
        | "pg_escape_literal"
        | "pg_escape_identifier"
        | "pg_escape_bytea"
        | "sqlite_escape_string"
        | "db2_escape_string" => Taint::SQL,
        "escapeshellarg" | "escapeshellcmd" => Taint::SHELL,
        "ldap_escape" => Taint::LDAP,
        // Strips directory components, so the value can't traverse out of its directory.
        "basename" => Taint::FILE,
        // Output is limited to `[A-Za-z0-9%+._~-]`.
        "urlencode" | "rawurlencode" | "http_build_query" => {
            Taint::HTML
                | Taint::SQL
                | Taint::SHELL
                | Taint::HEADER
                | Taint::COOKIE
                | Taint::LDAP
                | Taint::XPATH
        }
        // Output is limited to `[A-Za-z0-9+/=]`.
        "base64_encode" => Taint::HTML | Taint::HEADER | Taint::COOKIE,
        "md5" | "sha1" | "crc32" | "hash" | "hash_hmac" | "password_hash" | "bin2hex"
        | "dechex" | "decbin" | "decoct" | "number_format" | "uniqid" => Taint::ALL,
        _ => return None,
    })
}

/// Whether the literal text opens with `http(s)://host/` (or `ftp(s)`), fixing scheme and host
/// so whatever is appended lands in the path or query. `host` must end at a `/`, `?` or `#`.
fn is_fixed_url_origin(literal: &str) -> bool {
    let Some((scheme, rest)) = literal.split_once("://") else {
        return false;
    };
    if !["http", "https", "ftp", "ftps"]
        .iter()
        .any(|s| scheme.eq_ignore_ascii_case(s))
    {
        return false;
    }
    match rest.find(['/', '?', '#']) {
        Some(end) => {
            let host = &rest[..end];
            !host.is_empty() && !host.contains(['@', '\\', ' '])
        }
        None => false,
    }
}

/// Whether the expression is a string that starts with a fixed `scheme://host/` origin.
pub(super) fn starts_with_fixed_url_origin(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::String(s) => is_fixed_url_origin(s.as_str()),
        ExprKind::Parenthesized(inner) => starts_with_fixed_url_origin(inner),
        ExprKind::Binary(b) if matches!(b.op, php_ast::ast::BinaryOp::Concat) => {
            starts_with_fixed_url_origin(&b.left)
        }
        ExprKind::InterpolatedString(parts) | ExprKind::Heredoc { parts, .. } => {
            // Adjacent literal parts form one run; stop at the first interpolation.
            let mut prefix = String::new();
            for part in parts.iter() {
                match part {
                    StringPart::Literal(l) => prefix.push_str(l.as_str()),
                    _ => break,
                }
            }
            is_fixed_url_origin(&prefix)
        }
        _ => false,
    }
}

/// Taint kinds a decoding builtin restores on already-tainted input.
pub(super) fn unescaped_kinds(fn_name_lower: &str) -> Option<Taint> {
    Some(match fn_name_lower {
        "html_entity_decode" | "htmlspecialchars_decode" => Taint::HTML,
        "stripslashes" | "stripcslashes" => Taint::SQL,
        "urldecode" | "rawurldecode" => sanitized_kinds("urlencode")?,
        _ => return None,
    })
}

/// Methods whose result comes from server-side state or the arguments, never from the receiver's tainted data.
pub(super) fn result_ignores_receiver(
    db: &dyn crate::db::MirDatabase,
    fqcn: &str,
    method_lower: &str,
) -> bool {
    let is_a = |base: &str| {
        fqcn.eq_ignore_ascii_case(base) || crate::db::extends_or_implements(db, fqcn, base)
    };
    // A date renders through the format argument; the parsed input is never echoed.
    if matches!(
        method_lower,
        "format" | "gettimezone" | "getname" | "getoffset" | "gettimestamp"
    ) && ["DateTimeInterface", "DateInterval", "DateTimeZone"]
        .iter()
        .any(|base| is_a(base))
    {
        return true;
    }
    matches!(
        method_lower,
        "hashname"
            | "store"
            | "storeas"
            | "storepublicly"
            | "storepubliclyas"
            | "getrealpath"
            | "getpathname"
            | "getpath"
            | "path"
    ) && is_a("Symfony\\Component\\HttpFoundation\\File\\UploadedFile")
}

/// Taint kinds an escaping or template-rendering method of a known class removes.
pub(super) fn sanitized_method_kinds(
    db: &dyn crate::db::MirDatabase,
    fqcn: &str,
    method_lower: &str,
) -> Option<Taint> {
    let is_a = |base: &str| {
        fqcn.eq_ignore_ascii_case(base) || crate::db::extends_or_implements(db, fqcn, base)
    };
    let escapes_sql = match method_lower {
        "quote" => is_a("PDO"),
        "real_escape_string" | "escape_string" => is_a("mysqli"),
        "escapestring" => is_a("SQLite3"),
        _ => false,
    };
    if escapes_sql {
        return Some(Taint::SQL);
    }
    let escapes_html = matches!(
        method_lower,
        "escapehtml" | "escapehtmlattr" | "escapejs" | "escapecss" | "escapeurl"
    ) && (is_a("Laminas\\Escaper\\Escaper") || is_a("Zend\\Escaper\\Escaper"));
    if escapes_html {
        return Some(Taint::HTML);
    }
    // Twig autoescapes the variables it renders into the template.
    let renders_template = matches!(method_lower, "render" | "renderblock")
        && (is_a("Twig\\Environment") || is_a("Twig\\TemplateWrapper") || is_a("Twig\\Template"));
    renders_template.then_some(Taint::ALL)
}
