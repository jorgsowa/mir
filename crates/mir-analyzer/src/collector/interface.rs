use super::DefinitionCollector;
use crate::parser::name_to_string_owned;
use mir_codebase::storage::{ConstantDef, InterfaceDef, TemplateParam};
use mir_types::Type;
use php_ast::owned::{ClassMemberKind, InterfaceDecl};
use std::ops::ControlFlow;
use std::sync::Arc;

impl<'a> DefinitionCollector<'a> {
    pub(super) fn collect_interface(
        &mut self,
        decl: &InterfaceDecl,
        stmt_span: php_ast::Span,
    ) -> ControlFlow<()> {
        let interface_name = decl.name.as_deref().unwrap_or_default().to_string();
        let fqcn = self.resolve_name(&interface_name);

        let iface_doc = decl
            .doc_comment
            .as_ref()
            .map(|c| crate::parser::DocblockParser::parse(&c.text))
            .unwrap_or_default();

        let iface_doc_span = decl
            .doc_comment
            .as_ref()
            .map(|c| c.span.start)
            .unwrap_or(stmt_span.start);
        self.emit_docblock_issues(&iface_doc, iface_doc_span);

        if !self.version_allows(&iface_doc) {
            return ControlFlow::Continue(());
        }

        let iface_template_names: std::collections::HashSet<String> = iface_doc
            .templates
            .iter()
            .map(|(n, _, _)| n.to_string())
            .collect();
        let template_params: Vec<TemplateParam> = iface_doc
            .templates
            .iter()
            .map(|(name, bound, variance)| TemplateParam {
                name: name.as_str().into(),
                bound: bound.clone().map(|b| {
                    self.resolve_union_doc_with_templates(
                        b,
                        &iface_template_names,
                        fqcn.as_str(),
                        &[],
                    )
                }),
                defining_entity: fqcn.as_str().into(),
                variance: *variance,
            })
            .collect();

        // Build interface-level template params before the member loop so methods referencing
        // interface templates in their return types don't get them wrongly namespace-qualified.
        let iface_template_params = template_params.clone();

        let extends: Vec<Arc<str>> = decl
            .extends
            .iter()
            .map(|n| self.resolve_name(&name_to_string_owned(n)).into())
            .collect();

        let mut own_methods = indexmap::IndexMap::new();
        let mut own_constants = indexmap::IndexMap::new();

        for member in decl.body.members.iter() {
            match &member.kind {
                ClassMemberKind::Method(m) => {
                    if let Some(method) = self.build_method_storage(
                        m,
                        &fqcn,
                        Some(&member.span),
                        None,
                        &iface_template_params,
                    ) {
                        own_methods.insert(
                            Arc::from(method.name.to_lowercase().as_str()),
                            Arc::new(method),
                        );
                    }
                }
                ClassMemberKind::ClassConst(c) => {
                    let const_doc = self.parse_docblock_from_node(c.doc_comment.as_ref());
                    let const_doc_span = c
                        .doc_comment
                        .as_ref()
                        .map(|c| c.span.start)
                        .unwrap_or(member.span.start);
                    self.emit_docblock_issues(&const_doc, const_doc_span);
                    if !self.version_allows(&const_doc) {
                        continue;
                    }
                    let const_name = c.name.as_deref().unwrap_or_default();
                    own_constants.insert(
                        Arc::from(const_name),
                        ConstantDef {
                            name: Arc::from(const_name),
                            ty: Type::mixed(),
                            visibility: c.visibility.map(|v| Self::convert_visibility(Some(v))),
                            is_final: c.is_final,
                            location: Some(self.location(member.span.start, member.span.end)),
                        },
                    );
                }
                _ => {}
            }
        }

        let type_aliases = self.build_type_aliases(&iface_doc);
        let mut dummy_properties = indexmap::IndexMap::new();
        self.add_docblock_members(
            &iface_doc,
            &type_aliases,
            &fqcn,
            &mut own_methods,
            &mut dummy_properties,
            Some(self.location(stmt_span.start, stmt_span.end)),
        );

        self.slice
            .interfaces
            .push(std::sync::Arc::new(InterfaceDef {
                fqcn: fqcn.into(),
                short_name: Arc::from(interface_name.as_str()),
                extends,
                own_methods,
                own_constants,
                template_params,
                location: Some(self.location(stmt_span.start, stmt_span.end)),
            }));

        ControlFlow::Continue(())
    }
}
