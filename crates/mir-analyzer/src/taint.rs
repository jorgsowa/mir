/// Taint analysis helpers.
///
/// A value is "tainted" when it originates from user-controlled input
/// (superglobals, request headers, `@taint-source` calls). Taint is tracked
/// per [`Taint`] kind and propagates through assignments, string operations,
/// array access and pass-through builtins; a sanitizer removes only the kinds
/// it neutralizes.
///
/// When data tainted for a sink's kind reaches it, the matching `Tainted*`
/// issue is emitted.
use php_ast::owned::{Expr, ExprKind, StringPart};

use crate::flow_state::FlowState;

/// Set of sink kinds a value is still dangerous for.
///
/// The low [`KIND_BITS`] hold taint from real sources. While a function body
/// is analyzed for its [`TaintSummary`], each of the first
/// [`MAX_TRACKED_PARAMS`] parameters gets its own slot of the same width, so a
/// sanitizer applied to a parameter is remembered per parameter.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Taint(u128);

const KIND_BITS: u32 = 12;
const KIND_MASK: u128 = (1 << KIND_BITS) - 1;
pub const MAX_TRACKED_PARAMS: usize = 9;

/// `kinds` copied into the source slot and every parameter slot.
const fn replicate(kinds: u128) -> u128 {
    let kinds = kinds & KIND_MASK;
    let mut out = 0;
    let mut slot = 0;
    while slot <= MAX_TRACKED_PARAMS as u32 {
        out |= kinds << (slot * KIND_BITS);
        slot += 1;
    }
    out
}

impl Taint {
    pub const NONE: Taint = Taint(0);
    pub const HTML: Taint = Taint(1 << 0);
    pub const SQL: Taint = Taint(1 << 1);
    pub const SHELL: Taint = Taint(1 << 2);
    pub const HEADER: Taint = Taint(1 << 3);
    pub const COOKIE: Taint = Taint(1 << 4);
    pub const FILE: Taint = Taint(1 << 5);
    pub const UNSERIALIZE: Taint = Taint(1 << 6);
    pub const EVAL: Taint = Taint(1 << 7);
    pub const LLM_PROMPT: Taint = Taint(1 << 8);
    pub const LDAP: Taint = Taint(1 << 9);
    pub const XPATH: Taint = Taint(1 << 10);
    /// Any `@taint-sink` kind without a dedicated bit.
    pub const OTHER: Taint = Taint(1 << 11);
    pub const ALL: Taint = Taint(KIND_MASK);

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn intersects(self, other: Taint) -> bool {
        self.0 & other.0 != 0
    }

    /// Remove `kinds` from the source slot and every parameter slot.
    pub const fn without(self, kinds: Taint) -> Taint {
        Taint(self.0 & !replicate(kinds.0))
    }

    /// Keep only `kinds` in every slot.
    pub const fn restrict_to(self, kinds: Taint) -> Taint {
        Taint(self.0 & replicate(kinds.0))
    }

    /// Add `kinds` to every slot that is already tainted.
    pub fn restore(self, kinds: Taint) -> Taint {
        let mut out = self.0;
        for slot in 0..=MAX_TRACKED_PARAMS as u32 {
            if (self.0 >> (slot * KIND_BITS)) & KIND_MASK != 0 {
                out |= (kinds.0 & KIND_MASK) << (slot * KIND_BITS);
            }
        }
        Taint(out)
    }

    /// Taint from real sources only.
    pub const fn concrete(self) -> Taint {
        Taint(self.0 & KIND_MASK)
    }

    /// The label seeded into parameter `index` for summary analysis.
    pub const fn param(index: usize) -> Taint {
        Taint(KIND_MASK << ((index as u32 + 1) * KIND_BITS))
    }

    /// Kinds of parameter `index`'s label this value still carries.
    pub const fn param_kinds(self, index: usize) -> Taint {
        Taint((self.0 >> ((index as u32 + 1) * KIND_BITS)) & KIND_MASK)
    }
}

impl std::ops::BitOr for Taint {
    type Output = Taint;
    fn bitor(self, rhs: Taint) -> Taint {
        Taint(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Taint {
    fn bitor_assign(&mut self, rhs: Taint) {
        self.0 |= rhs.0;
    }
}

/// A parameter whose value reaches a sink inside the callee.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamSink {
    pub param: usize,
    pub kinds: Taint,
    pub issue: mir_issues::IssueKind,
}

/// How a function or method moves taint, seen from its call sites.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TaintSummary {
    /// Taint of the return value: the source slot is taint the body reads
    /// itself, each parameter slot the kinds of that argument that flow out.
    pub returns: Taint,
    pub param_sinks: Vec<ParamSink>,
}

impl TaintSummary {
    pub fn is_empty(&self) -> bool {
        self.returns.is_empty() && self.param_sinks.is_empty()
    }

    /// Record that `taint`, reaching a sink of `kinds`, carries parameter labels.
    pub fn record_param_sinks(
        &mut self,
        taint: Taint,
        kinds: Taint,
        issue: &mir_issues::IssueKind,
    ) {
        for param in 0..MAX_TRACKED_PARAMS {
            let reached = taint.param_kinds(param).restrict_to(kinds);
            if reached.is_empty() {
                continue;
            }
            match self
                .param_sinks
                .iter_mut()
                .find(|s| s.param == param && s.issue == *issue)
            {
                Some(existing) => existing.kinds |= reached,
                None => self.param_sinks.push(ParamSink {
                    param,
                    kinds: reached,
                    issue: issue.clone(),
                }),
            }
        }
    }

    /// Taint of a call's result, given the taint of the argument bound to each parameter.
    pub fn call_result(&self, arg_taint: impl Fn(usize) -> Taint) -> Taint {
        let mut out = self.returns.concrete();
        for param in 0..MAX_TRACKED_PARAMS {
            let kinds = self.returns.param_kinds(param);
            if !kinds.is_empty() {
                out |= arg_taint(param).restrict_to(kinds);
            }
        }
        out
    }
}

/// Take the body's summary; a generator's `return` value isn't the call result.
pub fn finish_summary(summary: &mut TaintSummary, is_generator: bool) -> TaintSummary {
    let mut summary = std::mem::take(summary);
    if is_generator {
        summary.returns = Taint::NONE;
    }
    summary
}

/// Label each parameter so the body's summary can tell which arguments flow where.
/// Only inference builds summaries; a full analysis reports source taint alone.
pub fn seed_param_labels(
    ctx: &mut FlowState,
    params: &[mir_codebase::DeclaredParam],
    mode: crate::body_analysis::AnalysisMode,
) {
    if mode != crate::body_analysis::AnalysisMode::InferenceOnly {
        return;
    }
    for (index, param) in params.iter().take(MAX_TRACKED_PARAMS).enumerate() {
        ctx.set_var_taint(param.name.as_ref(), Taint::param(index));
    }
}

/// Expressions passed for declared parameter `index`, including a spread that may cover it.
pub fn param_arg_exprs<'e>(
    params: &[mir_codebase::DeclaredParam],
    args: &'e [php_ast::owned::Arg],
    index: usize,
) -> Vec<&'e Expr> {
    let Some(param) = params.get(index) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut position = 0;
    for arg in args {
        let Some(value) = &arg.value else {
            position += 1;
            continue;
        };
        if let Some(name) = &arg.name {
            let name = crate::parser::name_to_string_owned(name);
            let unknown_name = || !params.iter().any(|p| p.name.as_ref() == name);
            if name == param.name.as_ref() || (param.is_variadic && unknown_name()) {
                out.push(value);
            }
        } else if arg.unpack {
            if position <= index || param.is_variadic {
                out.push(value);
            }
        } else if position == index || (param.is_variadic && position >= index) {
            out.push(value);
        }
        position += 1;
    }
    out
}

/// Taint of the arguments bound to parameter `index`.
pub fn param_arg_taint(
    params: &[mir_codebase::DeclaredParam],
    args: &[php_ast::owned::Arg],
    index: usize,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    param_arg_exprs(params, args, index)
        .into_iter()
        .fold(Taint::NONE, |acc, e| acc | expr_taint(e, ctx, db, file))
}

/// Union of the kinds named by taint annotations (`html`, `sql`, `input`, ...).
fn named_kinds(names: &[std::sync::Arc<str>]) -> Taint {
    names.iter().fold(Taint::NONE, |acc, name| {
        acc | match name.as_ref() {
            "input" | "text" => Taint::ALL,
            "has_quotes" => Taint::HTML | Taint::SQL,
            other => taint_sink_kind(other),
        }
    })
}

/// A return type whose values can't carry a payload into a sink.
fn return_drops_taint(ty: &mir_types::Type) -> bool {
    use mir_types::Atomic;
    ty.is_void()
        || ty.is_never()
        || is_injection_safe_type(ty)
        || ty.types.iter().all(|a| {
            matches!(
                a,
                Atomic::TNamedObject { .. }
                    | Atomic::TStaticObject { .. }
                    | Atomic::TSelf { .. }
                    | Atomic::TParent { .. }
                    | Atomic::TClosure { .. }
                    | Atomic::TObject
            )
        })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CalleeBody {
    /// Abstract, interface or stub: only annotations describe it.
    None,
    Dependency,
    Summarized,
}

impl CalleeBody {
    fn of(location: Option<&mir_types::Location>, from: &str) -> CalleeBody {
        match location {
            Some(loc) if crate::stubs::is_stub_path(loc.file.as_ref()) => CalleeBody::None,
            Some(loc) if is_dependency_of(loc.file.as_ref(), from) => CalleeBody::Dependency,
            Some(_) => CalleeBody::Summarized,
            None => CalleeBody::None,
        }
    }
}

/// Taint of a call to user code, from the callee's annotations and inferred summary.
#[allow(clippy::too_many_arguments)]
fn user_call_taint(
    params: &[mir_codebase::DeclaredParam],
    args: &[php_ast::owned::Arg],
    return_type: Option<&mir_types::Type>,
    annotations: Option<&mir_codebase::TaintAnnotations>,
    body: CalleeBody,
    reads_source: impl FnOnce() -> bool,
    summary: impl FnOnce() -> Option<TaintSummary>,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    if return_type.is_some_and(return_drops_taint) {
        return Taint::NONE;
    }
    let arg_taint = |index| param_arg_taint(params, args, index, ctx, db, file);
    // Inferring a summary only pays off when it can return source data or tainted arguments.
    let worth_summary = || !args_taint(args, ctx, db, file).is_empty() || reads_source();
    let mut taint = match body {
        CalleeBody::Summarized if worth_summary() => {
            summary().map_or(Taint::NONE, |s| s.call_result(arg_taint))
        }
        CalleeBody::Dependency => args_taint(args, ctx, db, file),
        _ => Taint::NONE,
    };
    let Some(annotations) = annotations else {
        return taint;
    };
    for name in &annotations.flow_params {
        if let Some(index) = params.iter().position(|p| p.name.as_ref() == name.as_ref()) {
            taint |= arg_taint(index);
        }
    }
    let escaped = named_kinds(&annotations.escape);
    let unescaped = named_kinds(&annotations.unescape);
    // An annotated escaper or decoder without a body to summarize passes its arguments through.
    if body == CalleeBody::None && annotations.flow_params.is_empty() {
        taint = args_taint(args, ctx, db, file);
    }
    taint.without(escaped).restore(unescaped)
}

/// Report arguments reaching a sink inside the callee: its `@taint-sink`
/// parameters and the sinks its inferred summary records.
#[allow(clippy::too_many_arguments)]
pub fn check_call_sinks(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    ctx: &FlowState,
    params: &[mir_codebase::DeclaredParam],
    args: &[php_ast::owned::Arg],
    sink_params: &[(std::sync::Arc<str>, std::sync::Arc<str>)],
    summary: impl FnOnce(&dyn crate::db::MirDatabase, &str) -> Option<TaintSummary>,
    span: php_ast::Span,
) {
    let mut sinks: Vec<ParamSink> = sink_params
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
    if !args_taint(args, ctx, ea.db, &ea.file).is_empty() {
        if let Some(summary) = summary(ea.db, &ea.file) {
            sinks.extend(summary.param_sinks);
        }
    }
    let mut emitted: Vec<mir_issues::IssueKind> = Vec::new();
    for sink in &sinks {
        for value in param_arg_exprs(params, args, sink.param) {
            let taint = expr_taint(value, ctx, ea.db, &ea.file);
            ea.taint_summary
                .record_param_sinks(taint, sink.kinds, &sink.issue);
            if taint.concrete().intersects(sink.kinds) && !emitted.contains(&sink.issue) {
                emitted.push(sink.issue.clone());
                ea.emit(sink.issue.clone(), mir_issues::Severity::Error, span);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Superglobal names (without the $ prefix, as stored in FlowState::vars)
// ---------------------------------------------------------------------------

/// Superglobals holding request data; `$_ENV` comes from the process environment.
pub const SUPERGLOBALS: &[&str] = &["_GET", "_POST", "_REQUEST", "_COOKIE", "_FILES", "_SERVER"];

pub fn is_superglobal(name: &str) -> bool {
    SUPERGLOBALS.contains(&name)
}

/// `$_SERVER` keys a client controls; the rest come from the server or environment.
fn is_request_server_key(key: &str) -> bool {
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

fn is_superglobal_var(expr: &Expr, name: &str) -> bool {
    matches!(&expr.kind, ExprKind::Variable(v) if v.trim_start_matches('$') == name)
}

/// A superglobal entry PHP, the server or the environment sets rather than the
/// client: `$_SERVER['DOCUMENT_ROOT']`, `$_FILES[x]['tmp_name']`.
pub fn is_trusted_superglobal_field(aa: &php_ast::owned::ArrayAccessExpr) -> bool {
    let Some(ExprKind::String(key)) = aa.index.as_deref().map(|i| &i.kind) else {
        return false;
    };
    if is_superglobal_var(&aa.array, "_SERVER") {
        return !is_request_server_key(key);
    }
    let ExprKind::ArrayAccess(file_entry) = &aa.array.kind else {
        return false;
    };
    matches!(&**key, "tmp_name" | "size" | "error")
        && is_superglobal_var(&file_entry.array, "_FILES")
}

/// `$GLOBALS['_GET']` reads the superglobal itself.
pub fn is_globals_superglobal_alias(aa: &php_ast::owned::ArrayAccessExpr) -> bool {
    matches!(aa.index.as_deref().map(|i| &i.kind), Some(ExprKind::String(key)) if is_superglobal(key))
        && is_superglobal_var(&aa.array, "GLOBALS")
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
    Header,      // header → response splitting / open redirect
    Cookie,      // setcookie / setrawcookie → cookie injection
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
        }
    }

    /// The argument indices whose taint should be reported for this sink.
    ///
    /// `None` means "any argument" — used for output/query/command sinks where
    /// the dangerous payload may be in any position (e.g. `mysqli_query($db,
    /// $sql)` carries the query in arg 1). The path/payload sinks below name
    /// the single relevant argument so that, e.g., writing tainted *data* to a
    /// constant path is not flagged — only a tainted *path* is.
    pub fn tainted_arg_indices(self) -> Option<&'static [usize]> {
        match self {
            SinkKind::Html
            | SinkKind::Sql
            | SinkKind::Shell
            | SinkKind::Header
            | SinkKind::Cookie => None,
            // Path is the first argument for every File sink listed below.
            SinkKind::File => Some(&[0]),
            // unserialize($data) — the payload is the first argument.
            SinkKind::Unserialize => Some(&[0]),
        }
    }
}

/// The declared parameter NAME at a File/Unserialize sink's single relevant
/// index (see `SinkKind::tainted_arg_indices`) — lets the caller resolve a
/// PHP 8 named argument to the same slot a positional call would use, so
/// `file_put_contents(data: 'safe', filename: $_GET['path'])` still flags
/// the actually-tainted `filename` even though it's not in position 0.
/// `None` for Html/Sql/Shell, which check "any argument" and don't need
/// per-parameter name resolution.
pub fn sink_param_name(fn_name_lower: &str) -> Option<&'static str> {
    match fn_name_lower {
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
        | "readgzfile" => Some("filename"),
        "mkdir" | "rmdir" | "scandir" | "opendir" | "chdir" => Some("directory"),
        "copy" | "rename" => Some("from"),
        "symlink" | "link" => Some("target"),
        "readlink" => Some("path"),
        "glob" => Some("pattern"),
        "fsockopen" => Some("hostname"),
        "get_headers" | "curl_init" => Some("url"),
        "stream_socket_client" => Some("address"),
        "dl" => Some("extension_filename"),
        "move_uploaded_file" => Some("to"),
        "unserialize" => Some("data"),
        _ => None,
    }
}

/// Positional-call override for a File sink whose dangerous argument isn't at
/// `SinkKind::tainted_arg_indices`'s shared index 0 — `move_uploaded_file($from,
/// $to)` writes to `$to` (index 1), unlike every other File sink listed there,
/// whose sole argument (or first argument) IS the path. `None` means "use the
/// shared index," which is every other File/Unserialize sink today.
pub fn sink_positional_index_override(fn_name_lower: &str) -> Option<usize> {
    match fn_name_lower {
        "move_uploaded_file" => Some(1),
        _ => None,
    }
}

/// Return the sink kind for a built-in function name, if it is a taint sink.
pub fn classify_sink(fn_name: &str) -> Option<SinkKind> {
    match crate::util::php_ident_lowercase(fn_name).as_str() {
        // HTML output
        "echo" | "print" | "printf" | "vprintf" | "fprintf" | "vfprintf" => Some(SinkKind::Html),

        // HTTP response header / cookie
        "header" => Some(SinkKind::Header),
        "setcookie" | "setrawcookie" => Some(SinkKind::Cookie),

        // SQL
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
        | "db2_prepare" => Some(SinkKind::Sql),

        // Shell
        "system" | "exec" | "shell_exec" | "passthru" | "popen" | "proc_open" | "pcntl_exec"
        | "ssh2_exec" | "expect_popen" => Some(SinkKind::Shell),

        // Filesystem path (path traversal / local-file-inclusion / SSRF). Each
        // of these takes the path/URL as its first argument, except
        // `move_uploaded_file` (see `sink_positional_index_override`).
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
        | "mkdir"
        | "rmdir"
        | "scandir"
        | "opendir"
        | "chdir"
        | "copy"
        | "rename"
        | "symlink"
        | "link"
        | "readlink"
        | "glob"
        | "fsockopen"
        | "get_headers"
        | "curl_init"
        | "stream_socket_client"
        | "dl"
        | "move_uploaded_file" => Some(SinkKind::File),

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
    None
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
        "eval" | "callable" => Taint::EVAL,
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

// ---------------------------------------------------------------------------
// Expression taint checker
// ---------------------------------------------------------------------------

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

/// Strip any number of redundant parens (`((expr))` -> `expr`) — a call
/// target keeps its parens in the AST (`(fn() => ...)()`), unlike most
/// other expression positions.
fn unwrap_parens(expr: &Expr) -> &Expr {
    let mut e = expr;
    while let ExprKind::Parenthesized(inner) = &e.kind {
        e = inner;
    }
    e
}

/// Union of the taint every argument of a call carries.
pub fn args_taint(
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    args.iter()
        .filter_map(|a| a.value.as_ref())
        .fold(Taint::NONE, |acc, v| acc | expr_taint(v, ctx, db, file))
}

/// The kinds of attacker-controlled data the expression could carry, given the
/// current `FlowState` taint state.
///
/// This is a conservative over-approximation:
/// - Any reference to a superglobal (or an array offset thereof) is tainted.
/// - Any variable that was previously marked tainted in `ctx.tainted_vars` is tainted.
/// - Binary string-concat or arithmetic on tainted operands propagates taint.
/// - Interpolated strings are tainted if any embedded variable is tainted.
/// - A call to a `@taint-source`-annotated function/method is tainted.
/// - A sanitizer (`htmlspecialchars`, `escapeshellarg`, …) removes only its own kinds.
pub fn expr_taint(
    expr: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    match &expr.kind {
        ExprKind::Variable(name) => {
            let n = name.trim_start_matches('$');
            if is_superglobal(n) {
                return Taint::ALL;
            }
            let sym = mir_types::Name::from(n);
            if ctx
                .vars
                .get(&sym)
                .is_some_and(|ty| is_injection_safe_type(ty))
            {
                return Taint::NONE;
            }
            // A variable with no tracked definition may be one `extract()`
            // defined from a tainted array; its name isn't known statically.
            if ctx.has_dynamic_tainted_var_def && !ctx.var_is_defined_sym(sym) {
                return Taint::ALL;
            }
            ctx.var_taint(n)
        }

        // `$$name` where `$name` holds a known literal string resolves to
        // that variable's own taint state.
        ExprKind::VariableVariable(inner) => {
            let Some(var_name) = crate::expr::helpers::extract_simple_var(inner) else {
                return Taint::NONE;
            };
            ctx.get_var(&var_name)
                .types
                .iter()
                .fold(Taint::NONE, |acc, atomic| match atomic {
                    mir_types::Atomic::TLiteralString(accessed) => {
                        acc | ctx.var_taint(accessed.as_ref())
                    }
                    _ => acc,
                })
        }

        ExprKind::ArrayAccess(aa) => {
            if is_trusted_superglobal_field(aa) {
                Taint::NONE
            } else if is_globals_superglobal_alias(aa) {
                Taint::ALL
            } else {
                expr_taint(&aa.array, ctx, db, file)
            }
        }

        // The error-suppression operator only silences a notice, it doesn't
        // sanitize the value.
        ExprKind::Parenthesized(inner) | ExprKind::ErrorSuppress(inner) => {
            expr_taint(inner, ctx, db, file)
        }

        // Only a simple-variable receiver is tracked, matching prop_refined's
        // narrowing scope.
        ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa) => {
            if let ExprKind::Variable(obj_var) = &pa.object.kind {
                if let Some(prop_name) =
                    crate::expr::helpers::extract_string_from_expr(&pa.property)
                {
                    return ctx.prop_taint(obj_var.trim_start_matches('$'), &prop_name);
                }
            }
            Taint::NONE
        }

        ExprKind::Assign(a) => expr_taint(&a.value, ctx, db, file),

        ExprKind::Binary(op) => {
            expr_taint(&op.left, ctx, db, file) | expr_taint(&op.right, ctx, db, file)
        }

        ExprKind::UnaryPrefix(u) => expr_taint(&u.operand, ctx, db, file),

        ExprKind::InterpolatedString(parts)
        | ExprKind::Heredoc { parts, .. }
        | ExprKind::ShellExec(parts) => parts.iter().fold(Taint::NONE, |acc, p| match p {
            StringPart::Expr(e) => acc | expr_taint(e, ctx, db, file),
            StringPart::Literal(_) => acc,
        }),

        // Short ternary (`$x ?: $y`): the true branch's value is the condition itself.
        ExprKind::Ternary(t) => {
            let then_taint = match &t.then_expr {
                Some(then_e) => expr_taint(then_e, ctx, db, file),
                None => expr_taint(&t.condition, ctx, db, file),
            };
            then_taint | expr_taint(&t.else_expr, ctx, db, file)
        }

        ExprKind::NullCoalesce(nc) => {
            expr_taint(&nc.left, ctx, db, file) | expr_taint(&nc.right, ctx, db, file)
        }

        // Numeric/boolean casts sanitize; string/array/object casts keep the payload.
        ExprKind::Cast(kind, inner) => match kind {
            php_ast::ast::CastKind::Int
            | php_ast::ast::CastKind::Float
            | php_ast::ast::CastKind::Bool => Taint::NONE,
            _ => expr_taint(inner, ctx, db, file),
        },

        ExprKind::Match(m) => m.arms.iter().fold(Taint::NONE, |acc, arm| {
            acc | expr_taint(&arm.body, ctx, db, file)
        }),

        // An element's key can carry taint just as much as its value.
        ExprKind::Array(elements) => elements.iter().fold(Taint::NONE, |acc, el| {
            let key = el
                .key
                .as_ref()
                .map_or(Taint::NONE, |k| expr_taint(k, ctx, db, file));
            acc | key | expr_taint(&el.value, ctx, db, file)
        }),

        ExprKind::FunctionCall(fc) => {
            if let ExprKind::Identifier(name) = &fc.name.kind {
                let lower_name =
                    crate::util::php_ident_lowercase(name.as_ref().trim_start_matches('\\'));
                if matches!(
                    lower_name.as_str(),
                    "getallheaders" | "apache_request_headers"
                ) {
                    return Taint::ALL;
                }
                if reads_raw_request_body(&lower_name, &fc.args) {
                    return Taint::ALL;
                }
                if let Some(taint) = filter_call_taint(&lower_name, &fc.args, ctx, db, file) {
                    return taint;
                }
                if let Some(removed) = sanitized_kinds(&lower_name) {
                    return args_taint(&fc.args, ctx, db, file).without(removed);
                }
                if let Some(restored) = unescaped_kinds(&lower_name) {
                    return args_taint(&fc.args, ctx, db, file).restore(restored);
                }
                // `compact('id')` copies `$id`'s current value into the
                // returned array; only literal names are resolved, matching
                // `compact_return_type`.
                if lower_name == "compact" {
                    return fc.args.iter().filter_map(|a| a.value.as_ref()).fold(
                        Taint::NONE,
                        |acc, v| match &v.kind {
                            ExprKind::String(var_name) => acc | ctx.var_taint(var_name.as_ref()),
                            _ => acc,
                        },
                    );
                }
                let mut resolved = crate::db::resolve_name(db, file, name.as_ref());
                let mut func = crate::db::find_function_from(
                    db,
                    file,
                    crate::db::Fqcn::from_str(db, &resolved),
                );
                // An unqualified call falls back to the global function.
                if func.is_none() && !name.contains('\\') && resolved.contains('\\') {
                    resolved = name.to_string();
                    func = crate::db::find_function_from(
                        db,
                        file,
                        crate::db::Fqcn::from_str(db, &resolved),
                    );
                }
                if func.as_ref().is_some_and(|f| f.is_taint_source) {
                    return Taint::ALL;
                }
                if let Some(f) = func.as_ref().filter(|f| is_user_code(f.location.as_ref())) {
                    return user_call_taint(
                        &f.params,
                        &fc.args,
                        f.return_type.as_deref(),
                        f.taint.as_deref(),
                        CalleeBody::of(f.location.as_ref(), file),
                        || {
                            crate::db::taint_sources::function_reads_source(
                                db,
                                file,
                                f.fqn.as_ref(),
                            )
                        },
                        || crate::db::function_taint_summary(db, file, f.fqn.as_ref()),
                        ctx,
                        db,
                        file,
                    );
                }
                // A builtin's result is derived from its arguments unless it can
                // only return values that carry no payload (`strlen`, `is_numeric`).
                let global = resolved.trim_start_matches('\\');
                if !global.contains('\\')
                    && crate::stubs::is_builtin_function(&crate::util::php_ident_lowercase(global))
                {
                    let safe_return = func
                        .as_ref()
                        .and_then(|f| f.return_type.as_deref())
                        .is_some_and(|ty| ty.is_void() || is_injection_safe_type(ty));
                    // A path sink returns file data, not the path; a tainted path
                    // is reported at the call itself.
                    let reads_path = classify_sink(&lower_name) == Some(SinkKind::File);
                    if !safe_return && !reads_path {
                        return args_taint(&fc.args, ctx, db, file);
                    }
                }
            } else if let ExprKind::ArrowFunction(af) = &unwrap_parens(&fc.name).kind {
                // An immediately-invoked, argument-less arrow function has no
                // params to shadow the outer scope, so its body can be checked
                // against the same `ctx`.
                if af.params.is_empty() && fc.args.is_empty() {
                    return expr_taint(&af.body, ctx, db, file);
                }
            }
            Taint::NONE
        }

        ExprKind::StaticPropertyAccess(spa) => {
            crate::expr::assignment::resolve_static_prop_target(spa, ctx, db, file)
                .map_or(Taint::NONE, |(fqcn, prop_name)| {
                    ctx.static_prop_taint(&fqcn, &prop_name)
                })
        }

        ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => {
            if let ExprKind::Identifier(method_name) = &mc.method.kind {
                if let Some(recv_ty) = crate::expr::assignment::resolve_chained_receiver_type(
                    &mc.object, ctx, db, file,
                ) {
                    let method_lower = crate::util::php_ident_lowercase(method_name.as_ref());
                    for atom in &recv_ty.types {
                        if let mir_types::Atomic::TNamedObject { fqcn, .. } = atom {
                            if let Some(removed) =
                                sanitized_method_kinds(db, fqcn.as_ref(), &method_lower)
                            {
                                return args_taint(&mc.args, ctx, db, file).without(removed);
                            }
                            let here = crate::db::Fqcn::from_str(db, fqcn.as_ref());
                            if let Some(taint) =
                                method_call_taint(here, &method_lower, &mc.args, ctx, db, file)
                            {
                                return taint;
                            }
                        }
                    }
                }
            }
            Taint::NONE
        }

        ExprKind::StaticMethodCall(smc) => {
            let fqcn_opt: Option<std::sync::Arc<str>> = if let ExprKind::Identifier(id) =
                &smc.class.kind
            {
                let resolved = crate::db::resolve_name(db, file, id.as_ref());
                match resolved.as_str() {
                    "self" | "static" => ctx.self_fqcn.clone().or_else(|| ctx.static_fqcn.clone()),
                    "parent" => ctx.parent_fqcn.clone(),
                    s => Some(std::sync::Arc::from(s)),
                }
            } else {
                // `$class::method()` / `$this::method()`.
                crate::expr::assignment::resolve_chained_receiver_type(&smc.class, ctx, db, file)
                    .and_then(|ty| {
                        ty.types.iter().find_map(|atom| match atom {
                            mir_types::Atomic::TClassString(Some(fqcn))
                            | mir_types::Atomic::TNamedObject { fqcn, .. }
                            | mir_types::Atomic::TSelf { fqcn }
                            | mir_types::Atomic::TStaticObject { fqcn }
                            | mir_types::Atomic::TParent { fqcn } => {
                                Some(std::sync::Arc::from(fqcn.as_ref()))
                            }
                            _ => None,
                        })
                    })
            };
            if let (Some(fqcn), ExprKind::Identifier(method_name)) = (fqcn_opt, &smc.method.kind) {
                let method_lower = crate::util::php_ident_lowercase(method_name.as_ref());
                let here = crate::db::Fqcn::from_str(db, fqcn.as_ref());
                if let Some(taint) =
                    method_call_taint(here, &method_lower, &smc.args, ctx, db, file)
                {
                    return taint;
                }
            }
            Taint::NONE
        }

        _ => Taint::NONE,
    }
}

/// Whether a declaration at `location` has a body worth summarizing.
fn is_user_code(location: Option<&mir_types::Location>) -> bool {
    location.is_some_and(|loc| !crate::stubs::is_stub_path(loc.file.as_ref()))
}

/// `…/vendor/<vendor>/<package>` prefix of a Composer dependency's path.
fn vendor_package(path: &str) -> Option<&str> {
    let start = path
        .match_indices("vendor")
        .filter(|(i, _)| *i == 0 || path[..*i].ends_with(['/', '\\']))
        .map(|(i, _)| i + "vendor".len())
        .filter(|end| path[*end..].starts_with(['/', '\\']))
        .last()?;
    let mut separators = path[start..]
        .match_indices(['/', '\\'])
        .map(|(i, _)| start + i);
    separators.next()?;
    separators.next()?;
    Some(separators.next().map_or(path, |end| &path[..end]))
}

/// A callee in another Composer package than `from` is treated like a builtin:
/// its arguments pass through, and its body isn't summarized.
pub fn is_dependency_of(callee_path: &str, from: &str) -> bool {
    vendor_package(callee_path).is_some_and(|package| vendor_package(from) != Some(package))
}

/// Taint of a call to `method_lower` on `class`, or `None` when the method is unknown.
fn method_call_taint(
    class: crate::db::Fqcn<'_>,
    method_lower: &str,
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<Taint> {
    let (owner, m) = crate::db::find_method_respecting_precedence(db, class, method_lower)?;
    if m.is_taint_source {
        return Some(Taint::ALL);
    }
    if !is_user_code(m.location.as_ref()) && m.taint.is_none() {
        return None;
    }
    let body = if m.is_abstract || crate::db::is_interface(db, owner.as_ref()) {
        CalleeBody::None
    } else {
        CalleeBody::of(m.location.as_ref(), file)
    };
    let taint = user_call_taint(
        &m.params,
        args,
        m.return_type.as_deref(),
        m.taint.as_deref(),
        body,
        || crate::db::taint_sources::method_reads_source(db, file, owner.as_ref(), method_lower),
        || crate::db::method_taint_summary(db, file, owner.as_ref(), method_lower),
        ctx,
        db,
        file,
    );
    (!taint.is_empty()).then_some(taint)
}

/// Builtins whose result is request data.
pub fn is_source_function(fn_name_lower: &str) -> bool {
    matches!(
        fn_name_lower,
        "getallheaders" | "apache_request_headers" | "filter_input" | "filter_input_array"
    )
}

/// `file_get_contents('php://input')` and friends read the raw request body.
fn reads_raw_request_body(fn_name_lower: &str, args: &[php_ast::owned::Arg]) -> bool {
    matches!(
        fn_name_lower,
        "file_get_contents" | "fopen" | "file" | "readfile" | "stream_get_contents"
    ) && matches!(
        args.first().and_then(|a| a.value.as_ref()).map(|v| &v.kind),
        Some(ExprKind::String(path))
            if path.eq_ignore_ascii_case("php://input") || path.eq_ignore_ascii_case("php://stdin")
    )
}

/// Taint of a `filter_var()`/`filter_input()` result: validating filters
/// (`FILTER_VALIDATE_INT`, …) leave nothing injectable, escaping ones remove
/// their own kind, everything else passes the input through.
fn filter_call_taint(
    fn_name_lower: &str,
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<Taint> {
    let arg = |i: usize| args.get(i).and_then(|a| a.value.as_ref());
    let (input, filter) = match fn_name_lower {
        "filter_var" => (
            arg(0).map_or(Taint::NONE, |v| expr_taint(v, ctx, db, file)),
            arg(1),
        ),
        "filter_input" => (Taint::ALL, arg(2)),
        "filter_input_array" => return Some(Taint::ALL),
        _ => return None,
    };
    let filter_name = match filter.map(|f| &f.kind) {
        Some(ExprKind::Identifier(name)) => name.trim_start_matches('\\').to_ascii_uppercase(),
        _ => return Some(input),
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
fn sanitized_kinds(fn_name_lower: &str) -> Option<Taint> {
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
        // Output is limited to `[A-Za-z0-9%+._~-]`.
        "urlencode" | "rawurlencode" => {
            Taint::HTML
                | Taint::SQL
                | Taint::SHELL
                | Taint::HEADER
                | Taint::COOKIE
                | Taint::LDAP
                | Taint::XPATH
        }
        "md5" | "sha1" | "crc32" | "hash" | "hash_hmac" | "password_hash" | "bin2hex"
        | "dechex" | "decbin" | "decoct" | "number_format" | "uniqid" => Taint::ALL,
        _ => return None,
    })
}

/// Taint kinds a decoding builtin restores on already-tainted input.
fn unescaped_kinds(fn_name_lower: &str) -> Option<Taint> {
    Some(match fn_name_lower {
        "html_entity_decode" | "htmlspecialchars_decode" => Taint::HTML,
        "stripslashes" | "stripcslashes" => Taint::SQL,
        "urldecode" | "rawurldecode" => sanitized_kinds("urlencode")?,
        _ => return None,
    })
}

/// Taint kinds an escaping method of a builtin database class removes.
fn sanitized_method_kinds(
    db: &dyn crate::db::MirDatabase,
    fqcn: &str,
    method_lower: &str,
) -> Option<Taint> {
    let is_a = |base: &str| {
        fqcn.eq_ignore_ascii_case(base) || crate::db::extends_or_implements(db, fqcn, base)
    };
    let escapes = match method_lower {
        "quote" => is_a("PDO"),
        "real_escape_string" | "escape_string" => is_a("mysqli"),
        "escapestring" => is_a("SQLite3"),
        _ => false,
    };
    escapes.then_some(Taint::SQL)
}
