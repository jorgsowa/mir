use super::*;
use crate::ast_ext::ArgExt as _;
use crate::ast_ext::ByteStringExt as _;
use crate::ast_ext::NameBareExt as _;
use crate::flow_state::ValueOrigin;

/// Report a call target or class name chosen by user input (`$_GET['f']()`).
pub fn check_callable_target(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    target: &Expr,
    ctx: &FlowState,
    span: php_ast::Span,
) {
    let issue = mir_issues::IssueKind::TaintedCallable;
    if reaches_sink(
        target,
        Taint::CALLABLE,
        &issue,
        ctx,
        ea.db,
        &ea.file,
        ea.taint_summary,
    ) {
        ea.emit_taint(issue, span, target, Taint::CALLABLE, ctx);
    }
}

/// The sinks `@taint-sink` tags name, by position of the parameter they name.
pub(super) fn annotated_param_sinks(
    params: &[mir_codebase::DeclaredParam],
    sink_params: &[(std::sync::Arc<str>, std::sync::Arc<str>)],
) -> Vec<ParamSink> {
    sink_params
        .iter()
        .filter_map(|(name, kind)| {
            Some(ParamSink {
                param: params
                    .iter()
                    .position(|p| p.name.as_ref() == name.as_ref())?,
                kinds: taint_sink_kind(kind),
                issue: taint_sink_issue(kind),
            })
        })
        .collect()
}

/// Report arguments reaching a sink inside the callee: its `@taint-sink`
/// parameters and the sinks its inferred summary records.
#[allow(clippy::too_many_arguments)]
pub fn check_call_sinks(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    ctx: &FlowState,
    params: &[mir_codebase::DeclaredParam],
    args: &[php_ast::owned::Arg],
    arg_types: &[mir_types::Type],
    receiver: Option<&Expr>,
    sink_params: &[(std::sync::Arc<str>, std::sync::Arc<str>)],
    summary: impl FnOnce(&dyn crate::db::MirDatabase, &str) -> Option<TaintSummary>,
    span: php_ast::Span,
) {
    if params.len() >= 2 || params.iter().any(|p| p.is_variadic) {
        for spread in args.iter().filter(|a| a.unpack()).filter_map(|a| a.value()) {
            check_named_arg_spread(ea, spread, ctx, span);
        }
    }
    let annotated = annotated_param_sinks(params, sink_params);
    let mut callee = TaintSummary::default();
    if !demanding_taint(args_value_taint(args, ctx, ea.db, &ea.file), ctx).is_empty()
        || receiver_demands_summary(receiver, ctx, ea.db, &ea.file)
    {
        if let Some(summary) = summary(ea.db, &ea.file) {
            callee = summary;
        }
    }
    // Annotations name declared parameters; only the summary's slots may be labels.
    let unlabelled = TaintSummary::default();
    let sinks = annotated
        .iter()
        .map(|sink| (sink, &unlabelled))
        .chain(callee.param_sinks.iter().map(|sink| (sink, &callee)));
    let mut emitted: Vec<mir_issues::IssueKind> = Vec::new();
    for (sink, labels) in sinks {
        if let Some(prop) = labels.prop_label(sink.param) {
            let (Some(receiver), taint) = (
                receiver,
                receiver_prop_taint(receiver, prop, ctx, ea.db, &ea.file),
            ) else {
                continue;
            };
            ea.taint_summary
                .record_param_sinks(taint, sink.kinds, &sink.issue);
            if taint.concrete().intersects(sink.kinds) && !emitted.contains(&sink.issue) {
                emitted.push(sink.issue.clone());
                ea.emit_taint(sink.issue.clone(), span, receiver, sink.kinds, ctx);
            }
            continue;
        }
        for value in labels.slot_args(sink.param, params, args, arg_types, ctx, ea.db, &ea.file) {
            let taint = labels.slot_arg_taint(sink.param, value, ctx, ea.db, &ea.file);
            ea.taint_summary
                .record_param_sinks(taint, sink.kinds, &sink.issue);
            if taint.concrete().intersects(sink.kinds) && !emitted.contains(&sink.issue) {
                emitted.push(sink.issue.clone());
                ea.emit_taint(sink.issue.clone(), span, value, sink.kinds, ctx);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Sink classification
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinkKind {
    Html,        // echo / print → XSS
    Sql,         // DB query functions → SQL injection
    Shell,       // system / exec / shell_exec → command injection
    File,        // filesystem path args → path traversal / local file inclusion
    Unserialize, // unserialize → PHP object injection
    Header,      // header → response splitting / open redirect
    Cookie,      // setcookie / setrawcookie → cookie injection
    Ssrf,        // connection targets → server-side request forgery
    Ldap,        // LDAP filters and DNs → LDAP injection
    Xpath,       // XPath expressions → XPath injection
    Callable,    // callbacks and dynamic calls → arbitrary code paths
    Xml,         // XML parsed with entity expansion → XXE / XML denial of service
}

impl SinkKind {
    pub fn issue(self) -> mir_issues::IssueKind {
        use mir_issues::IssueKind;
        match self {
            SinkKind::Html => IssueKind::TaintedHtml,
            SinkKind::Sql => IssueKind::TaintedSql,
            SinkKind::Shell => IssueKind::TaintedShell,
            SinkKind::Header => IssueKind::TaintedHeader,
            SinkKind::Cookie => IssueKind::TaintedCookie,
            SinkKind::File => IssueKind::TaintedFile,
            SinkKind::Unserialize => IssueKind::TaintedUnserialize,
            SinkKind::Ssrf => IssueKind::TaintedSsrf,
            SinkKind::Ldap => IssueKind::TaintedLdap,
            SinkKind::Xpath => IssueKind::TaintedXpath,
            SinkKind::Callable => IssueKind::TaintedCallable,
            SinkKind::Xml => IssueKind::TaintedXml,
        }
    }

    pub fn taint(self) -> Taint {
        match self {
            SinkKind::Html => Taint::HTML,
            SinkKind::Sql => Taint::SQL,
            SinkKind::Shell => Taint::SHELL,
            SinkKind::File => Taint::FILE,
            SinkKind::Unserialize => Taint::UNSERIALIZE,
            SinkKind::Header => Taint::HEADER,
            SinkKind::Cookie => Taint::COOKIE,
            SinkKind::Ssrf => Taint::SSRF,
            SinkKind::Ldap => Taint::LDAP,
            SinkKind::Xpath => Taint::XPATH,
            SinkKind::Callable => Taint::CALLABLE,
            SinkKind::Xml => Taint::OTHER,
        }
    }
}

/// One dangerous parameter of a builtin sink. `index: None` means every argument.
#[derive(Debug, Clone, Copy)]
pub struct SinkParam {
    pub index: Option<usize>,
    pub name: &'static str,
    pub kind: SinkKind,
}

pub(super) const fn any(kind: SinkKind) -> SinkParam {
    SinkParam {
        index: None,
        name: "",
        kind,
    }
}

pub(super) const fn at(index: usize, name: &'static str, kind: SinkKind) -> SinkParam {
    SinkParam {
        index: Some(index),
        name,
        kind,
    }
}

macro_rules! sinks {
    ($($param:expr),* $(,)?) => {{
        const PARAMS: &[SinkParam] = &[$($param),*];
        PARAMS
    }};
}

/// The parameters of builtin `fn_name` that are taint sinks.
pub fn builtin_sinks(fn_name: &str) -> &'static [SinkParam] {
    use SinkKind::*;
    match crate::util::php_ident_lowercase(fn_name.trim_start_matches('\\')).as_str() {
        "echo" | "print" | "printf" | "vprintf" | "fprintf" | "vfprintf" | "var_dump" => {
            sinks![any(Html)]
        }
        "print_r" | "var_export" | "highlight_string" => sinks![at(0, "value", Html)],

        "header" => sinks![at(0, "header", Header)],
        "setcookie" | "setrawcookie" => sinks![at(0, "name", Cookie), at(1, "value", Cookie)],
        "mail" | "mb_send_mail" => sinks![
            at(3, "additional_headers", Header),
            at(4, "additional_params", Shell),
        ],

        "mysql_query"
        | "mysqli_query"
        | "mysqli_real_query"
        | "mysqli_multi_query"
        | "mysql_unbuffered_query"
        | "mysql_db_query"
        | "mysqli_prepare"
        | "mysqli_stmt_prepare"
        | "pg_query"
        | "pg_send_query"
        | "pg_prepare"
        | "pg_exec"
        | "sqlite_query"
        | "sqlite_exec"
        | "sqlite_array_query"
        | "sqlite_single_query"
        | "sqlite_unbuffered_query"
        | "mssql_query"
        | "odbc_exec"
        | "odbc_prepare"
        | "oci_parse"
        | "db2_exec"
        | "db2_prepare" => sinks![any(Sql)],
        "mysqli_execute_query" => sinks![at(1, "query", Sql)],
        "sqlsrv_query" | "sqlsrv_prepare" => sinks![at(1, "sql", Sql)],

        "system" | "exec" | "shell_exec" | "passthru" | "popen" | "proc_open" | "expect_popen" => {
            sinks![at(0, "command", Shell)]
        }
        "pcntl_exec" => sinks![at(0, "path", Shell), at(1, "args", Shell)],
        "ssh2_exec" => sinks![at(1, "command", Shell)],

        "fopen"
        | "file_get_contents"
        | "file_put_contents"
        | "readfile"
        | "file"
        | "unlink"
        | "touch"
        | "chmod"
        | "chown"
        | "parse_ini_file"
        | "simplexml_load_file"
        | "highlight_file"
        | "show_source"
        | "php_strip_whitespace"
        | "getimagesize"
        | "gzopen"
        | "gzfile"
        | "readgzfile"
        | "chgrp"
        | "exif_read_data"
        | "exif_thumbnail"
        | "imagecreatefromavif"
        | "imagecreatefrombmp"
        | "imagecreatefromgd"
        | "imagecreatefromgd2"
        | "imagecreatefromgif"
        | "imagecreatefromjpeg"
        | "imagecreatefrompng"
        | "imagecreatefromtga"
        | "imagecreatefromwbmp"
        | "imagecreatefromwebp"
        | "imagecreatefromxbm"
        | "imagecreatefromxpm" => sinks![at(0, "filename", File)],
        "finfo_file" => sinks![at(1, "filename", File)],
        "mkdir" | "rmdir" | "scandir" | "opendir" | "chdir" => sinks![at(0, "directory", File)],
        "copy" | "rename" => sinks![at(0, "from", File), at(1, "to", File)],
        "symlink" | "link" => sinks![at(0, "target", File), at(1, "link", File)],
        "readlink" => sinks![at(0, "path", File)],
        "glob" => sinks![at(0, "pattern", File)],
        "dl" => sinks![at(0, "extension_filename", File)],
        "move_uploaded_file" => sinks![at(1, "to", File)],

        "fsockopen" | "pfsockopen" => sinks![at(0, "hostname", Ssrf)],
        "get_headers" | "curl_init" => sinks![at(0, "url", Ssrf)],
        "stream_socket_client" => sinks![at(0, "address", Ssrf)],
        "ftp_connect" | "ftp_ssl_connect" | "ssh2_connect" | "ldap_connect" => {
            sinks![at(0, "hostname", Ssrf)]
        }
        "socket_connect" => sinks![at(1, "address", Ssrf)],

        "ldap_search" | "ldap_list" | "ldap_read" => {
            sinks![at(1, "base", Ldap), at(2, "filter", Ldap)]
        }
        "ldap_delete" | "ldap_compare" | "ldap_bind" | "ldap_add" | "ldap_modify"
        | "ldap_mod_add" | "ldap_mod_del" | "ldap_mod_replace" | "ldap_rename" => {
            sinks![at(1, "dn", Ldap)]
        }

        "unserialize" | "igbinary_unserialize" | "yaml_parse" | "session_decode" => {
            sinks![at(0, "data", Unserialize)]
        }

        "call_user_func"
        | "call_user_func_array"
        | "forward_static_call"
        | "forward_static_call_array"
        | "register_shutdown_function"
        | "register_tick_function"
        | "spl_autoload_register"
        | "set_error_handler"
        | "set_exception_handler"
        | "array_map"
        | "ob_start" => sinks![at(0, "callback", Callable)],
        "usort"
        | "uasort"
        | "uksort"
        | "array_filter"
        | "array_walk"
        | "array_walk_recursive"
        | "array_reduce"
        | "iterator_apply"
        | "preg_replace_callback" => {
            sinks![at(1, "callback", Callable)]
        }

        _ => sinks![],
    }
}

/// The first sink kind of builtin `fn_name`, if it is a sink.
pub fn classify_sink(fn_name: &str) -> Option<SinkKind> {
    builtin_sinks(fn_name).first().map(|p| p.kind)
}

/// Values reaching sinks that depend on another argument: `curl_setopt($ch,
/// CURLOPT_URL, $url)`, `curl_setopt_array($ch, [CURLOPT_URL => $url])`.
pub fn option_sink_values<'e>(
    fn_name_lower: &str,
    args: &'e [php_ast::owned::Arg],
) -> Vec<(&'e Expr, SinkKind)> {
    let option_kind = |e: &Expr| match e.kind.name_bare() {
        Some("CURLOPT_URL" | "CURLOPT_PROXY") => Some(SinkKind::Ssrf),
        Some("CURLOPT_HTTPHEADER") => Some(SinkKind::Header),
        _ => None,
    };
    match fn_name_lower.trim_start_matches('\\') {
        "curl_setopt" => match (args.get(1).and_then(|a| a.value()), args.get(2)) {
            (Some(option), Some(value)) => option_kind(option)
                .and_then(|kind| value.value().map(|v| (v, kind)))
                .into_iter()
                .collect(),
            _ => Vec::new(),
        },
        "curl_setopt_array" => match args.get(1).and_then(|a| a.value()).map(|v| &v.kind) {
            Some(ExprKind::Array(elements)) => elements
                .iter()
                .filter_map(|el| Some((el.value(), option_kind(el.key.as_ref()?)?)))
                .collect(),
            _ => Vec::new(),
        },
        "simplexml_load_string" => xml_with_unsafe_flags(args, 0, "data", 2, "options")
            .map(|v| (v, SinkKind::Xml))
            .into_iter()
            .collect(),
        _ => Vec::new(),
    }
}

/// The XML argument of a parse call whose libxml flags load external entities or
/// DTDs, or lift the size limits.
fn xml_with_unsafe_flags<'e>(
    args: &'e [php_ast::owned::Arg],
    xml_index: usize,
    xml_name: &str,
    flags_index: usize,
    flags_name: &str,
) -> Option<&'e Expr> {
    let arg = |index: usize, name: &str| {
        args.iter()
            .find(|a| a.name.as_ref().is_some_and(|n| n.last_part() == name))
            .or_else(|| args.get(index).filter(|a| a.name.is_none()))
            .and_then(|a| a.value())
    };
    let flags = arg(flags_index, flags_name)?;
    ["LIBXML_NOENT", "LIBXML_DTDLOAD", "LIBXML_PARSEHUGE"]
        .iter()
        .any(|flag| has_flag(flags, flag))
        .then(|| arg(xml_index, xml_name))
        .flatten()
}

/// The XML a `DOMDocument::loadXML()` or `XMLReader::XML()` call parses with unsafe flags.
pub fn xml_method_sink_value<'e>(
    db: &dyn crate::db::MirDatabase,
    fqcn: &str,
    method_lower: &str,
    args: &'e [php_ast::owned::Arg],
) -> Option<&'e Expr> {
    let is_a = |base: &str| {
        fqcn.eq_ignore_ascii_case(base) || crate::db::extends_or_implements(db, fqcn, base)
    };
    match method_lower {
        "loadxml" if is_a("DOMDocument") => xml_with_unsafe_flags(args, 0, "source", 1, "options"),
        "xml" if is_a("XMLReader") => xml_with_unsafe_flags(args, 0, "source", 2, "flags"),
        _ => None,
    }
}

/// `proc_open([...])` runs the program directly, so its arguments can't inject shell syntax.
pub fn proc_open_bypasses_shell(args: &[php_ast::owned::Arg], ctx: &FlowState) -> bool {
    let Some(command) = args.first().and_then(|a| a.value()) else {
        return false;
    };
    match &command.kind {
        ExprKind::Array(_) => true,
        ExprKind::Variable(name) => {
            let ty = ctx.get_var(name.trim_start_matches('$'));
            !ty.types.is_empty() && ty.types.iter().all(|a| a.is_array())
        }
        _ => false,
    }
}

/// `unserialize($data, ['allowed_classes' => false])`, or a literal class list, can't reach a gadget.
pub fn unserialize_disallows_classes(args: &[php_ast::owned::Arg], ctx: &FlowState) -> bool {
    use mir_types::atomic::ArrayKey;
    use mir_types::Atomic;
    let Some(options) = args.get(1).and_then(|a| a.value()) else {
        return false;
    };
    match &options.kind {
        ExprKind::Array(options) => options.iter().any(|el| {
            matches!(el.key.as_ref().map(|k| &k.kind), Some(ExprKind::String(k)) if k.as_str() == "allowed_classes")
                && match &el.value().kind {
                    ExprKind::Bool(false) => true,
                    ExprKind::Array(classes) => classes.iter().all(|class| {
                        matches!(
                            class.value().kind,
                            ExprKind::String(_) | ExprKind::ClassConstAccess(_)
                        )
                    }),
                    _ => false,
                }
        }),
        ExprKind::Variable(name) => matches!(
            ctx.get_var(name).types.as_slice(),
            [Atomic::TKeyedArray { properties, .. }]
                if properties.iter().any(|(key, prop)| {
                    matches!(key, ArrayKey::String(k) if k.as_ref() == "allowed_classes")
                        && matches!(
                            prop.ty.types.as_slice(),
                            [Atomic::TFalse] | [Atomic::TKeyedArray { is_open: false, .. }]
                        )
                })
        ),
        _ => false,
    }
}

/// Whether a printing call stays out of the response: `fprintf(STDERR, ...)` writes to the error
/// stream, as `fprintf($h, ...)` does to a file `$h = fopen('log.txt', 'a')` opened, and
/// `print_r($x, $flag)` may hand its output back unless the flag is literally false.
pub fn output_stays_out_of_response(
    fn_name_lower: &str,
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
) -> bool {
    let arg_kind = |index: usize| args.get(index).and_then(|a| a.value()).map(|v| &v.kind);
    match fn_name_lower {
        "fprintf" | "vfprintf" => match arg_kind(0) {
            Some(ExprKind::Variable(handle)) => {
                ctx.has_value_origin(handle, ValueOrigin::FileHandle)
            }
            Some(handle) => handle.name_bare() == Some("STDERR"),
            None => false,
        },
        "print_r" | "var_export" | "highlight_string" => {
            !matches!(arg_kind(1), None | Some(ExprKind::Bool(false)))
        }
        _ => false,
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
    if !matches!(
        method.as_str(),
        "query"
            | "exec"
            | "prepare"
            | "real_query"
            | "multi_query"
            | "evaluate"
            | "xpath"
            | "execute_query"
            | "querysingle"
            | "executequery"
            | "executestatement"
            | "executeupdate"
            | "fetchone"
            | "fetchassociative"
            | "fetchnumeric"
            | "fetchallassociative"
            | "fetchallnumeric"
            | "fetchallkeyvalue"
            | "fetchallassociativeindexed"
            | "fetchfirstcolumn"
            | "iterateassociative"
            | "iteratenumeric"
            | "iteratecolumn"
    ) {
        return None;
    }
    let is_a = |base: &str| {
        fqcn.eq_ignore_ascii_case(base) || crate::db::extends_or_implements(db, fqcn, base)
    };

    if is_a("PDO") && matches!(method.as_str(), "query" | "exec" | "prepare") {
        return Some(SinkKind::Sql);
    }
    if is_a("mysqli")
        && matches!(
            method.as_str(),
            "query" | "prepare" | "real_query" | "multi_query" | "execute_query"
        )
    {
        return Some(SinkKind::Sql);
    }
    if is_a("mysqli_stmt") && method == "prepare" {
        return Some(SinkKind::Sql);
    }
    if is_a("SQLite3")
        && matches!(
            method.as_str(),
            "query" | "exec" | "prepare" | "querysingle"
        )
    {
        return Some(SinkKind::Sql);
    }
    if is_a("Doctrine\\DBAL\\Connection")
        && !matches!(
            method.as_str(),
            "real_query" | "multi_query" | "evaluate" | "xpath" | "execute_query" | "querysingle"
        )
    {
        return Some(SinkKind::Sql);
    }
    if is_a("DOMXPath") && matches!(method.as_str(), "query" | "evaluate") {
        return Some(SinkKind::Xpath);
    }
    if is_a("SimpleXMLElement") && method == "xpath" {
        return Some(SinkKind::Xpath);
    }
    None
}

/// Constructors whose first argument is a sink: `new SplFileObject($path)`.
/// Only the stub constructor counts: a subclass overriding `__construct`, or
/// `SplTempFileObject` (first argument isn't a path), takes other arguments.
pub fn classify_constructor_sink(
    db: &dyn crate::db::MirDatabase,
    fqcn: &str,
    args: &[php_ast::owned::Arg],
) -> Option<SinkKind> {
    let (declarer, _) =
        crate::db::find_method_in_chain(db, crate::db::Fqcn::from_str(db, fqcn), "__construct")?;
    match crate::util::php_ident_lowercase(declarer.as_ref()).as_str() {
        "simplexmlelement" => {
            xml_with_unsafe_flags(args, 0, "data", 1, "options").map(|_| SinkKind::Xml)
        }
        "splfileobject"
        | "directoryiterator"
        | "filesystemiterator"
        | "recursivedirectoryiterator"
        | "globiterator" => Some(SinkKind::File),
        "soapclient" => Some(SinkKind::Ssrf),
        _ => None,
    }
}

/// Map a `@taint-sink <kind> $param` docblock tag's free-text kind to the
/// issue it should raise. Named kinds reuse the issue the matching built-in
/// sink raises (`classify_sink` / `classify_method_sink`); anything else,
/// including typos, falls back to the generic `TaintedInput`.
pub fn taint_sink_issue(kind: &str) -> mir_issues::IssueKind {
    match kind {
        "llm_prompt" => mir_issues::IssueKind::TaintedLlmPrompt,
        "html" => mir_issues::IssueKind::TaintedHtml,
        "ssrf" => mir_issues::IssueKind::TaintedSsrf,
        "ldap" => mir_issues::IssueKind::TaintedLdap,
        "xpath" => mir_issues::IssueKind::TaintedXpath,
        "callable" => mir_issues::IssueKind::TaintedCallable,
        "xml" => mir_issues::IssueKind::TaintedXml,
        "header" => mir_issues::IssueKind::TaintedHeader,
        "cookie" => mir_issues::IssueKind::TaintedCookie,
        "sql" => mir_issues::IssueKind::TaintedSql,
        "shell" => mir_issues::IssueKind::TaintedShell,
        "file" => mir_issues::IssueKind::TaintedFile,
        "include" => mir_issues::IssueKind::TaintedInclude,
        "eval" => mir_issues::IssueKind::TaintedEval,
        "unserialize" => mir_issues::IssueKind::TaintedUnserialize,
        other => mir_issues::IssueKind::TaintedInput {
            sink: other.to_string(),
        },
    }
}

/// The taint kind a `@taint-sink <kind>` parameter checks for.
pub fn taint_sink_kind(kind: &str) -> Taint {
    match kind {
        "llm_prompt" => Taint::LLM_PROMPT,
        "html" => Taint::HTML,
        "header" => Taint::HEADER,
        "cookie" => Taint::COOKIE,
        "sql" => Taint::SQL,
        "shell" => Taint::SHELL,
        "file" | "include" => Taint::FILE,
        "unserialize" => Taint::UNSERIALIZE,
        "eval" => Taint::EVAL,
        "callable" => Taint::CALLABLE,
        "ssrf" => Taint::SSRF,
        "ldap" => Taint::LDAP,
        "xpath" => Taint::XPATH,
        _ => Taint::OTHER,
    }
}

/// Whether `expr` carries source taint the given sink kind cares about.
/// Parameter labels reaching the sink are recorded in `summary` instead.
#[allow(clippy::too_many_arguments)]
pub fn reaches_sink(
    expr: &Expr,
    sink: Taint,
    issue: &mir_issues::IssueKind,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
    summary: &mut TaintSummary,
) -> bool {
    let sink = if ctx.non_html_response {
        sink.without(Taint::HTML)
    } else {
        sink
    };
    let taint = stringified_taint(expr, ctx, db, file);
    summary.record_param_sinks(taint, sink, issue);
    taint.concrete().intersects(sink)
}

/// The origin a value has, if it is `fopen()` on a literal path naming a file (not an output
/// stream like `php://output`) or a `realpath()` result.
pub fn assigned_value_origin(value: &Expr) -> Option<ValueOrigin> {
    let mut value = unwrap_parens(value);
    while let ExprKind::ErrorSuppress(inner) = &value.kind {
        value = unwrap_parens(inner);
    }
    let ExprKind::FunctionCall(fc) = &value.kind else {
        return None;
    };
    let name = fc.name.kind.name_bare()?;
    if name.eq_ignore_ascii_case("realpath") {
        return Some(ValueOrigin::CanonicalPath);
    }
    let path = fc
        .args
        .first()
        .filter(|a| a.name.is_none() && !a.unpack())
        .and_then(|a| a.value());
    (name.eq_ignore_ascii_case("fopen")
        && matches!(path.map(|p| &p.kind), Some(ExprKind::String(path)) if {
            let path = path.as_str().to_ascii_lowercase();
            !path.contains("php://") && !path.starts_with("/dev/")
        }))
    .then_some(ValueOrigin::FileHandle)
}

/// What a `header()` call makes of the response type: `Some(true)` for a literal content type
/// HTML can't run in, `Some(false)` for any other content type or an unknown header, `None` for
/// another named header.
pub fn header_sets_non_html(args: &[php_ast::owned::Arg]) -> Option<bool> {
    let Some(value) = args.first().filter(|a| !a.unpack()).and_then(|a| a.value()) else {
        return Some(false);
    };
    let mut prefix = value;
    while let ExprKind::Binary(b) = &prefix.kind {
        if b.op != php_ast::ast::BinaryOp::Concat {
            break;
        }
        prefix = &b.left;
    }
    let (text, complete) = match &prefix.kind {
        ExprKind::String(s) => (s.as_str(), std::ptr::eq(prefix, value)),
        ExprKind::InterpolatedString(parts) => match parts.first() {
            Some(php_ast::owned::StringPart::Literal(s)) => (s.as_str(), false),
            _ => return Some(false),
        },
        _ => return Some(false),
    };
    let text = text.to_ascii_lowercase();
    let Some(media) = text.strip_prefix("content-type:") else {
        return (!text.contains(':')).then_some(false);
    };
    let media = match media.split_once(';') {
        Some((media, _)) => media,
        None if complete => media,
        None => return Some(false),
    };
    let media = media.trim();
    Some(
        matches!(
            media,
            "application/json" | "text/plain" | "text/csv" | "application/octet-stream"
        ) || (media.starts_with("application/") && media.ends_with("+json")),
    )
}

// ---------------------------------------------------------------------------
// Mass assignment
// ---------------------------------------------------------------------------

/// Report a write whose target name user input chooses: `$obj->$k = $v`, `$$k = $v`,
/// `$_SESSION[$k] = $v`.
pub fn check_mass_assignment(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    target: &Expr,
    ctx: &FlowState,
    span: php_ast::Span,
) {
    let (name, what) = match &target.kind {
        ExprKind::PropertyAccess(pa) => (&pa.property, "property name"),
        ExprKind::VariableVariable(inner) => (&**inner, "variable name"),
        ExprKind::ArrayAccess(aa) if matches!(&aa.array.kind, ExprKind::Variable(v) if v.trim_start_matches('$') == "_SESSION") => {
            match &aa.index {
                Some(index) => (index, "$_SESSION key"),
                None => return,
            }
        }
        _ => return,
    };
    if has_literal_prefix(name) || !crate::security::is_request_input(name, ctx) {
        return;
    }
    if expr_taint(name, ctx, ea.db, &ea.file)
        .concrete()
        .intersects(Taint::CALLABLE)
    {
        let issue = mir_issues::IssueKind::TaintedMassAssignment {
            target: what.to_string(),
        };
        ea.emit_taint(issue, span, name, Taint::CALLABLE, ctx);
    }
}

/// Report spreading an array whose keys user input chooses into a call, which PHP 8 maps to
/// named parameters: `f(...$_POST)`.
pub fn check_named_arg_spread(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    value: &Expr,
    ctx: &FlowState,
    span: php_ast::Span,
) {
    if crate::db::db_php_version(ea.db) < crate::php_version::PhpVersion::new(8, 0)
        || !crate::security::is_request_input(value, ctx)
        || !may_have_string_keys(value, ctx)
    {
        return;
    }
    let whole = expr_taint(value, ctx, ea.db, &ea.file);
    if iteration_key_taint(value, whole, ctx, ea.db, &ea.file)
        .concrete()
        .intersects(Taint::CALLABLE)
    {
        let issue = mir_issues::IssueKind::TaintedMassAssignment {
            target: "array keys spread into named arguments".to_string(),
        };
        ea.emit_taint(issue, span, value, Taint::CALLABLE, ctx);
    }
}

/// `'prefix_' . $x` or `"prefix_$x"`: a name confined to its literal prefix.
fn has_literal_prefix(expr: &Expr) -> bool {
    let mut first = unwrap_parens(expr);
    while let ExprKind::Binary(b) = &first.kind {
        if b.op != php_ast::ast::BinaryOp::Concat {
            break;
        }
        first = unwrap_parens(&b.left);
    }
    match &first.kind {
        ExprKind::String(s) => !std::ptr::eq(first, unwrap_parens(expr)) && !s.is_empty(),
        ExprKind::InterpolatedString(_) => matches!(
            crate::ast_ext::StringPartsExt::string_parts(&first.kind).first(),
            Some(StringPart::Literal(s)) if !s.is_empty()
        ),
        _ => false,
    }
}

fn may_have_string_keys(value: &Expr, ctx: &FlowState) -> bool {
    use mir_types::Atomic;
    let ExprKind::Variable(name) = &unwrap_parens(value).kind else {
        return false;
    };
    ctx.get_var(name).types.iter().any(|atom| match atom {
        Atomic::TArray { key, .. } | Atomic::TNonEmptyArray { key, .. } => {
            !key.types.iter().all(|k| k.is_int())
        }
        Atomic::TKeyedArray { is_list, .. } => !is_list,
        Atomic::TMixed => true,
        _ => false,
    })
}

// ---------------------------------------------------------------------------
// Argument injection
// ---------------------------------------------------------------------------

/// Classes whose constructor takes a program and its arguments as an array: Symfony's `Process`.
pub fn runs_program_directly(db: &dyn crate::db::MirDatabase, fqcn: &str) -> bool {
    let base = "Symfony\\Component\\Process\\Process";
    fqcn.eq_ignore_ascii_case(base) || crate::db::extends_or_implements(db, fqcn, base)
}

/// Report a tainted argument of a program run without a shell (`proc_open(['git', 'clone',
/// $url])`) that can start with `-` and so pass an option, unless a `--` comes before it.
pub fn check_command_arguments(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    command: &Expr,
    ctx: &FlowState,
    span: php_ast::Span,
) {
    let ExprKind::Array(elements) = &unwrap_parens(command).kind else {
        return;
    };
    for element in elements.iter().skip(1) {
        if element.is_spread() || element.key.is_some() {
            return;
        }
        let value = element.value();
        if matches!(&value.kind, ExprKind::String(s) if s.as_str() == "--") {
            return;
        }
        let dash_checked = matches!(&value.kind, ExprKind::Variable(v)
            if ctx.has_value_origin(v, ValueOrigin::NoLeadingDash));
        if has_literal_prefix(value) || dash_checked {
            continue;
        }
        if expr_taint(value, ctx, ea.db, &ea.file)
            .concrete()
            .intersects(Taint::SHELL)
        {
            let issue = mir_issues::IssueKind::TaintedCommandArgument;
            ea.emit_taint(issue, span, value, Taint::SHELL, ctx);
            return;
        }
    }
}

/// Record that a variable can't start with `-` where `cond` evaluating to `holds` proves it:
/// `!str_starts_with($x, '-')`, `$x[0] !== '-'`, `str_starts_with($x, 'https://')`.
pub fn mark_dash_checked(cond: &Expr, holds: bool, ctx: &mut FlowState) {
    let is_dash = |e: &Expr| matches!(&e.kind, ExprKind::String(s) if s.as_str() == "-");
    // The variable, and whether `cond` holding means it starts with `-`.
    let checked: Option<(&str, bool)> = match &unwrap_parens(cond).kind {
        ExprKind::FunctionCall(fc)
            if fc
                .name
                .kind
                .name_bare()
                .is_some_and(|n| n.eq_ignore_ascii_case("str_starts_with")) =>
        {
            match (
                positional_value(&fc.args, 0).map(|v| &v.kind),
                positional_value(&fc.args, 1),
            ) {
                (Some(ExprKind::Variable(v)), Some(prefix)) if is_dash(prefix) => {
                    Some((&**v, true))
                }
                (Some(ExprKind::Variable(v)), Some(prefix)) => match &prefix.kind {
                    ExprKind::String(s) if !s.is_empty() && !s.as_str().starts_with('-') => {
                        holds.then_some((&**v, false))
                    }
                    _ => None,
                },
                _ => None,
            }
        }
        ExprKind::Binary(b) => {
            let equal = match b.op {
                php_ast::ast::BinaryOp::Identical | php_ast::ast::BinaryOp::Equal => true,
                php_ast::ast::BinaryOp::NotIdentical | php_ast::ast::BinaryOp::NotEqual => false,
                _ => return,
            };
            let (subject, literal) = (unwrap_parens(&b.left), unwrap_parens(&b.right));
            let (subject, literal) =
                if matches!(subject.kind, ExprKind::String(_) | ExprKind::Int(_)) {
                    (literal, subject)
                } else {
                    (subject, literal)
                };
            first_char_subject(subject, literal).map(|v| (v, equal))
        }
        _ => None,
    };
    if let Some((var, means_dash)) = checked {
        if holds != means_dash {
            ctx.mark_value_origin(var, ValueOrigin::NoLeadingDash);
        }
    }
}

fn positional_value(args: &[php_ast::owned::Arg], index: usize) -> Option<&Expr> {
    args.get(index)
        .filter(|a| a.name.is_none() && !a.unpack())
        .and_then(|a| a.value())
}

/// The variable whose first character `subject` compared with `literal` tests for `-`:
/// `$x[0] === '-'`, `substr($x, 0, 1) === '-'`, `strpos($x, '-') === 0`.
fn first_char_subject<'e>(subject: &'e Expr, literal: &Expr) -> Option<&'e str> {
    let var = |e: &'e Expr| match &e.kind {
        ExprKind::Variable(v) => Some(&**v),
        _ => None,
    };
    let int =
        |e: Option<&Expr>, n: i64| matches!(e.map(|e| &e.kind), Some(ExprKind::Int(i)) if *i == n);
    let dash = |e: Option<&Expr>| matches!(e.map(|e| &e.kind), Some(ExprKind::String(s)) if s.as_str() == "-");
    match &subject.kind {
        ExprKind::ArrayAccess(aa) if int(aa.index.as_ref(), 0) && dash(Some(literal)) => {
            var(&aa.array)
        }
        ExprKind::FunctionCall(fc) => {
            let name = fc.name.kind.name_bare()?.to_ascii_lowercase();
            let arg = |i: usize| fc.args.get(i).and_then(|a| a.value());
            match name.as_str() {
                "substr" if int(arg(1), 0) && int(arg(2), 1) && dash(Some(literal)) => var(arg(0)?),
                "strpos" if dash(arg(1)) && int(Some(literal), 0) => var(arg(0)?),
                _ => None,
            }
        }
        _ => None,
    }
}
