use super::DefinitionCollector;
use crate::parser::{name_to_string_owned, type_from_hint_owned};
use mir_codebase::definitions::{
    wrap_template_bound, ConstantDef, InterfaceDef, PropertyDef, TemplateParam,
};
use mir_types::{Atomic, Type};
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
        let fqcn = self.declared_fqn(&interface_name);

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

        // Hoisted above the `@template` bound/default resolution below so a
        // same-file `@psalm-type` alias used in a bound/default (`@template T
        // of Numeric`) is expanded before resolution — `class.rs`'s own
        // template-param construction already does this, this collector
        // previously built its aliases too late to ever use them here.
        let type_aliases = self.build_type_aliases(&iface_doc);

        let iface_template_names: rustc_hash::FxHashSet<String> = iface_doc
            .templates
            .iter()
            .map(|(n, _, _, _)| n.to_string())
            .collect();
        let template_params: Vec<TemplateParam> = iface_doc
            .templates
            .iter()
            .map(|(name, bound, variance, default)| TemplateParam {
                name: name.as_str().into(),
                bound: wrap_template_bound(bound.clone().map(|b| {
                    let b = super::expand_aliases_only(b, &type_aliases);
                    Self::fill_self_static_parent(
                        self.resolve_union_doc_with_templates(
                            b,
                            &iface_template_names,
                            fqcn.as_str(),
                            &[],
                        ),
                        fqcn.as_str(),
                    )
                })),
                default: wrap_template_bound(default.clone().map(|d| {
                    let d = super::expand_aliases_only(d, &type_aliases);
                    Self::fill_self_static_parent(
                        self.resolve_union_doc_with_templates(
                            d,
                            &iface_template_names,
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

        // Build interface-level template params before the member loop so methods referencing
        // interface templates in their return types don't get them wrongly namespace-qualified.
        let iface_template_params = template_params.clone();

        let extends: Vec<Arc<str>> = decl
            .extends
            .iter()
            .map(|n| self.resolve_name(&name_to_string_owned(n)).into())
            .collect();

        // Type args from `@extends BaseIface<T1, T2>` docblock lines — keyed by
        // FQCN (not positional) since a native `extends A, B` clause may list
        // several base interfaces, matched independently of docblock tag order.
        let extends_type_args: Vec<(Arc<str>, Vec<Type>)> = iface_doc
            .extends
            .iter()
            .filter_map(|ty| {
                if let Some(Atomic::TNamedObject {
                    fqcn: base,
                    type_params,
                }) = ty.types.first()
                {
                    Some((
                        self.resolve_type_name(base.as_str(), true).into(),
                        type_params
                            .iter()
                            .map(|tp| {
                                // Template-aware: `T1` in `@extends Base<T1>` is
                                // this interface's own template param, not a class.
                                self.resolve_union_doc_with_templates(
                                    super::expand_aliases_only(tp.clone(), &type_aliases),
                                    &iface_template_names,
                                    &fqcn,
                                    &iface_template_params,
                                )
                            })
                            .collect(),
                    ))
                } else {
                    None
                }
            })
            .collect();

        let mut own_methods = mir_codebase::definitions::MemberMap::default();
        let mut own_constants = mir_codebase::definitions::MemberMap::default();
        let mut own_properties = mir_codebase::definitions::MemberMap::default();

        // See `collector/class.rs` for why this runs before the loop: it lets
        // `int-mask-of<self::*>` in a method docblock below resolve against
        // this interface's own literal-int constants.
        let self_int_constants: Arc<rustc_hash::FxHashMap<Arc<str>, i64>> = Arc::new(
            decl.body
                .members
                .iter()
                .filter_map(|m| match &m.kind {
                    ClassMemberKind::ClassConst(c) => {
                        let name = c.name.as_deref()?;
                        match super::infer_const_value(self, &c.value.kind) {
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
                    if let Some(method) = self.build_method_storage(
                        m,
                        &fqcn,
                        Some(&member.span),
                        Some(&type_aliases),
                        &iface_template_params,
                    ) {
                        own_methods.insert(
                            Arc::from(crate::util::php_ident_lowercase(&method.name).as_str()),
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
                    // PHP 8.3: typed interface constants (`const int FOO;`).
                    // Prefer @var docblock, then a same-kind literal narrowing
                    // of the native hint, then the bare hint, then mixed —
                    // same precedence as class.rs.
                    let hint_ty = self.resolve_union_opt(
                        c.type_hint
                            .as_ref()
                            .map(|h| type_from_hint_owned(h, Some(&fqcn))),
                    );
                    let const_ty = const_doc
                        .var_type
                        .map(|t| {
                            self.resolve_union_doc_with_templates(
                                super::expand_aliases_only(t, &type_aliases),
                                &iface_template_names,
                                &fqcn,
                                &iface_template_params,
                            )
                        })
                        .or_else(|| {
                            super::const_type_with_literal_narrowing(
                                hint_ty,
                                super::infer_const_value(self, &c.value.kind),
                            )
                        })
                        .unwrap_or_else(Type::mixed);
                    own_constants.insert(
                        Arc::from(const_name),
                        ConstantDef {
                            name: Arc::from(const_name),
                            ty: const_ty,
                            visibility: c.visibility.map(|v| Self::convert_visibility(Some(v))),
                            is_final: c.is_final,
                            location: Some(self.location(member.span.start, member.span.end)),
                            deprecated: const_doc.deprecated.as_deref().map(Arc::from).or_else(
                                || {
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
                                },
                            ),
                        },
                    );
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
                    let ty = self
                        .version_attr_type_string(&p.attributes)
                        .map(|s| crate::parser::docblock::parse_type_string(&s))
                        .or_else(|| {
                            prop_doc.var_type.clone().map(|t| {
                                self.resolve_union_doc_with_templates(
                                    super::expand_aliases_only(t, &type_aliases),
                                    &iface_template_names,
                                    fqcn.as_str(),
                                    &iface_template_params,
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
                            deprecated: prop_doc.deprecated.as_deref().map(Arc::from).or_else(
                                || {
                                    if p.attributes.iter().any(|a| {
                                        a.name
                                            .parts
                                            .last()
                                            .map(|part| {
                                                part.as_ref().eq_ignore_ascii_case("Deprecated")
                                            })
                                            .unwrap_or(false)
                                    }) {
                                        Some(Arc::from(""))
                                    } else {
                                        None
                                    }
                                },
                            ),
                            has_native_type: p.type_hint.is_some(),
                            from_docblock: false,
                        },
                    );
                }
                _ => {}
            }
        }

        self.add_docblock_members(
            &iface_doc,
            &type_aliases,
            &fqcn,
            &mut own_methods,
            &mut own_properties,
            Some(self.location(stmt_span.start, stmt_span.end)),
            &iface_template_names,
            &template_params,
        );
        let seal_properties = iface_doc.seal_properties;

        self.slice
            .interfaces
            .push(std::sync::Arc::new(InterfaceDef {
                fqcn: fqcn.into(),
                short_name: Arc::from(interface_name.as_str()),
                extends,
                extends_type_args,
                own_methods,
                own_constants,
                template_params,
                location: Some(self.location(stmt_span.start, stmt_span.end)),
                deprecated: Self::deprecated_from_doc_or_attrs(
                    iface_doc.deprecated.as_deref(),
                    &decl.attributes,
                ),
                own_properties,
                seal_properties,
                type_aliases: type_aliases
                    .iter()
                    .map(|(k, v)| (Arc::from(k.as_str()), v.clone()))
                    .collect(),
            }));

        ControlFlow::Continue(())
    }
}
