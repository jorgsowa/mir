use std::sync::Arc;

use crate::db::{resolve_name_via_db, type_exists_via_db, MirDatabase};
use crate::php_version::PhpVersion;

// ---------------------------------------------------------------------------
// Offset to char-count column conversion
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
            let resolved = resolve_name_via_db(db, file.as_ref(), &name_str);
            if !type_exists_via_db(db, &resolved) {
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
                            col_end: col_end.max(col_start + 1),
                        },
                    )
                    .with_snippet(crate::parser::span_text(source, hint.span).unwrap_or_default()),
                );
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
            let resolved = resolve_name_via_db(db, file.as_ref(), &name_str);
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
    let resolved = resolve_name_via_db(db, file.as_ref(), &name_str);
    if !type_exists_via_db(db, &resolved) {
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
                    col_end: col_end.max(col_start + 1),
                },
            )
            .with_snippet(crate::parser::span_text(source, span).unwrap_or_default()),
        );
        return;
    }

    // Check if extending an interface
    if is_extends {
        let here = crate::db::Fqcn::new(db, Arc::<str>::from(resolved.as_str()));
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
                            col_end: col_end.max(col_start + 1),
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
        name.to_lowercase().as_str(),
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
            let resolved = resolve_name_via_db(db, file.as_ref(), &name_str);
            if !type_exists_via_db(db, &resolved) {
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
                            col_end: col_end.max(col_start + 1),
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
    ctx: &crate::context::Context,
    method_name: &str,
    file: &Arc<str>,
    issues: &mut Vec<mir_issues::Issue>,
) {
    if MAGIC_METHODS_WITH_RUNTIME_PARAMS.contains(&method_name) {
        return;
    }
    for p in params {
        let name = p.name.as_ref().trim_start_matches('$');
        if !ctx.read_vars.contains(name) {
            let (line, col_start, line_end, col_end) =
                ctx.var_locations.get(name).copied().unwrap_or((1, 0, 1, 0));
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
                        col_end: col_end.max(col_start + 1),
                    },
                )
                .with_snippet(format!("${name}")),
            );
        }
    }
}

pub(crate) fn emit_unused_variables(
    ctx: &crate::context::Context,
    file: &Arc<str>,
    issues: &mut Vec<mir_issues::Issue>,
) {
    const SUPERGLOBALS: &[&str] = &[
        "_SERVER", "_GET", "_POST", "_REQUEST", "_SESSION", "_COOKIE", "_FILES", "_ENV", "GLOBALS",
    ];
    for name in &ctx.assigned_vars {
        if ctx.param_names.contains(name) {
            continue;
        }
        if SUPERGLOBALS.contains(&name.as_str()) {
            continue;
        }
        if name == "this" {
            continue;
        }
        if name.starts_with('_') {
            continue;
        }
        if !ctx.read_vars.contains(name) {
            let (line, col_start, line_end, col_end) = ctx
                .var_locations
                .get(name.as_str())
                .copied()
                .unwrap_or((1, 0, 1, 0));
            issues.push(mir_issues::Issue::new(
                mir_issues::IssueKind::UnusedVariable { name: name.clone() },
                mir_issues::Location {
                    file: file.clone(),
                    line,
                    line_end,
                    col_start,
                    col_end: col_end.max(col_start + 1),
                },
            ));
        }
    }
}
