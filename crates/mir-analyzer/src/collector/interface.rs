use super::DefinitionCollector;
use crate::parser::{name_to_string_owned, type_from_hint_owned};
use mir_codebase::storage::{wrap_template_bound, ConstantDef, InterfaceDef, TemplateParam};
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
            .map(|(n, _, _, _)| n.to_string())
            .collect();
        let template_params: Vec<TemplateParam> = iface_doc
            .templates
            .iter()
            .map(|(name, bound, variance, default)| TemplateParam {
                name: name.as_str().into(),
                bound: wrap_template_bound(bound.clone().map(|b| {
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

        let mut own_methods = indexmap::IndexMap::new();
        let mut own_constants = indexmap::IndexMap::new();

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
                    if let Some(method) = self.build_method_storage(
                        m,
                        &fqcn,
                        Some(&member.span),
                        None,
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
                    // Prefer @var docblock, then the native type hint, then the
                    // literal value, then mixed — same precedence as class.rs.
                    let hint_ty = self.resolve_union_opt(
                        c.type_hint
                            .as_ref()
                            .map(|h| type_from_hint_owned(h, Some(&fqcn))),
                    );
                    let const_ty = const_doc
                        .var_type
                        .map(|t| {
                            self.resolve_union_doc_with_templates(
                                t,
                                &iface_template_names,
                                &fqcn,
                                &iface_template_params,
                            )
                        })
                        .or(hint_ty)
                        .or_else(|| super::infer_const_value(&c.value.kind))
                        .unwrap_or_else(Type::mixed);
                    own_constants.insert(
                        Arc::from(const_name),
                        ConstantDef {
                            name: Arc::from(const_name),
                            ty: const_ty,
                            visibility: c.visibility.map(|v| Self::convert_visibility(Some(v))),
                            is_final: c.is_final,
                            location: Some(self.location(member.span.start, member.span.end)),
                            deprecated: const_doc.deprecated.as_deref().map(Arc::from),
                        },
                    );
                }
                _ => {}
            }
        }

        let type_aliases = self.build_type_aliases(&iface_doc);
        let mut own_properties = indexmap::IndexMap::new();
        self.add_docblock_members(
            &iface_doc,
            &type_aliases,
            &fqcn,
            &mut own_methods,
            &mut own_properties,
            Some(self.location(stmt_span.start, stmt_span.end)),
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
                deprecated: iface_doc.deprecated.as_deref().map(Arc::from),
                own_properties,
                seal_properties,
            }));

        ControlFlow::Continue(())
    }
}
