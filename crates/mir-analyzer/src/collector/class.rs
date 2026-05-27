use super::DefinitionCollector;
use crate::parser::{name_to_string_owned, type_from_hint_owned};
use mir_codebase::storage::{ConstantDef, PropertyDef, TemplateParam};
use mir_codebase::ClassDef;
use mir_types::Atomic;
use php_ast::owned::{ClassDecl, ClassMemberKind};
use std::ops::ControlFlow;
use std::sync::Arc;

impl<'a> DefinitionCollector<'a> {
    pub(super) fn collect_class(
        &mut self,
        decl: &ClassDecl,
        stmt_span: php_ast::Span,
    ) -> ControlFlow<()> {
        let name = match decl.name.as_ref().and_then(|n| n.as_deref()) {
            Some(n) => n.to_string(),
            None => return ControlFlow::Continue(()), // anonymous class — handled at expression level
        };
        let fqcn = self.resolve_name(&name);
        let short_name = name;

        let parent = decl
            .extends
            .as_ref()
            .map(|n| self.resolve_name(&name_to_string_owned(n)).into());
        let interfaces: Vec<Arc<str>> = decl
            .implements
            .iter()
            .map(|n| self.resolve_name(&name_to_string_owned(n)).into())
            .collect();

        let mut own_methods = indexmap::IndexMap::new();
        let mut own_properties = indexmap::IndexMap::new();
        let mut own_constants = indexmap::IndexMap::new();
        let mut trait_uses: Vec<Arc<str>> = vec![];
        let mut trait_use_locations: Vec<(Arc<str>, mir_types::Location)> = vec![];

        let class_doc =
            self.parse_docblock_from_node_or_preceding(decl.doc_comment.as_ref(), stmt_span.start);

        let class_doc_span = decl
            .doc_comment
            .as_ref()
            .map(|c| c.span.start)
            .unwrap_or(stmt_span.start);
        self.emit_docblock_issues(&class_doc, class_doc_span);

        if !self.version_allows(&class_doc) {
            return ControlFlow::Continue(());
        }

        let type_aliases = self.build_type_aliases(&class_doc);

        for member in decl.members.iter() {
            match &member.kind {
                ClassMemberKind::Method(m) => {
                    if m.name.as_deref() == Some("__construct") {
                        for p in m.params.iter() {
                            if p.visibility.is_some() {
                                let param_name = p.name.as_deref().unwrap_or_default();
                                let ty = self.resolve_union_opt(
                                    p.type_hint
                                        .as_ref()
                                        .map(|h| type_from_hint_owned(h, Some(&fqcn))),
                                );
                                let prop = PropertyDef {
                                    name: Arc::from(param_name),
                                    ty,
                                    inferred_ty: None,
                                    visibility: Self::convert_visibility(p.visibility),
                                    is_static: false,
                                    is_readonly: decl.modifiers.is_readonly,
                                    default: p.default.as_ref().map(|_| mir_types::Type::mixed()),
                                    location: Some(
                                        self.location(member.span.start, member.span.end),
                                    ),
                                };
                                own_properties.insert(Arc::from(param_name), prop);
                            }
                        }
                    }
                    if let Some(method) =
                        self.build_method_storage(m, &fqcn, Some(&member.span), Some(&type_aliases))
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
                    let prop_name = p.name.as_deref().unwrap_or_default();
                    let hint_ty = self.resolve_union_opt(
                        p.type_hint
                            .as_ref()
                            .map(|h| type_from_hint_owned(h, Some(&fqcn))),
                    );
                    // @var docblock overrides the PHP type hint when present, allowing
                    // @var Box<string> to refine a bare Box declaration with type params.
                    let ty = prop_doc.var_type.map(|t| self.resolve_union(t)).or(hint_ty);
                    let prop = PropertyDef {
                        name: Arc::from(prop_name),
                        ty,
                        inferred_ty: None,
                        visibility: Self::convert_visibility(p.visibility),
                        is_static: p.is_static,
                        is_readonly: p.is_readonly || decl.modifiers.is_readonly,
                        default: p.default.as_ref().map(|_| mir_types::Type::mixed()),
                        location: Some(self.location(member.span.start, member.span.end)),
                    };
                    own_properties.insert(Arc::from(prop_name), prop);
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
                    let const_name = c.name.as_deref().unwrap_or_default();
                    let constant = ConstantDef {
                        name: Arc::from(const_name),
                        ty: mir_types::Type::mixed(),
                        visibility: c.visibility.map(|v| Self::convert_visibility(Some(v))),
                        is_final: c.is_final,
                        location: Some(self.location(member.span.start, member.span.end)),
                    };
                    own_constants.insert(Arc::from(const_name), constant);
                }
                ClassMemberKind::TraitUse(tu) => {
                    for t in tu.traits.iter() {
                        let fqcn: Arc<str> = self.resolve_name(&name_to_string_owned(t)).into();
                        let loc = self.location(t.span.start, t.span.end);
                        trait_use_locations.push((fqcn.clone(), loc));
                        trait_uses.push(fqcn);
                    }
                }
            }
        }

        self.add_docblock_members(
            &class_doc,
            &type_aliases,
            &fqcn,
            &mut own_methods,
            &mut own_properties,
            Some(self.location(stmt_span.start, stmt_span.end)),
        );

        let template_params: Vec<TemplateParam> = class_doc
            .templates
            .iter()
            .map(|(name, bound, variance)| TemplateParam {
                name: name.as_str().into(),
                bound: self.resolve_union_opt(bound.clone()),
                defining_entity: fqcn.as_str().into(),
                variance: *variance,
            })
            .collect();

        let extends_type_args: Vec<mir_types::Type> = class_doc
            .extends
            .as_ref()
            .and_then(|ty| {
                if let Some(Atomic::TNamedObject { type_params, .. }) = ty.types.first() {
                    Some(
                        type_params
                            .iter()
                            .map(|tp| self.resolve_union(tp.clone()))
                            .collect(),
                    )
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let implements_type_args: Vec<(Arc<str>, Vec<mir_types::Type>)> = class_doc
            .implements
            .iter()
            .filter_map(|ty| {
                if let Some(Atomic::TNamedObject { fqcn, type_params }) = ty.types.first() {
                    Some((
                        self.resolve_type_name(fqcn.as_str(), true).into(),
                        type_params
                            .iter()
                            .map(|tp| self.resolve_union(tp.clone()))
                            .collect(),
                    ))
                } else {
                    None
                }
            })
            .collect();

        let storage = ClassDef {
            fqcn: fqcn.clone().into(),
            short_name: short_name.into(),
            parent,
            interfaces,
            traits: trait_uses,
            trait_use_locations,
            own_methods,
            own_properties,
            own_constants,
            mixins: class_doc
                .mixins
                .iter()
                .map(|m| self.resolve_type_name(m.as_str(), true).into())
                .collect(),
            template_params,
            extends_type_args,
            implements_type_args,
            is_abstract: decl.modifiers.is_abstract,
            is_final: decl.modifiers.is_final,
            is_readonly: decl.modifiers.is_readonly,
            deprecated: class_doc.deprecated.as_deref().map(Arc::from),
            is_internal: class_doc.is_internal,
            location: Some(self.location(stmt_span.start, stmt_span.end)),
            type_aliases: type_aliases
                .iter()
                .map(|(k, v)| (Arc::from(k.as_str()), v.clone()))
                .collect(),
            pending_import_types: class_doc
                .import_types
                .iter()
                .map(|imp| {
                    let from_resolved: Arc<str> =
                        self.resolve_type_name(imp.from_class.as_str(), true).into();
                    (
                        Arc::from(imp.local.as_str()),
                        Arc::from(imp.original.as_str()),
                        from_resolved,
                    )
                })
                .collect(),
        };

        self.slice.classes.push(std::sync::Arc::new(storage));
        ControlFlow::Continue(())
    }
}
