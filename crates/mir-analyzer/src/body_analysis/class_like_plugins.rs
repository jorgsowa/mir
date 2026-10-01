use super::plugin_hooks::plugin_issue_to_issue;
use super::*;

/// What `AfterClassLikeAnalysis` plugins produced for one file.
#[derive(Default)]
pub(crate) struct ClassLikePluginOutput {
    pub issues: Vec<Issue>,
    pub suppressions: Vec<ClassSuppression>,
    pub ref_locs: Vec<crate::db::RefLoc>,
}

/// Issue names a plugin suppressed inside a class-like's line range.
pub(crate) struct ClassSuppression {
    pub file: Arc<str>,
    pub line: u32,
    pub line_end: u32,
    pub names: Vec<String>,
}

impl ClassSuppression {
    pub(crate) fn applies_to(&self, issue: &Issue) -> bool {
        issue.location.file == self.file
            && issue.location.line >= self.line
            && issue.location.line <= self.line_end
            && self.names.iter().any(|n| n == issue.kind.display_name())
    }
}

impl<'a> BodyAnalyzer<'a> {
    /// Dispatch `AfterClassLikeAnalysis` for every class-like declared in
    /// `program`.
    pub(crate) fn run_after_class_like_plugins(
        &self,
        program: &php_ast::owned::Program,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
    ) -> ClassLikePluginOutput {
        use php_ast::owned::StmtKind;

        let mut out = ClassLikePluginOutput::default();
        let Some(plugins) = mir_plugin::snapshot() else {
            return out;
        };
        if !plugins.hooks().after_class_like_analysis {
            return out;
        }

        let mut decls: Vec<(String, u32, php_ast::Span)> = Vec::new();
        for_each_file_scope_decl(&program.stmts, &mut |stmt| {
            let named = match &stmt.kind {
                StmtKind::Class(d) => d
                    .name
                    .as_ref()
                    .and_then(|i| i.as_deref())
                    .map(|n| (n, d.body.span.start)),
                StmtKind::Interface(d) => d.name.as_deref().map(|n| (n, d.body.span.start)),
                StmtKind::Trait(d) => d.name.as_deref().map(|n| (n, d.body.span.start)),
                StmtKind::Enum(d) => d.name.as_deref().map(|n| (n, d.body.span.start)),
                _ => None,
            };
            if let Some((name, body_start)) = named {
                decls.push((name.to_string(), body_start, stmt.span));
            }
        });

        for (name, body_start, span) in decls {
            let fqcn = declared_class_like_fqcn(
                self.db,
                file.as_ref(),
                &name,
                body_start,
                source,
                source_map,
            );
            let mut event = mir_plugin::AfterClassLikeAnalysisEvent {
                fqcn: &fqcn,
                file: file.as_ref(),
                span,
                issues: Vec::new(),
                suppressed_issues: Vec::new(),
                used_classes: Vec::new(),
                used_methods: Vec::new(),
            };
            plugins.after_class_like_analysis(&mut event);

            let (line, col_start) =
                crate::diagnostics::offset_to_line_col(source, span.start, source_map);
            let (line_end, _) =
                crate::diagnostics::offset_to_line_col(source, span.end, source_map);
            let ref_loc = |symbol_key: String| crate::db::RefLoc {
                symbol_key: Arc::from(symbol_key),
                file: file.clone(),
                line,
                col_start,
                col_end: col_start.saturating_add(1),
            };
            for class in &event.used_classes {
                out.ref_locs.push(ref_loc(format!("cls:{class}")));
            }
            for (class, method) in &event.used_methods {
                out.ref_locs.push(ref_loc(format!(
                    "meth:{class}::{}",
                    crate::util::php_ident_lowercase(method)
                )));
            }
            if !event.suppressed_issues.is_empty() {
                out.suppressions.push(ClassSuppression {
                    file: file.clone(),
                    line,
                    line_end,
                    names: std::mem::take(&mut event.suppressed_issues),
                });
            }
            for pi in std::mem::take(&mut event.issues) {
                out.issues
                    .push(plugin_issue_to_issue(pi, span, file, source, source_map));
            }
        }
        out
    }
}
