use std::sync::Arc;

use mir_codebase::storage::{wrap_return_type, FnParam, FunctionStorage, TemplateParam};
use mir_types::Symbol;

use super::DefinitionCollector;
use crate::parser::type_from_hint_owned;

impl DefinitionCollector<'_> {
    pub(super) fn collect_function(
        &mut self,
        decl: &php_ast::owned::FunctionDecl,
        stmt_span: php_ast::Span,
    ) {
        let short_name = decl.name.as_deref().unwrap_or_default().to_string();
        let fqn = if let Some(ns) = &self.namespace {
            format!("{ns}\\{short_name}")
        } else {
            short_name.clone()
        };

        let doc =
            self.parse_docblock_from_node_or_preceding(decl.doc_comment.as_ref(), stmt_span.start);
        let doc_span = decl
            .doc_comment
            .as_ref()
            .map(|c| c.span.start)
            .unwrap_or(stmt_span.start);
        self.emit_docblock_issues(&doc, doc_span);

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
        let mut local_scalar = 0usize;
        let mut local_complex = 0usize;
        let mut local_defaults = 0usize;
        for p in decl.params.iter() {
            let param_name = p.name.as_deref().unwrap_or_default();
            let ty = doc
                .get_param_type(param_name)
                .cloned()
                .map(|u| {
                    // If the type is a simple named object that matches a template param,
                    // convert it to a TTemplateParam
                    self.resolve_union_doc_with_templates(
                        u,
                        &template_names,
                        &fqn,
                        &template_params,
                    )
                })
                .or_else(|| {
                    self.resolve_union_opt(
                        p.type_hint.as_ref().map(|h| type_from_hint_owned(h, None)),
                    )
                });
            if let Some(ty_ref) = &ty {
                if super::is_simple_scalar(ty_ref) {
                    local_scalar += 1;
                } else {
                    local_complex += 1;
                }
            }
            let has_default = p.default.is_some();
            if has_default {
                local_defaults += 1;
            }

            params.push(FnParam {
                name: Symbol::new(param_name),
                ty: mir_codebase::wrap_param_type(ty),
                has_default,
                is_variadic: p.variadic,
                is_byref: p.by_ref,
                is_optional: has_default || p.variadic,
            });
        }
        if local_scalar > 0 {
            super::SCALAR_PARAM_COUNT.fetch_add(local_scalar, std::sync::atomic::Ordering::Relaxed);
        }
        if local_complex > 0 {
            super::COMPLEX_PARAM_COUNT
                .fetch_add(local_complex, std::sync::atomic::Ordering::Relaxed);
        }
        if local_defaults > 0 {
            super::PARAM_WITH_DEFAULT
                .fetch_add(local_defaults, std::sync::atomic::Ordering::Relaxed);
        }

        let return_type = match (doc.return_type.clone(), decl.return_type.as_ref()) {
            (Some(mut ty), _) => {
                ty.from_docblock = true;
                Some(self.resolve_union_doc_with_templates(
                    ty,
                    &template_names,
                    &fqn,
                    &template_params,
                ))
            }
            (None, Some(h)) => self.resolve_union_opt(Some(type_from_hint_owned(h, None))),
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

        let docstring = if doc.description.trim().is_empty() {
            None
        } else {
            Some(Arc::from(doc.description.as_str()))
        };

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
            docstring,
        };

        self.slice.functions.push(std::sync::Arc::new(storage));

        // Scan the function body for `@var`-annotated global declarations.
        self.scan_stmts_for_global_vars(&decl.body);
    }

    pub(super) fn collect_global_stmt(&mut self, stmt: &php_ast::owned::Stmt) {
        // Top-level `global $x` — unusual in PHP but valid.
        self.try_collect_global_var_annotation(stmt);
    }
}
