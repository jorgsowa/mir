use super::DefinitionCollector;
use crate::parser::name_to_string;
use mir_codebase::storage::{ConstantStorage, InterfaceStorage, TemplateParam};
use mir_types::Union;
use php_ast::ast::{ClassMemberKind, InterfaceDecl};
use std::ops::ControlFlow;
use std::sync::Arc;

impl<'a> DefinitionCollector<'a> {
    pub(super) fn collect_interface<'arena, 'src>(
        &mut self,
        decl: &InterfaceDecl<'arena, 'src>,
        stmt_span: php_ast::Span,
    ) -> ControlFlow<()> {
        let interface_name = decl.name.to_string();
        let fqcn = self.resolve_name(&interface_name);

        let iface_doc = decl
            .doc_comment
            .as_ref()
            .map(|c| crate::parser::DocblockParser::parse(c.text))
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

        let template_params: Vec<TemplateParam> = iface_doc
            .templates
            .iter()
            .map(|(name, bound, variance)| TemplateParam {
                name: name.as_str().into(),
                bound: bound.clone(),
                defining_entity: fqcn.as_str().into(),
                variance: *variance,
            })
            .collect();

        let extends: Vec<Arc<str>> = decl
            .extends
            .iter()
            .map(|n| self.resolve_name(&name_to_string(n)).into())
            .collect();

        let mut own_methods = indexmap::IndexMap::new();
        let mut own_constants = indexmap::IndexMap::new();

        for member in decl.members.iter() {
            match &member.kind {
                ClassMemberKind::Method(m) => {
                    if let Some(method) =
                        self.build_method_storage(m, &fqcn, Some(&member.span), None)
                    {
                        own_methods.insert(
                            Arc::from(method.name.to_lowercase().as_str()),
                            Arc::new(method),
                        );
                    }
                }
                ClassMemberKind::ClassConst(c) => {
                    let const_doc = self.parse_docblock_from_node_or_preceding(
                        c.doc_comment.as_ref(),
                        member.span.start,
                    );
                    let const_doc_span = c
                        .doc_comment
                        .as_ref()
                        .map(|c| c.span.start)
                        .unwrap_or(member.span.start);
                    self.emit_docblock_issues(&const_doc, const_doc_span);
                    if !self.version_allows(&const_doc) {
                        continue;
                    }
                    own_constants.insert(
                        Arc::from(c.name.to_string()),
                        ConstantStorage {
                            name: Arc::from(c.name.to_string()),
                            ty: Union::mixed(),
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

        self.slice.interfaces.push(InterfaceStorage {
            fqcn: fqcn.into(),
            short_name: Arc::from(decl.name.to_string()),
            extends,
            own_methods,
            own_constants,
            template_params,
            location: Some(self.location(stmt_span.start, stmt_span.end)),
        });

        ControlFlow::Continue(())
    }
}
