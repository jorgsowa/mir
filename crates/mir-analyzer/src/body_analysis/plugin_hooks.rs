use super::*;
use mir_plugin::{AttributeArgInfo, AttributeInfo, FunctionLikeParamInfo};

pub(super) struct FunctionLikeSite<'s> {
    pub name: &'s str,
    /// Declaring class FQCN for methods.
    pub class: Option<&'s str>,
    pub params: &'s [php_ast::owned::Param],
    /// Looks up storage params; only invoked when a plugin subscribes.
    pub declared: &'s dyn Fn() -> Option<Arc<[mir_codebase::DeclaredParam]>>,
    pub span: php_ast::Span,
}

impl<'a> BodyAnalyzer<'a> {
    /// Dispatch `AfterFunctionLikeAnalysis` to plugins and turn the issues
    /// they raise into diagnostics.
    pub(super) fn run_after_function_like_plugins(
        &self,
        site: &FunctionLikeSite<'_>,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
    ) {
        let Some(plugins) = mir_plugin::snapshot() else {
            return;
        };
        if !plugins.hooks().after_function_like_analysis {
            return;
        }

        let declared = (site.declared)();
        let params: Vec<FunctionLikeParamInfo> = site
            .params
            .iter()
            .map(|p| self.function_like_param_info(p, site, declared.as_deref(), file))
            .collect();
        let snippet =
            source.get(site.span.start as usize..(site.span.end as usize).min(source.len()));
        let mut event = mir_plugin::AfterFunctionLikeAnalysisEvent {
            name: site.name,
            class: site.class,
            params: &params,
            span: site.span,
            snippet,
            file: file.as_ref(),
            issues: Vec::new(),
        };
        plugins.after_function_like_analysis(&mut event);
        let issues = std::mem::take(&mut event.issues);
        drop(event);

        if self.mode != AnalysisMode::Full {
            return;
        }
        for pi in issues {
            all_issues.push(plugin_issue_to_issue(
                pi, site.span, file, source, source_map,
            ));
        }
    }

    fn function_like_param_info(
        &self,
        param: &php_ast::owned::Param,
        site: &FunctionLikeSite<'_>,
        declared: Option<&[mir_codebase::DeclaredParam]>,
        file: &Arc<str>,
    ) -> FunctionLikeParamInfo {
        let name = param.name.as_deref().unwrap_or("").to_string();
        let declared_type = declared
            .and_then(|d| d.iter().find(|p| p.name.as_ref() == name))
            .and_then(|p| p.ty.as_ref())
            .map(|t| t.to_string());
        let attributes = param
            .attributes
            .iter()
            .map(|attr| AttributeInfo {
                fq_class_name: crate::attributes::resolve_attr_name(self.db, file.as_ref(), attr)
                    .trim_start_matches('\\')
                    .to_string(),
                args: attr
                    .args
                    .iter()
                    .map(|arg| AttributeArgInfo {
                        name: arg.name.as_ref().map(|n| {
                            n.parts
                                .iter()
                                .map(|p| p.as_ref())
                                .collect::<Vec<_>>()
                                .join("\\")
                        }),
                        type_string: arg
                            .value
                            .as_ref()
                            .and_then(|v| self.constant_expr_type(v, file, site.class)),
                    })
                    .collect(),
                span: attr.span,
            })
            .collect();
        FunctionLikeParamInfo {
            name,
            declared_type,
            attributes,
        }
    }

    /// Docblock-syntax type of a constant expression: literals, and class
    /// constants resolved through the codebase.
    fn constant_expr_type(
        &self,
        expr: &php_ast::owned::Expr,
        file: &Arc<str>,
        class: Option<&str>,
    ) -> Option<String> {
        use php_ast::owned::ExprKind;
        match &expr.kind {
            ExprKind::String(s) => Some(format!(
                "'{}'",
                s.replace('\\', "\\\\").replace('\'', "\\'")
            )),
            ExprKind::Int(i) => Some(i.to_string()),
            ExprKind::Bool(b) => Some(b.to_string()),
            ExprKind::Null => Some("null".to_string()),
            ExprKind::ClassConstAccess(cca) => {
                let ExprKind::Identifier(class_name) = &cca.class.kind else {
                    return None;
                };
                let ExprKind::Identifier(member) = &cca.member.kind else {
                    return None;
                };
                let resolved = match class_name.as_ref() {
                    "self" | "static" => class?.to_string(),
                    other => resolve_name(self.db, file.as_ref(), other),
                };
                if member.as_ref() == "class" {
                    return Some(format!("class-string<{resolved}>"));
                }
                let (_, constant) = crate::db::find_class_constant_in_chain(
                    self.db,
                    crate::db::Fqcn::from_str(self.db, &resolved),
                    member.as_ref(),
                )?;
                Some(constant.ty.to_string())
            }
            _ => None,
        }
    }
}

/// Convert a plugin-raised issue into a diagnostic at its span (or
/// `default_span` when the plugin gave none).
pub(super) fn plugin_issue_to_issue(
    pi: mir_plugin::PluginIssue,
    default_span: php_ast::Span,
    file: &Arc<str>,
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
) -> Issue {
    let span = pi.span.unwrap_or(default_span);
    let (line, col_start) = crate::diagnostics::offset_to_line_col(source, span.start, source_map);
    let (line_end, col_end) = crate::diagnostics::offset_to_line_col(source, span.end, source_map);
    let mut issue = Issue::new(
        mir_issues::IssueKind::PluginIssue {
            name: pi.name,
            message: pi.message,
        },
        mir_issues::Location {
            file: file.clone(),
            line,
            line_end,
            col_start,
            col_end: crate::diagnostics::clamp_col_end(line, line_end, col_start, col_end),
        },
    );
    issue.severity = pi.severity;
    if let Some(text) = crate::parser::span_text(source, span) {
        issue.snippet = Some(text);
    }
    issue
}
