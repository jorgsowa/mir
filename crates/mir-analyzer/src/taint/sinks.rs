use super::*;

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
    let annotated: Vec<ParamSink> = sink_params
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
        .collect();
    let mut callee = TaintSummary::default();
    if !demanding_taint(args_taint(args, ctx, ea.db, &ea.file), ctx).is_empty()
        || receiver_demands_summary(receiver, ctx)
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
            let (Some(receiver), taint) = (receiver, receiver_prop_taint(receiver, prop, ctx))
            else {
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
        let guarded = labels.is_object_guard(sink.param);
        for value in labels.slot_args(sink.param, params, args, arg_types, ctx, ea.db, &ea.file) {
            let taint = if guarded {
                object_arg_taint(value, ctx, ea.db, &ea.file)
            } else {
                expr_taint(value, ctx, ea.db, &ea.file)
            };
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
            SinkKind::File => IssueKind::TaintedInput {
                sink: "file".to_string(),
            },
            SinkKind::Unserialize => IssueKind::TaintedInput {
                sink: "unserialize".to_string(),
            },
            SinkKind::Ssrf => IssueKind::TaintedSsrf,
            SinkKind::Ldap => IssueKind::TaintedLdap,
            SinkKind::Xpath => IssueKind::TaintedXpath,
            SinkKind::Callable => IssueKind::TaintedCallable,
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
        "echo" | "print" | "printf" | "vprintf" | "fprintf" | "vfprintf" => sinks![any(Html)],

        "header" => sinks![at(0, "header", Header)],
        "setcookie" | "setrawcookie" => sinks![any(Cookie)],
        "mail" => sinks![
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
        | "readgzfile" => sinks![at(0, "filename", File)],
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

        "ldap_search" | "ldap_list" | "ldap_read" => {
            sinks![at(1, "base", Ldap), at(2, "filter", Ldap)]
        }
        "ldap_delete" | "ldap_compare" => sinks![at(1, "dn", Ldap)],

        "unserialize" => sinks![at(0, "data", Unserialize)],

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
    let is_target_option = |e: &Expr| {
        matches!(&e.kind, ExprKind::Identifier(name)
            if matches!(name.trim_start_matches('\\'), "CURLOPT_URL" | "CURLOPT_PROXY"))
    };
    match fn_name_lower.trim_start_matches('\\') {
        "curl_setopt" => match (args.get(1).and_then(|a| a.value.as_ref()), args.get(2)) {
            (Some(option), Some(value)) if is_target_option(option) => {
                value.value.iter().map(|v| (v, SinkKind::Ssrf)).collect()
            }
            _ => Vec::new(),
        },
        "curl_setopt_array" => match args.get(1).and_then(|a| a.value.as_ref()).map(|v| &v.kind) {
            Some(ExprKind::Array(elements)) => elements
                .iter()
                .filter(|el| el.key.as_ref().is_some_and(is_target_option))
                .map(|el| (&el.value, SinkKind::Ssrf))
                .collect(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    }
}

/// `unserialize($data, ['allowed_classes' => false])` can't instantiate objects.
pub fn unserialize_disallows_classes(args: &[php_ast::owned::Arg]) -> bool {
    let Some(ExprKind::Array(options)) =
        args.get(1).and_then(|a| a.value.as_ref()).map(|v| &v.kind)
    else {
        return false;
    };
    options.iter().any(|el| {
        matches!(el.key.as_ref().map(|k| &k.kind), Some(ExprKind::String(k)) if &**k == "allowed_classes")
            && matches!(&el.value.kind, ExprKind::Bool(false))
    })
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
        "query" | "exec" | "prepare" | "real_query" | "multi_query" | "evaluate" | "xpath"
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
            "query" | "prepare" | "real_query" | "multi_query"
        )
    {
        return Some(SinkKind::Sql);
    }
    if is_a("SQLite3") && matches!(method.as_str(), "query" | "exec" | "prepare") {
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
pub fn classify_constructor_sink(db: &dyn crate::db::MirDatabase, fqcn: &str) -> Option<SinkKind> {
    let is_a = |base: &str| {
        fqcn.eq_ignore_ascii_case(base) || crate::db::extends_or_implements(db, fqcn, base)
    };
    if ["SplFileObject", "DirectoryIterator", "GlobIterator"]
        .iter()
        .any(|base| is_a(base))
    {
        return Some(SinkKind::File);
    }
    is_a("SoapClient").then_some(SinkKind::Ssrf)
}

/// Map a `@taint-sink <kind> $param` docblock tag's free-text kind to the
/// issue it should raise. Named kinds reuse the issue the matching built-in
/// sink raises (`classify_sink` / `classify_method_sink`); anything else,
/// including typos, falls back to the generic `TaintedInput`, as
/// `classify_sink`'s File/Unserialize arms do.
pub fn taint_sink_issue(kind: &str) -> mir_issues::IssueKind {
    match kind {
        "llm_prompt" => mir_issues::IssueKind::TaintedLlmPrompt,
        "html" => mir_issues::IssueKind::TaintedHtml,
        "ssrf" => mir_issues::IssueKind::TaintedSsrf,
        "ldap" => mir_issues::IssueKind::TaintedLdap,
        "xpath" => mir_issues::IssueKind::TaintedXpath,
        "callable" => mir_issues::IssueKind::TaintedCallable,
        "header" => mir_issues::IssueKind::TaintedHeader,
        "cookie" => mir_issues::IssueKind::TaintedCookie,
        "sql" => mir_issues::IssueKind::TaintedSql,
        "shell" => mir_issues::IssueKind::TaintedShell,
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
    let taint = expr_taint(expr, ctx, db, file);
    summary.record_param_sinks(taint, sink, issue);
    taint.concrete().intersects(sink)
}
