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

/// The declared parameter NAME at a File/Unserialize sink's single relevant
/// index (see `SinkKind::tainted_arg_indices`) — lets the caller resolve a
/// PHP 8 named argument to the same slot a positional call would use, so
/// `file_put_contents(data: 'safe', filename: $_GET['path'])` still flags
/// the actually-tainted `filename` even though it's not in position 0.
/// `None` for Html/Sql/Shell, which check "any argument" and don't need
/// per-parameter name resolution.
pub fn sink_param_name(fn_name_lower: &str) -> Option<&'static str> {
    match fn_name_lower {
        "fopen" | "file_get_contents" | "file_put_contents" | "readfile" | "file" | "unlink" => {
            Some("filename")
        }
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
        // of these takes the path/URL as its first argument, except
        // `move_uploaded_file` (see `sink_positional_index_override`).
        "fopen" | "file_get_contents" | "file_put_contents" | "readfile" | "file" | "unlink"
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
/// issue it should raise. Previously only the literal string `"llm_prompt"`
/// produced anything at all — any other kind (a typo, or one of the sink
/// kinds `classify_sink`/`classify_method_sink` already detect for built-in
/// functions: html/sql/shell) silently did nothing. Named kinds now reuse
/// the same issue those built-in sinks raise; anything else falls back to
/// the generic `TaintedInput`, mirroring the fallback `classify_sink`'s own
/// File/Unserialize arms already use, instead of a silent no-op.
pub fn taint_sink_issue(kind: &str) -> mir_issues::IssueKind {
    match kind {
        "llm_prompt" => mir_issues::IssueKind::TaintedLlmPrompt,
        "html" => mir_issues::IssueKind::TaintedHtml,
        "sql" => mir_issues::IssueKind::TaintedSql,
        "shell" => mir_issues::IssueKind::TaintedShell,
        other => mir_issues::IssueKind::TaintedInput {
            sink: other.to_string(),
        },
    }
}

// ---------------------------------------------------------------------------
// Expression taint checker
// ---------------------------------------------------------------------------

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

/// Returns `true` if the expression could carry tainted data, given the
/// current `FlowState` taint state.
///
/// This is a conservative over-approximation:
/// - Any reference to a superglobal (or an array offset thereof) is tainted.
/// - Any variable that was previously marked tainted in `ctx.tainted_vars` is tainted.
/// - Binary string-concat or arithmetic on tainted operands propagates taint.
/// - Interpolated strings are tainted if any embedded variable is tainted.
/// - A call to a `@taint-source`-annotated function/method is tainted.
pub fn is_expr_tainted(
    expr: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> bool {
    match &expr.kind {
        ExprKind::Variable(name) => {
            let n = name.trim_start_matches('$');
            is_superglobal(n)
                || ctx.is_tainted(n)
                // A variable with no tracked definition at all may be one
                // `extract()` just defined from a tainted source array —
                // its name isn't known statically, so this can't be a
                // specific per-name taint lookup like `is_tainted` above.
                // Doesn't apply to an already-tracked variable: that one has
                // its own real, independently-computed taint state.
                || (ctx.has_dynamic_tainted_var_def
                    && !ctx.var_is_defined_sym(mir_types::Name::from(n)))
        }

        // `$$name` where `$name` holds a known literal string resolves to
        // that variable's own taint state — previously always untainted
        // (no arm at all), even when the accessed variable name was fully
        // known and itself tainted.
        ExprKind::VariableVariable(inner) => {
            if let Some(var_name) = crate::expr::helpers::extract_simple_var(inner) {
                let inner_ty = ctx.get_var(&var_name);
                inner_ty.types.iter().any(|atomic| {
                    matches!(atomic, mir_types::Atomic::TLiteralString(accessed)
                        if ctx.is_tainted(accessed.as_ref()))
                })
            } else {
                false
            }
        }

        ExprKind::ArrayAccess(aa) => {
            // $_GET['key'] — tainted if the array is tainted/superglobal
            is_expr_tainted(&aa.array, ctx, db, file)
        }

        ExprKind::Parenthesized(inner) => is_expr_tainted(inner, ctx, db, file),

        // $obj->prop / $obj?->prop — tainted if this property was previously
        // assigned a tainted value (see FlowState::taint_prop, set on
        // property writes in expr/assignment.rs). Only a simple-variable
        // receiver is tracked, matching prop_refined's narrowing scope.
        // `NullsafePropertyAccess` wraps the identical `PropertyAccessExpr`
        // shape as `PropertyAccess` (PHP forbids `?->` as a write target, so
        // only the read side needs this pairing — unlike the taint-source
        // method-call check below, which pairs both because calls ARE valid
        // through either).
        ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa) => {
            if let ExprKind::Variable(obj_var) = &pa.object.kind {
                if let Some(prop_name) =
                    crate::expr::helpers::extract_string_from_expr(&pa.property)
                {
                    return ctx.is_prop_tainted(obj_var.trim_start_matches('$'), &prop_name);
                }
            }
            false
        }

        ExprKind::Assign(a) => is_expr_tainted(&a.value, ctx, db, file),

        ExprKind::Binary(op) => {
            is_expr_tainted(&op.left, ctx, db, file) || is_expr_tainted(&op.right, ctx, db, file)
        }

        ExprKind::UnaryPrefix(u) => is_expr_tainted(&u.operand, ctx, db, file),

        ExprKind::InterpolatedString(parts)
        | ExprKind::Heredoc { parts, .. }
        | ExprKind::ShellExec(parts) => parts.iter().any(|p| match p {
            StringPart::Expr(e) => is_expr_tainted(e, ctx, db, file),
            StringPart::Literal(_) => false,
        }),

        ExprKind::Ternary(t) => match &t.then_expr {
            Some(then_e) => {
                is_expr_tainted(then_e, ctx, db, file)
                    || is_expr_tainted(&t.else_expr, ctx, db, file)
            }
            // Short ternary (`$x ?: $y`): the true branch's VALUE is the
            // condition itself, not a separate expression — `then_expr` is
            // `None` for this form, so the condition's own taint must be
            // checked too, not just skipped.
            None => {
                is_expr_tainted(&t.condition, ctx, db, file)
                    || is_expr_tainted(&t.else_expr, ctx, db, file)
            }
        },

        // `$x ?? $default` — tainted if either side could be, same as a
        // ternary. This is the single most common superglobal-read idiom
        // (`$_GET['x'] ?? 'default'`), so missing it left a large real-world
        // coverage hole.
        ExprKind::NullCoalesce(nc) => {
            is_expr_tainted(&nc.left, ctx, db, file) || is_expr_tainted(&nc.right, ctx, db, file)
        }

        // Numeric/boolean casts sanitize: PHP coerces the value to that scalar
        // type, so a subsequent SQL/shell/HTML sink can no longer receive
        // arbitrary attacker-controlled text through it. String/array/object
        // casts don't remove the payload, so taint still propagates.
        ExprKind::Cast(kind, inner) => match kind {
            php_ast::ast::CastKind::Int
            | php_ast::ast::CastKind::Float
            | php_ast::ast::CastKind::Bool => false,
            _ => is_expr_tainted(inner, ctx, db, file),
        },

        ExprKind::Match(m) => m
            .arms
            .iter()
            .any(|arm| is_expr_tainted(&arm.body, ctx, db, file)),

        // An element's KEY can carry taint just as much as its value
        // (`[$_GET['env'] => 'active']` — the attacker controls which key
        // exists), but only `el.value` was ever checked here.
        ExprKind::Array(elements) => elements.iter().any(|el| {
            is_expr_tainted(&el.value, ctx, db, file)
                || el
                    .key
                    .as_ref()
                    .is_some_and(|k| is_expr_tainted(k, ctx, db, file))
        }),

        // `@taint-source`-annotated function/method calls are themselves a
        // taint source, mirroring `@taint-sink`'s mechanism on the source
        // side. A general "any call result could be tainted" pass-through
        // (e.g. htmlspecialchars-style sanitizers, or an unannotated call
        // whose body reads a superglobal) is NOT modeled — that's a much
        // bigger, deliberately deferred change.
        ExprKind::FunctionCall(fc) => {
            if let ExprKind::Identifier(name) = &fc.name.kind {
                // `getallheaders()`/`apache_request_headers()` return the raw
                // HTTP request headers — as attacker-controlled as any
                // superglobal, but neither was ever treated as a source
                // (unlike $_SERVER, which only covers some of the same data).
                if matches!(
                    crate::util::php_ident_lowercase(name.as_ref()).as_str(),
                    "getallheaders" | "apache_request_headers"
                ) {
                    return true;
                }
                // `sprintf`/`vsprintf` interpolate every argument straight
                // into the returned string — a tainted format string OR any
                // tainted interpolated value makes the result tainted, the
                // same pass-through `call/function.rs`/`call/callable.rs`
                // already special-case for these two builtins' return-type
                // inference.
                if matches!(
                    crate::util::php_ident_lowercase(name.as_ref()).as_str(),
                    "sprintf" | "vsprintf"
                ) && fc
                    .args
                    .iter()
                    .any(|a| is_expr_tainted(&a.value, ctx, db, file))
                {
                    return true;
                }
                // `compact('id')` copies `$id`'s CURRENT value into the
                // returned array under key `'id'` — `call/array_builtins.rs`'s
                // `compact_return_type` already reads each named variable's
                // TYPE this same way for shape inference; do the same lookup
                // for taint so `$data = compact('id'); echo $data['id'];`
                // is recognized as tainted when `$id` is. Only literal
                // string argument names are handled, matching
                // `compact_return_type`'s own scope (a dynamic/computed name
                // bails there too).
                if crate::util::php_ident_lowercase(name.as_ref()) == "compact"
                    && fc.args.iter().any(|a| {
                        matches!(&a.value.kind, ExprKind::String(var_name) if ctx.is_tainted(var_name.as_ref()))
                    })
                {
                    return true;
                }
                // A broad set of non-sanitizing string-transform builtins pass
                // taint straight through, unchanged — same "any tainted
                // argument taints the result" rule already established for
                // sprintf/vsprintf above. Deliberately excludes genuine
                // sanitizers/encoders (htmlspecialchars, htmlentities,
                // addslashes, urlencode, rawurlencode,
                // quoted_printable_encode, …), which correctly stay untainted.
                if matches!(
                    crate::util::php_ident_lowercase(name.as_ref()).as_str(),
                    "str_replace"
                        | "str_ireplace"
                        | "preg_replace"
                        | "preg_replace_callback"
                        | "substr_replace"
                        | "trim"
                        | "ltrim"
                        | "rtrim"
                        | "explode"
                        | "implode"
                        | "join"
                        | "strtolower"
                        | "strtoupper"
                        | "mb_strtolower"
                        | "mb_strtoupper"
                        | "ucfirst"
                        | "ucwords"
                        | "wordwrap"
                        | "chunk_split"
                        | "str_pad"
                        | "str_rot13"
                        | "nl2br"
                ) && fc
                    .args
                    .iter()
                    .any(|a| is_expr_tainted(&a.value, ctx, db, file))
                {
                    return true;
                }
                let resolved = crate::db::resolve_name(db, file, name.as_ref());
                let here = crate::db::Fqcn::from_str(db, resolved.as_str());
                if crate::db::find_function(db, here).is_some_and(|f| f.is_taint_source) {
                    return true;
                }
            } else if let ExprKind::ArrowFunction(af) = &unwrap_parens(&fc.name).kind {
                // An immediately-invoked, argument-less arrow function
                // (`(fn() => $_GET['x'])()`) is tainted iff its single
                // expression body is — narrower than the general
                // "any call result could be tainted" pass-through this
                // module deliberately doesn't model (see the comment
                // above), since an arrow function's body is a single
                // expression evaluated with no params to shadow the outer
                // scope, so checking it directly against the SAME `ctx`
                // is sound here specifically. A closure (`function(){...}`)
                // or an arrow function taking arguments is NOT covered —
                // both need real per-callee analysis, not a same-ctx check.
                if af.params.is_empty() && fc.args.is_empty() {
                    return is_expr_tainted(&af.body, ctx, db, file);
                }
            }
            false
        }

        // self::$prop / Foo::$prop — tainted if this static property was
        // previously assigned a tainted value (see FlowState::taint_static_prop,
        // set on static-property writes in expr/assignment.rs).
        ExprKind::StaticPropertyAccess(spa) => {
            crate::expr::assignment::resolve_static_prop_target(spa, ctx, db, file)
                .is_some_and(|(fqcn, prop_name)| ctx.is_static_prop_tainted(&fqcn, &prop_name))
        }

        ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => {
            if let ExprKind::Identifier(method_name) = &mc.method.kind {
                // Chain-walk the receiver (`$this->req->getParam()`) the same
                // way the purity checks already do for a write, instead of
                // requiring `mc.object` to literally BE a bare variable.
                if let Some(recv_ty) = crate::expr::assignment::resolve_chained_receiver_type(
                    &mc.object, ctx, db, file,
                ) {
                    let method_lower = crate::util::php_ident_lowercase(method_name.as_ref());
                    for atom in &recv_ty.types {
                        if let mir_types::Atomic::TNamedObject { fqcn, .. } = atom {
                            let here = crate::db::Fqcn::from_str(db, fqcn.as_ref());
                            if crate::db::find_method_respecting_precedence(db, here, &method_lower)
                                .is_some_and(|(_, m)| m.is_taint_source)
                            {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        }

        // Foo::getQuery() / self::getQuery() with @taint-source — static-call
        // counterpart of the instance-method arm above, which this had no
        // pairing for at all (unlike the read-side StaticPropertyAccess arm
        // right below, which already handles self/static/parent).
        ExprKind::StaticMethodCall(smc) => {
            if let ExprKind::Identifier(id) = &smc.class.kind {
                let resolved = crate::db::resolve_name(db, file, id.as_ref());
                let fqcn_opt: Option<std::sync::Arc<str>> = match resolved.as_str() {
                    "self" | "static" => ctx.self_fqcn.clone().or_else(|| ctx.static_fqcn.clone()),
                    "parent" => ctx.parent_fqcn.clone(),
                    s => Some(std::sync::Arc::from(s)),
                };
                if let (Some(fqcn), ExprKind::Identifier(method_name)) =
                    (fqcn_opt, &smc.method.kind)
                {
                    let method_lower = crate::util::php_ident_lowercase(method_name.as_ref());
                    let here = crate::db::Fqcn::from_str(db, fqcn.as_ref());
                    if crate::db::find_method_respecting_precedence(db, here, &method_lower)
                        .is_some_and(|(_, m)| m.is_taint_source)
                    {
                        return true;
                    }
                }
            }
            false
        }

        // Conservative: function call results are not tracked as tainted
        // unless it's a known pass-through built-in (htmlspecialchars sanitizes)
        _ => false,
    }
}
