use super::DefinitionCollector;
use crate::parser::{name_to_string_owned, type_from_hint_owned};
use mir_codebase::definitions::{
    wrap_template_bound, ConstantDef, PropertyDef, TemplateParam, TraitDef,
};
use mir_types::Type;
use php_ast::owned::{ClassMemberKind, TraitDecl};
use std::ops::ControlFlow;
use std::sync::Arc;

impl<'a> DefinitionCollector<'a> {
    pub(super) fn collect_trait(
        &mut self,
        decl: &TraitDecl,
        stmt_span: php_ast::Span,
    ) -> ControlFlow<()> {
        let trait_name = decl.name.as_deref().unwrap_or_default().to_string();
        let fqcn = self.declared_fqn(&trait_name);

        let trait_doc = decl
            .doc_comment
            .as_ref()
            .map(|c| crate::parser::DocblockParser::parse(&c.text))
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

        let trait_template_names: rustc_hash::FxHashSet<String> = trait_doc
            .templates
            .iter()
            .map(|(n, _, _, _)| n.to_string())
            .collect();
        let trait_template_params: Vec<TemplateParam> = trait_doc
            .templates
            .iter()
            .map(|(name, bound, variance, default)| TemplateParam {
                name: name.as_str().into(),
                bound: wrap_template_bound(bound.clone().map(|b| {
                    Self::fill_self_static_parent(
                        self.resolve_union_doc_with_templates(
                            b,
                            &trait_template_names,
                            fqcn.as_str(),
                            &[],
                        ),
                        fqcn.as_str(),
                    )
                })),
                default: wrap_template_bound(default.clone().map(|d| {
                    Self::fill_self_static_parent(
                        self.resolve_union_doc_with_templates(
                            d,
                            &trait_template_names,
                            fqcn.as_str(),
                            &[],
                        ),
                        fqcn.as_str(),
                    )
                })),
                defining_entity: fqcn.as_str().into(),
                variance: *variance,
            })
            .collect();

        let mut own_methods = mir_codebase::definitions::MemberMap::default();
        let mut own_properties = mir_codebase::definitions::MemberMap::default();
        let mut own_constants = mir_codebase::definitions::MemberMap::default();
        let mut trait_uses: Vec<Arc<str>> = vec![];
        let mut trait_use_locations: Vec<(Arc<str>, mir_types::Location)> = vec![];

        // See `class.rs` for why this runs before the loop and only covers
        // self/static references to this same trait's constants.
        let self_int_constants: Arc<rustc_hash::FxHashMap<Arc<str>, i64>> = Arc::new(
            decl.body
                .members
                .iter()
                .filter_map(|m| match &m.kind {
                    ClassMemberKind::ClassConst(c) => {
                        let name = c.name.as_deref()?;
                        match super::infer_const_value(&c.value.kind) {
                            Some(t) if t.types.len() == 1 => match &t.types[0] {
                                mir_types::Atomic::TLiteralInt(n) => Some((Arc::from(name), *n)),
                                _ => None,
                            },
                            _ => None,
                        }
                    }
                    _ => None,
                })
                .collect(),
        );
        let _int_mask_guard =
            crate::parser::docblock::SelfIntConstantsGuard::activate(&fqcn, &self_int_constants);

        for member in decl.body.members.iter() {
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
                                    ty: mir_codebase::definitions::wrap_property_type(ty.clone()),
                                    inferred_ty: None,
                                    native_ty: mir_codebase::definitions::wrap_property_type(ty),
                                    visibility: Self::convert_visibility(p.visibility),
                                    is_static: false,
                                    is_readonly: p.is_readonly,
                                    has_native_readonly: p.is_readonly,
                                    default: mir_codebase::definitions::wrap_property_type(
                                        p.default.as_ref().map(|_| Type::mixed()),
                                    ),
                                    location: Some(
                                        self.location(member.span.start, member.span.end),
                                    ),
                                    deprecated: None,
                                    has_native_type: p.type_hint.is_some(),
                                    from_docblock: false,
                                };
                                own_properties.insert(Arc::from(param_name), prop);
                            }
                        }
                    }
                    if let Some(method) = self.build_method_storage(
                        m,
                        &fqcn,
                        Some(&member.span),
                        None,
                        &trait_template_params,
                    ) {
                        own_methods.insert(
                            Arc::from(crate::util::php_ident_lowercase(&method.name).as_str()),
                            Arc::new(method),
                        );
                    }
                }
                ClassMemberKind::Property(p) => {
                    let prop_doc = self.parse_docblock_from_node(p.doc_comment.as_ref());
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
                    // phpstorm-stubs `#[LanguageLevelTypeAware]` wins, then an `@var`
                    // docblock (mirroring class.rs's precedence — a trait property's
                    // @var was previously ignored entirely, only the native hint was
                    // ever used), then the native hint.
                    let ty = self
                        .version_attr_type_string(&p.attributes)
                        .map(|s| crate::parser::docblock::parse_type_string(&s))
                        .or_else(|| {
                            prop_doc.var_type.clone().map(|t| {
                                self.resolve_union_doc_with_templates(
                                    t,
                                    &trait_template_names,
                                    fqcn.as_str(),
                                    &trait_template_params,
                                )
                            })
                        })
                        .or_else(|| hint_ty.clone());
                    own_properties.insert(
                        Arc::from(prop_name),
                        PropertyDef {
                            name: Arc::from(prop_name),
                            ty: mir_codebase::definitions::wrap_property_type(ty),
                            native_ty: mir_codebase::definitions::wrap_property_type(hint_ty),
                            inferred_ty: None,
                            visibility: Self::convert_visibility(p.visibility),
                            is_static: p.is_static,
                            is_readonly: p.is_readonly,
                            has_native_readonly: p.is_readonly,
                            default: None,
                            location: Some(self.location(member.span.start, member.span.end)),
                            deprecated: prop_doc.deprecated.as_deref().map(Arc::from).or_else(|| {
                                if p.attributes.iter().any(|a| {
                                    a.name
                                        .parts
                                        .last()
                                        .map(|part| part.as_ref().eq_ignore_ascii_case("Deprecated"))
                                        .unwrap_or(false)
                                }) {
                                    Some(Arc::from(""))
                                } else {
                                    None
                                }
                            }),
                            has_native_type: p.type_hint.is_some(),
                            from_docblock: false,
                        },
                    );
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
                            visibility: None,
                            is_final: c.is_final,
                            location: Some(self.location(member.span.start, member.span.end)),
                            deprecated: const_doc.deprecated.as_deref().map(Arc::from).or_else(|| {
                                if c.attributes.iter().any(|a| {
                                    a.name
                                        .parts
                                        .last()
                                        .map(|p| p.as_ref().eq_ignore_ascii_case("Deprecated"))
                                        .unwrap_or(false)
                                }) {
                                    Some(Arc::from(""))
                                } else {
                                    None
                                }
                            }),
                        },
                    );
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

        let type_aliases = self.build_type_aliases(&trait_doc);
        self.add_docblock_members(
            &trait_doc,
            &type_aliases,
            &fqcn,
            &mut own_methods,
            &mut own_properties,
            Some(self.location(stmt_span.start, stmt_span.end)),
            &trait_template_names,
            &trait_template_params,
        );

        let require_extends: Vec<Arc<str>> = trait_doc
            .require_extends
            .iter()
            .map(|s| self.resolve_type_name(s.as_str(), true).into())
            .collect();
        let require_implements: Vec<Arc<str>> = trait_doc
            .require_implements
            .iter()
            .map(|s| self.resolve_type_name(s.as_str(), true).into())
            .collect();

        self.slice.traits.push(std::sync::Arc::new(TraitDef {
            fqcn: fqcn.into(),
            short_name: Arc::from(trait_name.as_str()),
            own_methods,
            own_properties,
            own_constants,
            template_params: trait_template_params,
            traits: trait_uses,
            location: Some(self.location(stmt_span.start, stmt_span.end)),
            trait_use_locations,
            require_extends,
            require_implements,
            deprecated: Self::deprecated_from_doc_or_attrs(
                trait_doc.deprecated.as_deref(),
                &decl.attributes,
            ),
            type_aliases: type_aliases
                .iter()
                .map(|(k, v)| (Arc::from(k.as_str()), v.clone()))
                .collect(),
        }));

        ControlFlow::Continue(())
    }
}
