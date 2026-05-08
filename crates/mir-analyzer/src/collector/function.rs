use std::sync::Arc;

use mir_codebase::storage::{wrap_return_type, FnParam, FunctionStorage, TemplateParam};

use super::DefinitionCollector;
use crate::parser::type_from_hint;

impl DefinitionCollector<'_> {
    pub(super) fn collect_function<'arena, 'src>(
        &mut self,
        decl: &php_ast::ast::FunctionDecl<'arena, 'src>,
        stmt_span: php_ast::Span,
    ) {
        let short_name = decl.name.to_string();
        let fqn = if let Some(ns) = &self.namespace {
            format!("{ns}\\{short_name}")
        } else {
            short_name.clone()
        };

        let doc = decl
            .doc_comment
            .as_ref()
            .map(|c| crate::parser::DocblockParser::parse(c.text))
            .unwrap_or_default();

        if let Some(c) = decl.doc_comment.as_ref() {
            self.emit_docblock_issues(&doc, c.span.start);
        }

        if !self.version_allows(&doc) {
            return;
        }

        // Extract template parameters first so they're available during type resolution
        let template_params = doc
            .templates
            .iter()
            .map(|(name, bound, variance)| TemplateParam {
                name: name.as_str().into(),
                bound: bound.clone(),
                defining_entity: fqn.as_str().into(),
                variance: *variance,
            })
            .collect::<Vec<_>>();

        // Build a set of template names for use during param type resolution
        let template_names: std::collections::HashSet<String> = doc
            .templates
            .iter()
            .map(|(n, _, _)| n.to_string())
            .collect();

        let mut params = Vec::new();
        for p in decl.params.iter() {
            let ty = doc
                .get_param_type(&p.name.to_string())
                .cloned()
                .map(|u| {
                    // If the type is a simple named object that matches a template param,
                    // convert it to a TTemplateParam
                    self.resolve_union_doc_with_templates(u, &template_names, &fqn)
                })
                .or_else(|| {
                    self.resolve_union_opt(p.type_hint.as_ref().map(|h| type_from_hint(h, None)))
                });
            // Profiling: track scalar vs complex param types
            if let Some(ty_ref) = &ty {
                if super::is_simple_scalar(ty_ref) {
                    super::SCALAR_PARAM_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                } else {
                    super::COMPLEX_PARAM_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
            let has_default = p.default.is_some();
            if has_default {
                super::PARAM_WITH_DEFAULT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }

            params.push(FnParam {
                name: Arc::from(p.name.to_string()),
                ty: mir_codebase::wrap_param_type(ty),
                has_default,
                is_variadic: p.variadic,
                is_byref: p.by_ref,
                is_optional: has_default || p.variadic,
            });
        }

        let return_type = match (doc.return_type.clone(), decl.return_type.as_ref()) {
            (Some(mut ty), _) => {
                ty.from_docblock = true;
                Some(self.resolve_union_doc_with_templates(ty, &template_names, &fqn))
            }
            (None, Some(h)) => self.resolve_union_opt(Some(type_from_hint(h, None))),
            (None, None) => None,
        };

        let throws = doc
            .throws
            .iter()
            .map(|t| {
                Arc::from(
                    super::resolution::resolve_name(t, &self.namespace, &self.use_aliases).as_str(),
                )
            })
            .collect();

        let storage = FunctionStorage {
            fqn: fqn.clone().into(),
            short_name: short_name.into(),
            params: Arc::from(params.into_boxed_slice()),
            return_type: wrap_return_type(return_type),
            inferred_return_type: None,
            template_params,
            assertions: self.build_assertions(&doc),
            throws,
            deprecated: doc.deprecated.as_deref().map(Arc::from),
            is_pure: doc.is_pure,
            location: Some(self.location(stmt_span.start, stmt_span.end)),
        };

        self.slice.functions.push(storage);

        // Scan the function body for `@var`-annotated global declarations.
        self.scan_stmts_for_global_vars(&decl.body);
    }

    pub(super) fn collect_global_stmt<'arena, 'src>(
        &mut self,
        stmt: &php_ast::ast::Stmt<'arena, 'src>,
    ) {
        // Top-level `global $x` — unusual in PHP but valid.
        self.try_collect_global_var_annotation(stmt);
    }
}
