use std::sync::Arc;

use mir_codebase::storage::{FnParam, FunctionStorage, TemplateParam};
use mir_types::Union;

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

        let mut params = Vec::new();
        for p in decl.params.iter() {
            let ty = doc
                .get_param_type(p.name)
                .cloned()
                .map(|u| self.resolve_union_doc(u))
                .or_else(|| {
                    self.resolve_union_opt(p.type_hint.as_ref().map(|h| type_from_hint(h, None)))
                });
            params.push(FnParam {
                name: p.name.into(),
                ty: mir_codebase::wrap_param_type(ty),
                default: p.default.as_ref().map(|_| Union::mixed()),
                is_variadic: p.variadic,
                is_byref: p.by_ref,
                is_optional: p.default.is_some() || p.variadic,
            });
        }

        let return_type = match (doc.return_type.clone(), decl.return_type.as_ref()) {
            (Some(mut ty), _) => {
                ty.from_docblock = true;
                Some(self.resolve_union_doc(ty))
            }
            (None, Some(h)) => self.resolve_union_opt(Some(type_from_hint(h, None))),
            (None, None) => None,
        };

        let template_params = doc
            .templates
            .iter()
            .map(|(name, bound, variance)| TemplateParam {
                name: name.as_str().into(),
                bound: bound.clone(),
                defining_entity: fqn.as_str().into(),
                variance: *variance,
            })
            .collect();

        let storage = FunctionStorage {
            fqn: fqn.clone().into(),
            short_name: short_name.into(),
            params,
            return_type,
            inferred_return_type: None,
            template_params,
            assertions: self.build_assertions(&doc),
            throws: doc.throws.iter().map(|t| Arc::from(t.as_str())).collect(),
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
