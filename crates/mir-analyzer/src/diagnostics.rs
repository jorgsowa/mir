use std::sync::Arc;

use mir_codebase::Codebase;

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

pub(crate) fn check_type_hint_classes<'arena, 'src>(
    hint: &php_ast::ast::TypeHint<'arena, 'src>,
    codebase: &Codebase,
    file: &Arc<str>,
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
    issues: &mut Vec<mir_issues::Issue>,
) {
    use php_ast::ast::TypeHintKind;
    match &hint.kind {
        TypeHintKind::Named(name) => {
            let name_str = crate::parser::name_to_string(name);
            if is_pseudo_type(&name_str) {
                return;
            }
            let resolved = codebase.resolve_class_name(file.as_ref(), &name_str);
            if !codebase.type_exists(&resolved) {
                let (line, col_start) = offset_to_line_col(source, hint.span.start, source_map);
                let col_end = if hint.span.start < hint.span.end {
                    let (_end_line, end_col) =
                        offset_to_line_col(source, hint.span.end, source_map);
                    end_col
                } else {
                    col_start
                };
                issues.push(
                    mir_issues::Issue::new(
                        mir_issues::IssueKind::UndefinedClass { name: resolved },
                        mir_issues::Location {
                            file: file.clone(),
                            line,
                            col_start,
                            col_end: col_end.max(col_start + 1),
                        },
                    )
                    .with_snippet(crate::parser::span_text(source, hint.span).unwrap_or_default()),
                );
            }
        }
        TypeHintKind::Nullable(inner) => {
            check_type_hint_classes(inner, codebase, file, source, source_map, issues);
        }
        TypeHintKind::Union(parts) | TypeHintKind::Intersection(parts) => {
            for part in parts.iter() {
                check_type_hint_classes(part, codebase, file, source, source_map, issues);
            }
        }
        TypeHintKind::Keyword(_, _) => {}
    }
}

pub(crate) fn check_name_class(
    name: &php_ast::ast::Name<'_, '_>,
    codebase: &Codebase,
    file: &Arc<str>,
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
    issues: &mut Vec<mir_issues::Issue>,
) {
    let name_str = crate::parser::name_to_string(name);
    let resolved = codebase.resolve_class_name(file.as_ref(), &name_str);
    if !codebase.type_exists(&resolved) {
        let span = name.span();
        let (line, col_start) = offset_to_line_col(source, span.start, source_map);
        let (_, col_end) = offset_to_line_col(source, span.end, source_map);
        issues.push(
            mir_issues::Issue::new(
                mir_issues::IssueKind::UndefinedClass { name: resolved },
                mir_issues::Location {
                    file: file.clone(),
                    line,
                    col_start,
                    col_end: col_end.max(col_start + 1),
                },
            )
            .with_snippet(crate::parser::span_text(source, span).unwrap_or_default()),
        );
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
            issues.push(
                mir_issues::Issue::new(
                    mir_issues::IssueKind::UnusedParam {
                        name: name.to_string(),
                    },
                    mir_issues::Location {
                        file: file.clone(),
                        line: 1,
                        col_start: 0,
                        col_end: 0,
                    },
                )
                .with_snippet(format!("${}", name)),
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
            issues.push(mir_issues::Issue::new(
                mir_issues::IssueKind::UnusedVariable { name: name.clone() },
                mir_issues::Location {
                    file: file.clone(),
                    line: 1,
                    col_start: 0,
                    col_end: 0,
                },
            ));
        }
    }
}
