use std::sync::Arc;

use crate::db::{class_exists, resolve_name, MirDatabase};
use crate::php_version::PhpVersion;

// ---------------------------------------------------------------------------
// Stored-location → Issue Location passthrough
// ---------------------------------------------------------------------------

/// Convert a stored `mir_types::Location` reference into an `Issue` `Location`,
/// passing all fields through unchanged.  Use this for diagnostics whose stored
/// span is already tight (property, method, function declarations).
/// For class-level spans that cover the entire body, use the clamping logic in
/// `class.rs::issue_location` instead.
pub(crate) fn storage_loc_to_location(loc: Option<&mir_types::Location>) -> mir_issues::Location {
    match loc {
        Some(l) => mir_issues::Location {
            file: l.file.clone(),
            line: l.line,
            line_end: l.line_end,
            col_start: l.col_start,
            col_end: l.col_end,
        },
        None => mir_issues::Location {
            file: Arc::from("<unknown>"),
            line: 1,
            line_end: 1,
            col_start: 0,
            col_end: 1,
        },
    }
}

// ---------------------------------------------------------------------------
// Offset to char-count column conversion (1-indexed)
// ---------------------------------------------------------------------------

pub(crate) fn offset_to_line_col(
    source: &str,
    offset: u32,
    source_map: &php_rs_parser::source_map::SourceMap,
) -> (u32, u16) {
    let lc = source_map.offset_to_line_col(offset);
    let line = lc.line + 1;

    let byte_offset = offset as usize;
    let line_start_byte = if byte_offset == 0 {
        0
    } else {
        source[..byte_offset]
            .rfind('\n')
            .map(|p| p + 1)
            .unwrap_or(0)
    };

    let col = source[line_start_byte..byte_offset].chars().count() as u16;

    (line, col)
}

/// Widen a same-line span by at least one column so a genuinely zero-width
/// span (e.g. `start == end`) still renders as a visible range. Must NOT be
/// applied across lines: `col_start`/`col_end` are columns on different lines
/// there, so comparing them is meaningless — taking their max produces a
/// nonsensical column past the end of `line_end`'s actual content.
pub(crate) fn clamp_col_end(line: u32, line_end: u32, col_start: u16, col_end: u16) -> u16 {
    if line == line_end {
        col_end.max(col_start.saturating_add(1))
    } else {
        col_end
    }
}

// ---------------------------------------------------------------------------
// Type-hint class existence checker
// ---------------------------------------------------------------------------

pub(crate) fn check_type_hint_classes(
    hint: &php_ast::owned::TypeHint,
    db: &dyn MirDatabase,
    file: &Arc<str>,
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
    issues: &mut Vec<mir_issues::Issue>,
    php_version: PhpVersion,
) {
    use php_ast::owned::TypeHintKind;
    match &hint.kind {
        TypeHintKind::Named(name) => {
            let name_str = crate::parser::name_to_string_owned(name);
            if is_pseudo_type(&name_str) {
                return;
            }
            let resolved = resolve_name(db, file.as_ref(), &name_str);
            if !class_exists(db, &resolved) {
                // Soft-fallback: build-time stub index recognises this class
                // as a PHP built-in → assume lazy-stub timing rather than
                // user error. See call/function.rs for the parallel path.
                // However, don't suppress if the class is version-filtered.
                if let Some(stub_path) = crate::stubs::stub_path_for_class(&resolved) {
                    if let Some(stub_src) = crate::stubs::stub_content_for_path(stub_path) {
                        if let Some(docblock_text) =
                            crate::call::extract_class_docblock(stub_src, &resolved)
                        {
                            let doc = crate::parser::DocblockParser::parse(docblock_text);
                            if php_version
                                .includes_symbol(doc.since.as_deref(), doc.removed.as_deref())
                            {
                                return;
                            }
                        } else {
                            return;
                        }
                    }
                }
                let (line, col_start) = offset_to_line_col(source, hint.span.start, source_map);
                let (line_end, col_end) = if hint.span.start < hint.span.end {
                    let (end_line, end_col) = offset_to_line_col(source, hint.span.end, source_map);
                    (end_line, end_col)
                } else {
                    (line, col_start)
                };
                issues.push(
                    mir_issues::Issue::new(
                        mir_issues::IssueKind::UndefinedClass { name: resolved },
                        mir_issues::Location {
                            file: file.clone(),
                            line,
                            line_end,
                            col_start,
                            col_end: clamp_col_end(line, line_end, col_start, col_end),
                        },
                    )
                    .with_snippet(crate::parser::span_text(source, hint.span).unwrap_or_default()),
                );
            } else {
                // Class exists — check for wrong case and deprecation
                let here = crate::db::Fqcn::from_str(db, resolved.as_str());
                if let Some(class) = crate::db::find_class_like(db, here) {
                    if let Some((used, canonical_str)) =
                        crate::fqcn_case_mismatch(&name_str, class.fqcn().as_ref())
                    {
                        let (line, col_start) =
                            offset_to_line_col(source, hint.span.start, source_map);
                        let (line_end, col_end) = if hint.span.start < hint.span.end {
                            offset_to_line_col(source, hint.span.end, source_map)
                        } else {
                            (line, col_start)
                        };
                        issues.push(mir_issues::Issue::new(
                            mir_issues::IssueKind::WrongCaseClass {
                                used,
                                canonical: canonical_str,
                            },
                            mir_issues::Location {
                                file: file.clone(),
                                line,
                                line_end,
                                col_start,
                                col_end: clamp_col_end(line, line_end, col_start, col_end),
                            },
                        ));
                    }
                    if let Some(msg) = class.deprecated() {
                        let (line, col_start) =
                            offset_to_line_col(source, hint.span.start, source_map);
                        let (line_end, col_end) = if hint.span.start < hint.span.end {
                            offset_to_line_col(source, hint.span.end, source_map)
                        } else {
                            (line, col_start)
                        };
                        issues.push(mir_issues::Issue::new(
                            mir_issues::IssueKind::DeprecatedClass {
                                name: resolved,
                                message: Some(msg.clone()).filter(|m| !m.is_empty()),
                            },
                            mir_issues::Location {
                                file: file.clone(),
                                line,
                                line_end,
                                col_start,
                                col_end: clamp_col_end(line, line_end, col_start, col_end),
                            },
                        ));
                    }
                }
            }
        }
        TypeHintKind::Nullable(inner) => {
            check_type_hint_classes(inner, db, file, source, source_map, issues, php_version);
        }
        TypeHintKind::Union(parts) | TypeHintKind::Intersection(parts) => {
            for part in parts.iter() {
                check_type_hint_classes(part, db, file, source, source_map, issues, php_version);
            }
        }
        TypeHintKind::Keyword(_, _) => {}
    }
}

/// Collect all resolved Named class FQCNs referenced in a type hint, regardless
/// of whether those classes exist. Used to record dependency edges even for
/// classes that are defined (not just missing ones).
pub(crate) fn collect_type_hint_class_refs(
    hint: &php_ast::owned::TypeHint,
    db: &dyn MirDatabase,
    file: &Arc<str>,
) -> Vec<(Arc<str>, php_ast::Span)> {
    let mut out = Vec::new();
    collect_type_hint_class_refs_inner(hint, db, file, &mut out);
    out
}

fn collect_type_hint_class_refs_inner(
    hint: &php_ast::owned::TypeHint,
    db: &dyn MirDatabase,
    file: &Arc<str>,
    out: &mut Vec<(Arc<str>, php_ast::Span)>,
) {
    use php_ast::owned::TypeHintKind;
    match &hint.kind {
        TypeHintKind::Named(name) => {
            let name_str = crate::parser::name_to_string_owned(name);
            if is_pseudo_type(&name_str) {
                return;
            }
            let resolved = resolve_name(db, file.as_ref(), &name_str);
            out.push((Arc::from(resolved.as_str()), hint.span));
        }
        TypeHintKind::Nullable(inner) => {
            collect_type_hint_class_refs_inner(inner, db, file, out);
        }
        TypeHintKind::Union(parts) | TypeHintKind::Intersection(parts) => {
            for part in parts.iter() {
                collect_type_hint_class_refs_inner(part, db, file, out);
            }
        }
        TypeHintKind::Keyword(_, _) => {}
    }
}

pub(crate) fn check_name_class(
    name: &php_ast::owned::Name,
    db: &dyn MirDatabase,
    file: &Arc<str>,
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
    issues: &mut Vec<mir_issues::Issue>,
    php_version: PhpVersion,
) {
    check_name_class_with_context(
        name,
        db,
        file,
        source,
        source_map,
        issues,
        php_version,
        false,
    );
}

pub(crate) fn check_name_class_for_extends(
    name: &php_ast::owned::Name,
    db: &dyn MirDatabase,
    file: &Arc<str>,
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
    issues: &mut Vec<mir_issues::Issue>,
    php_version: PhpVersion,
) {
    check_name_class_with_context(
        name,
        db,
        file,
        source,
        source_map,
        issues,
        php_version,
        true,
    );
}

#[allow(clippy::too_many_arguments)]
fn check_name_class_with_context(
    name: &php_ast::owned::Name,
    db: &dyn MirDatabase,
    file: &Arc<str>,
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
    issues: &mut Vec<mir_issues::Issue>,
    php_version: PhpVersion,
    is_extends: bool,
) {
    let name_str = crate::parser::name_to_string_owned(name);
    let resolved = resolve_name(db, file.as_ref(), &name_str);
    if !class_exists(db, &resolved) {
        // Soft-fallback: see call/function.rs for the rationale.
        // However, don't suppress if the class is version-filtered.
        if let Some(stub_path) = crate::stubs::stub_path_for_class(&resolved) {
            if let Some(stub_src) = crate::stubs::stub_content_for_path(stub_path) {
                if let Some(docblock_text) =
                    crate::call::extract_class_docblock(stub_src, &resolved)
                {
                    let doc = crate::parser::DocblockParser::parse(docblock_text);
                    if php_version.includes_symbol(doc.since.as_deref(), doc.removed.as_deref()) {
                        return;
                    }
                } else {
                    return;
                }
            }
        }
        let span = name.span;
        let (line, col_start) = offset_to_line_col(source, span.start, source_map);
        let (line_end, col_end) = offset_to_line_col(source, span.end, source_map);
        issues.push(
            mir_issues::Issue::new(
                mir_issues::IssueKind::UndefinedClass { name: resolved },
                mir_issues::Location {
                    file: file.clone(),
                    line,
                    line_end,
                    col_start,
                    col_end: clamp_col_end(line, line_end, col_start, col_end),
                },
            )
            .with_snippet(crate::parser::span_text(source, span).unwrap_or_default()),
        );
        return;
    }

    // Check if extending an interface
    if is_extends {
        let here = crate::db::Fqcn::from_str(db, resolved.as_str());
        let is_iface = crate::db::find_class_like(db, here)
            .map(|c| c.is_interface())
            .unwrap_or(false);
        if is_iface {
            {
                let span = name.span;
                let (line, col_start) = offset_to_line_col(source, span.start, source_map);
                let (line_end, col_end) = offset_to_line_col(source, span.end, source_map);
                issues.push(
                    mir_issues::Issue::new(
                        mir_issues::IssueKind::UndefinedClass { name: resolved },
                        mir_issues::Location {
                            file: file.clone(),
                            line,
                            line_end,
                            col_start,
                            col_end: clamp_col_end(line, line_end, col_start, col_end),
                        },
                    )
                    .with_snippet(crate::parser::span_text(source, span).unwrap_or_default()),
                );
            }
        }
    }
}

fn is_pseudo_type(name: &str) -> bool {
    matches!(
        crate::util::php_ident_lowercase(name).as_str(),
        "self"
            | "static"
            | "parent"
            | "null"
            | "true"
            | "false"
            | "never"
            | "void"
            | "mixed"
            | "object"
            | "callable"
            | "iterable"
    )
}

// ---------------------------------------------------------------------------
// Expression class checking
// ---------------------------------------------------------------------------

pub(crate) fn check_expr_for_undefined_classes(
    expr: &php_ast::owned::Expr,
    db: &dyn MirDatabase,
    file: &Arc<str>,
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
    issues: &mut Vec<mir_issues::Issue>,
    _php_version: PhpVersion,
) {
    use php_ast::owned::ExprKind;
    if let ExprKind::ClassConstAccess(cca) = &expr.kind {
        // Check for undefined class in ::CONSTANT or ::class
        if let ExprKind::Identifier(class_name) = &cca.class.kind {
            let name_str = class_name.to_string();
            let resolved = resolve_name(db, file.as_ref(), &name_str);
            if !class_exists(db, &resolved) {
                let (line, col_start) =
                    offset_to_line_col(source, cca.class.span.start, source_map);
                let (line_end, col_end) =
                    offset_to_line_col(source, cca.class.span.end, source_map);
                issues.push(
                    mir_issues::Issue::new(
                        mir_issues::IssueKind::UndefinedClass { name: resolved },
                        mir_issues::Location {
                            file: file.clone(),
                            line,
                            line_end,
                            col_start,
                            col_end: clamp_col_end(line, line_end, col_start, col_end),
                        },
                    )
                    .with_snippet(
                        crate::parser::span_text(source, cca.class.span).unwrap_or_default(),
                    ),
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Unused param / variable emission
// ---------------------------------------------------------------------------

const MAGIC_METHODS_WITH_RUNTIME_PARAMS: &[&str] = &[
    "__get",
    "__set",
    "__call",
    "__callStatic",
    "__isset",
    "__unset",
    "__unserialize",
];

pub(crate) fn emit_unused_params(
    params: &[mir_codebase::FnParam],
    ctx: &crate::flow_state::FlowState,
    method_name: &str,
    file: &Arc<str>,
    issues: &mut Vec<mir_issues::Issue>,
) {
    if MAGIC_METHODS_WITH_RUNTIME_PARAMS.contains(&method_name) {
        return;
    }
    for p in params {
        let name = p.name.as_ref().trim_start_matches('$');
        // Skip the synthetic variadic param injected by func_get_args() detection —
        // its name "..." is not a valid PHP identifier and never appears in source.
        if name == "..." {
            continue;
        }
        let name_sym = mir_types::Name::from(name);
        if !ctx.read_vars.contains(&name_sym) {
            let (line, col_start, line_end, col_end) = ctx
                .var_locations
                .get(&name_sym)
                .copied()
                .unwrap_or((1, 0, 1, 0));
            issues.push(
                mir_issues::Issue::new(
                    mir_issues::IssueKind::UnusedParam {
                        name: name.to_string(),
                    },
                    mir_issues::Location {
                        file: file.clone(),
                        line,
                        line_end,
                        col_start,
                        col_end: clamp_col_end(line, line_end, col_start, col_end),
                    },
                )
                .with_snippet(format!("${name}")),
            );
        }
    }
}

/// A `__toString` method must return a `string`. Emits `InvalidToString` when
/// the effective return type (declared if present, else inferred from the body)
/// is definitely not a string. Conservative: `mixed`/empty types are skipped so
/// incomplete inference never produces a false positive.
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_to_string_return(
    fqcn: &str,
    declared_return: Option<&mir_types::Type>,
    inferred: &mir_types::Type,
    body_span: &php_ast::Span,
    file: &Arc<str>,
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
    issues: &mut Vec<mir_issues::Issue>,
) {
    let effective = declared_return.unwrap_or(inferred);
    if effective.is_mixed() || effective.is_empty() {
        return;
    }
    if effective.types.iter().all(|a| a.is_string()) {
        return;
    }
    let (line, col_start) = offset_to_line_col(source, body_span.start, source_map);
    let (line_end, col_end) = offset_to_line_col(source, body_span.end, source_map);
    issues.push(mir_issues::Issue::new(
        mir_issues::IssueKind::InvalidToString {
            class: fqcn.to_string(),
        },
        mir_issues::Location {
            file: file.clone(),
            line,
            line_end,
            col_start,
            col_end: clamp_col_end(line, line_end, col_start, col_end),
        },
    ));
}

/// True when a declared return type obliges every code path to `return` a
/// value, so falling off the end of the body is an error. Conservative:
/// `void`/`mixed`/nullable returns are exempt (falling off yields `null`,
/// which those accept or which is moot). `never` is NOT exempt: a `: never`
/// body that falls off the end is a bug — it must always throw/exit/diverge.
/// Iterable/generator-like returns are exempt because a generator body
/// legitimately never returns.
fn return_requires_value(t: &mir_types::Type) -> bool {
    use mir_types::Atomic;
    if t.is_empty() || t.is_void() || t.is_mixed() || t.is_nullable() {
        return false;
    }
    // `static|void`, `T|void` — void in a union means implicit return is valid.
    if t.contains(|a| matches!(a, Atomic::TVoid)) {
        return false;
    }
    // Conditional and template return types are resolved per-call/contextually;
    // an empty-bodied stub with such a return must not be flagged (mirrors the
    // exemption in `analyze_return_stmt`).
    if t.types.iter().any(|a| {
        matches!(
            a,
            Atomic::TConditional { .. } | Atomic::TTemplateParam { .. }
        )
    }) {
        return false;
    }
    !t.types.iter().any(|a| match a {
        Atomic::TNamedObject { fqcn, .. } => {
            let n = fqcn.trim_start_matches('\\');
            n.eq_ignore_ascii_case("Generator")
                || n.eq_ignore_ascii_case("Iterator")
                || n.eq_ignore_ascii_case("IteratorAggregate")
                || n.eq_ignore_ascii_case("Traversable")
                || n.eq_ignore_ascii_case("iterable")
        }
        // `iterable` and array returns also cover generator bodies.
        Atomic::TArray { .. }
        | Atomic::TList { .. }
        | Atomic::TNonEmptyArray { .. }
        | Atomic::TNonEmptyList { .. }
        | Atomic::TKeyedArray { .. } => true,
        _ => false,
    })
}

/// Emit `InvalidReturnType` when a value-returning function/method can reach the
/// end of its body without returning. `diverges` is the flow flag after body
/// analysis: `true` means every path already returned/threw/exited.
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_missing_return(
    declared_return: Option<&mir_types::Type>,
    diverges: bool,
    body_span: &php_ast::Span,
    file: &Arc<str>,
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
    issues: &mut Vec<mir_issues::Issue>,
) {
    if diverges {
        return;
    }
    let Some(declared) = declared_return else {
        return;
    };
    if !return_requires_value(declared) {
        return;
    }
    let (line, col_start) = offset_to_line_col(source, body_span.start, source_map);
    let (line_end, col_end) = offset_to_line_col(source, body_span.end, source_map);
    issues.push(mir_issues::Issue::new(
        mir_issues::IssueKind::InvalidReturnType {
            expected: format!("{declared}"),
            actual: "void".to_string(),
        },
        mir_issues::Location {
            file: file.clone(),
            line,
            line_end,
            col_start,
            col_end: clamp_col_end(line, line_end, col_start, col_end),
        },
    ));
}

/// Returns true for Blade templates and files under `resources/views/`.
/// Normalizes path separators before matching so Windows paths work correctly.
pub(crate) fn is_view_template_path(file: &str) -> bool {
    if file.ends_with(".blade.php") {
        return true;
    }
    let normalized = file.replace('\\', "/");
    normalized.contains("/resources/views/")
}

pub(crate) fn emit_unused_variables(
    ctx: &crate::flow_state::FlowState,
    file: &Arc<str>,
    issues: &mut Vec<mir_issues::Issue>,
) {
    // View template files have variables injected from the calling scope; unused-variable
    // diagnostics are false positives there.
    if is_view_template_path(file) {
        return;
    }

    const SUPERGLOBALS: &[&str] = &[
        "_SERVER", "_GET", "_POST", "_REQUEST", "_SESSION", "_COOKIE", "_FILES", "_ENV", "GLOBALS",
        "argv", "argc",
    ];

    // Helper: should we skip this variable name?
    let skip = |name: &mir_types::Name| -> bool {
        ctx.param_names.contains(name)
            || SUPERGLOBALS.contains(&name.as_str())
            || name == "this"
            || name.starts_with('_')
            || ctx.foreach_byref_var_names.contains(name)
            || ctx.catch_var_names.contains(name)
    };

    // Emit at most one UnusedVariable/UnusedForeachValue per variable name to avoid
    // noise from multiple dead-write occurrences in complex control flow.
    let mut emitted_names: rustc_hash::FxHashSet<mir_types::Name> =
        rustc_hash::FxHashSet::default();

    let mut push = |name: mir_types::Name, // Name is Copy
                    line: u32,
                    col_start: u16,
                    line_end: u32,
                    col_end: u16,
                    issues: &mut Vec<mir_issues::Issue>| {
        if emitted_names.insert(name) {
            let kind = if ctx.foreach_value_var_names.contains(&name) {
                mir_issues::IssueKind::UnusedForeachValue {
                    name: name.to_string(),
                }
            } else {
                mir_issues::IssueKind::UnusedVariable {
                    name: name.to_string(),
                }
            };
            issues.push(mir_issues::Issue::new(
                kind,
                mir_issues::Location {
                    file: file.clone(),
                    line,
                    line_end,
                    col_start,
                    col_end: clamp_col_end(line, line_end, col_start, col_end),
                },
            ));
        }
    };

    // Dead writes: values overwritten without being read (detected at overwrite time).
    // These are emitted at the location of the overwritten (dead) write.
    // A write consumed by a read on ANY path (e.g. the loop-never-ran path
    // after `foreach { $x = ...; }`) is not dead.
    for (name, line, col_start, line_end, col_end) in &ctx.dead_writes {
        if skip(name)
            || ctx
                .consumed_write_locs
                .contains(&(*name, (*line, *col_start, *line_end, *col_end)))
        {
            continue;
        }
        push(*name, *line, *col_start, *line_end, *col_end, issues);
    }

    // Remaining pending writes: variables with a write that was never consumed.
    // This covers both "variable never read at all" and compound-op results not read.
    for (name, locs) in &ctx.last_write_locs {
        if skip(name) {
            continue;
        }
        for (line, col_start, line_end, col_end) in locs {
            if ctx
                .consumed_write_locs
                .contains(&(*name, (*line, *col_start, *line_end, *col_end)))
            {
                continue;
            }
            push(*name, *line, *col_start, *line_end, *col_end, issues);
        }
    }

    // Fallback for variables in assigned_vars that lack last_write_locs entries
    // (e.g. created via set_var without record_var_location in older code paths).
    for name in ctx.assigned_vars.iter() {
        if skip(name) {
            continue;
        }
        if !ctx.read_vars.contains(name) && !ctx.last_write_locs.contains_key(name) {
            let (line, col_start, line_end, col_end) =
                ctx.var_locations.get(name).copied().unwrap_or((1, 0, 1, 0));
            push(*name, line, col_start, line_end, col_end, issues);
        }
    }
}
