use super::DefinitionCollector;
use crate::parser::{name_to_string, type_from_hint};
use mir_codebase::storage::{ConstantStorage, PropertyStorage, TemplateParam, TraitStorage};
use mir_types::Union;
use php_ast::ast::{ClassMemberKind, TraitDecl};
use std::ops::ControlFlow;
use std::sync::Arc;

impl<'a> DefinitionCollector<'a> {
    pub(super) fn collect_trait<'arena, 'src>(
        &mut self,
        decl: &TraitDecl<'arena, 'src>,
        stmt_span: php_ast::Span,
    ) -> ControlFlow<()> {
        let trait_name = decl.name.to_string();
        let fqcn = self.resolve_name(&trait_name);

        let trait_doc = decl
            .doc_comment
            .as_ref()
            .map(|c| crate::parser::DocblockParser::parse(c.text))
            .unwrap_or_default();

        let trait_doc_span = decl
            .doc_comment
            .as_ref()
            .map(|c| c.span.start)
            .unwrap_or(stmt_span.start);
        self.emit_docblock_issues(&trait_doc, trait_doc_span);

        if !self.version_allows(&trait_doc) {
            return ControlFlow::Continue(());
        }

        let trait_template_params: Vec<TemplateParam> = trait_doc
            .templates
            .iter()
            .map(|(name, bound, variance)| TemplateParam {
                name: name.as_str().into(),
                bound: bound.clone(),
                defining_entity: fqcn.as_str().into(),
                variance: *variance,
            })
            .collect();

        let mut own_methods = indexmap::IndexMap::new();
        let mut own_properties = indexmap::IndexMap::new();
        let mut own_constants = indexmap::IndexMap::new();
        let mut trait_uses: Vec<Arc<str>> = vec![];

        for member in decl.members.iter() {
            match &member.kind {
                ClassMemberKind::Method(m) => {
                    if m.name == "__construct" {
                        for p in m.params.iter() {
                            if p.visibility.is_some() {
                                let ty = self.resolve_union_opt(
                                    p.type_hint.as_ref().map(|h| type_from_hint(h, Some(&fqcn))),
                                );
                                let prop = PropertyStorage {
                                    name: Arc::from(p.name.to_string()),
                                    ty,
                                    inferred_ty: None,
                                    visibility: Self::convert_visibility(p.visibility),
                                    is_static: false,
                                    is_readonly: p.is_readonly,
                                    default: p.default.as_ref().map(|_| Union::mixed()),
                                    location: Some(
                                        self.location(member.span.start, member.span.end),
                                    ),
                                };
                                own_properties.insert(Arc::from(p.name.to_string()), prop);
                            }
                        }
                    }
                    if let Some(method) =
                        self.build_method_storage(m, &fqcn, Some(&member.span), None)
                    {
                        own_methods.insert(
                            Arc::from(method.name.to_lowercase().as_str()),
                            Arc::new(method),
                        );
                    }
                }
                ClassMemberKind::Property(p) => {
                    let prop_doc = self.parse_docblock_from_node_or_preceding(
                        p.doc_comment.as_ref(),
                        member.span.start,
                    );
                    let prop_doc_span = p
                        .doc_comment
                        .as_ref()
                        .map(|c| c.span.start)
                        .unwrap_or(member.span.start);
                    self.emit_docblock_issues(&prop_doc, prop_doc_span);
                    if !self.version_allows(&prop_doc) {
                        continue;
                    }
                    own_properties.insert(
                        Arc::from(p.name.to_string()),
                        PropertyStorage {
                            name: Arc::from(p.name.to_string()),
                            ty: self.resolve_union_opt(
                                p.type_hint.as_ref().map(|h| type_from_hint(h, Some(&fqcn))),
                            ),
                            inferred_ty: None,
                            visibility: Self::convert_visibility(p.visibility),
                            is_static: p.is_static,
                            is_readonly: p.is_readonly,
                            default: None,
                            location: Some(self.location(member.span.start, member.span.end)),
                        },
                    );
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
                            visibility: None,
                            is_final: c.is_final,
                            location: Some(self.location(member.span.start, member.span.end)),
                        },
                    );
                }
                ClassMemberKind::TraitUse(tu) => {
                    for t in tu.traits.iter() {
                        trait_uses.push(self.resolve_name(&name_to_string(t)).into());
                    }
                }
            }
        }

        let type_aliases = self.build_type_aliases(&trait_doc);
        self.add_docblock_members(
            &trait_doc,
            &type_aliases,
            &fqcn,
            &mut own_methods,
            &mut own_properties,
            Some(self.location(stmt_span.start, stmt_span.end)),
        );

        let require_extends: Vec<Arc<str>> = trait_doc
            .require_extends
            .iter()
            .map(|s| self.resolve_type_name(&Arc::from(s.as_str()), true))
            .collect();
        let require_implements: Vec<Arc<str>> = trait_doc
            .require_implements
            .iter()
            .map(|s| self.resolve_type_name(&Arc::from(s.as_str()), true))
            .collect();

        self.slice.traits.push(TraitStorage {
            fqcn: fqcn.into(),
            short_name: Arc::from(decl.name.to_string()),
            own_methods,
            own_properties,
            own_constants,
            template_params: trait_template_params,
            traits: trait_uses,
            location: Some(self.location(stmt_span.start, stmt_span.end)),
            require_extends,
            require_implements,
        });

        ControlFlow::Continue(())
    }
}
