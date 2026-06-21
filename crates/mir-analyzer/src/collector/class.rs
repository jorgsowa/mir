use super::DefinitionCollector;
use crate::parser::{name_to_string_owned, type_from_hint_owned};
use mir_codebase::storage::{ConstantDef, FnParam, MethodDef, PropertyDef, TemplateParam};
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
        let mut trait_insteadof: indexmap::IndexMap<Arc<str>, Vec<Arc<str>>> =
            indexmap::IndexMap::new();
        #[allow(clippy::type_complexity)]
        let mut trait_aliases: rustc_hash::FxHashMap<
            Arc<str>,
            (
                Option<Arc<str>>,
                Arc<str>,
                Option<mir_codebase::Visibility>,
                Arc<str>,
            ),
        > = rustc_hash::FxHashMap::default();

        let class_doc = self.parse_docblock_from_node(decl.doc_comment.as_ref());

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

        // Build class-level template params before the member loop so they can be passed
        // to build_method_storage, allowing method return types to reference class templates
        // (e.g. TKey) without those names being wrongly namespace-qualified.
        // Collect names first so bounds referencing sibling template params are not FQN-qualified.
        let class_template_names: std::collections::HashSet<String> = class_doc
            .templates
            .iter()
            .map(|(n, _, _)| n.to_string())
            .collect();
        let class_template_params: Vec<mir_codebase::storage::TemplateParam> = class_doc
            .templates
            .iter()
            .map(
                |(name, bound, variance)| mir_codebase::storage::TemplateParam {
                    name: name.as_str().into(),
                    bound: mir_codebase::storage::wrap_template_bound(bound.clone().map(|b| {
                        self.resolve_union_doc_with_templates(
                            b,
                            &class_template_names,
                            fqcn.as_str(),
                            &[],
                        )
                    })),
                    defining_entity: fqcn.as_str().into(),
                    variance: *variance,
                },
            )
            .collect();

        for member in decl.body.members.iter() {
            match &member.kind {
                ClassMemberKind::Method(m) => {
                    if m.name.as_deref() == Some("__construct") {
                        // Promoted-constructor properties take their type from the
                        // native type hint, falling back to the constructor's
                        // `@param` docblock — so `@param T $value` (with `@template T`
                        // on the class) types the promoted `$value` property as the
                        // template param `T`, enabling generic member inference.
                        let ctor_doc = self.parse_docblock_from_node(m.doc_comment.as_ref());
                        let ctor_template_names: std::collections::HashSet<String> = ctor_doc
                            .templates
                            .iter()
                            .map(|(n, _, _)| n.to_string())
                            .chain(class_template_names.iter().cloned())
                            .collect();
                        for p in m.params.iter() {
                            if p.visibility.is_some() {
                                let param_name = p.name.as_deref().unwrap_or_default();
                                let native_ty = self.resolve_union_opt(
                                    p.type_hint
                                        .as_ref()
                                        .map(|h| type_from_hint_owned(h, Some(&fqcn))),
                                );
                                let resolve_doc = || {
                                    ctor_doc.get_param_type(param_name).cloned().map(|u| {
                                        self.resolve_union_doc_with_templates(
                                            u,
                                            &ctor_template_names,
                                            &fqcn,
                                            &class_template_params,
                                        )
                                    })
                                };
                                // When the native hint is a plain unspecialized `array`, prefer
                                // the @param docblock if it provides a more specific generic array.
                                let ty = if native_ty.as_ref().is_some_and(|t| {
                                    t.types.len() == 1
                                        && matches!(
                                            &t.types[0],
                                            Atomic::TArray { key, value }
                                                if key.is_mixed() && value.is_mixed()
                                        )
                                }) {
                                    resolve_doc().or(native_ty)
                                } else {
                                    native_ty.or_else(resolve_doc)
                                };
                                let prop = PropertyDef {
                                    name: Arc::from(param_name),
                                    ty: mir_codebase::storage::wrap_property_type(ty),
                                    inferred_ty: None,
                                    visibility: Self::convert_visibility(p.visibility),
                                    is_static: false,
                                    is_readonly: decl.modifiers.is_readonly || p.is_readonly,
                                    default: mir_codebase::storage::wrap_property_type(
                                        p.default.as_ref().map(|_| mir_types::Type::mixed()),
                                    ),
                                    location: Some(
                                        self.location(member.span.start, member.span.end),
                                    ),
                                    deprecated: None,
                                    has_native_type: p.type_hint.is_some(),
                                };
                                own_properties.insert(Arc::from(param_name), prop);
                            }
                        }
                    }
                    if let Some(method) = self.build_method_storage(
                        m,
                        &fqcn,
                        Some(&member.span),
                        Some(&type_aliases),
                        &class_template_params,
                    ) {
                        let key = Arc::from(method.name.to_lowercase().as_str());
                        if let Some(existing) = own_methods.get(&key) {
                            own_methods.insert(key, merge_method_overloads(existing, &method));
                        } else {
                            own_methods.insert(key, Arc::new(method));
                        }
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
                    // @var docblock overrides the PHP type hint when present, allowing
                    // @var Box<string> to refine a bare Box declaration with type params.
                    // Resolve with class template awareness so `@var array<TKey, TValue>`
                    // keeps TKey/TValue as template params instead of resolving them to
                    // namespaced classes (App\Support\TKey).
                    let ty = prop_doc
                        .var_type
                        .map(|t| {
                            self.resolve_union_doc_with_templates(
                                t,
                                &class_template_names,
                                &fqcn,
                                &class_template_params,
                            )
                        })
                        .or(hint_ty);
                    let prop = PropertyDef {
                        name: Arc::from(prop_name),
                        ty: mir_codebase::storage::wrap_property_type(ty),
                        inferred_ty: None,
                        visibility: Self::convert_visibility(p.visibility),
                        is_static: p.is_static,
                        is_readonly: p.is_readonly
                            || decl.modifiers.is_readonly
                            || prop_doc.is_readonly,
                        default: mir_codebase::storage::wrap_property_type(
                            p.default.as_ref().map(|_| mir_types::Type::mixed()),
                        ),
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
                    };
                    own_properties.insert(Arc::from(prop_name), prop);
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
                    let constant = ConstantDef {
                        name: Arc::from(const_name),
                        ty: mir_types::Type::mixed(),
                        visibility: c.visibility.map(|v| Self::convert_visibility(Some(v))),
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
                    for adaptation in tu.adaptations.iter() {
                        match &adaptation.kind {
                            php_ast::owned::TraitAdaptationKind::Precedence {
                                method,
                                insteadof,
                                ..
                            } => {
                                let method_lower: Arc<str> =
                                    name_to_string_owned(method).to_ascii_lowercase().into();
                                for excluded in insteadof.iter() {
                                    let excluded_fqcn: Arc<str> =
                                        self.resolve_name(&name_to_string_owned(excluded)).into();
                                    trait_insteadof
                                        .entry(method_lower.clone())
                                        .or_default()
                                        .push(excluded_fqcn);
                                }
                            }
                            php_ast::owned::TraitAdaptationKind::Alias {
                                trait_name,
                                method,
                                new_modifier,
                                new_name,
                            } => {
                                if let Some(new_name) = new_name {
                                    let alias_cased: Arc<str> =
                                        name_to_string_owned(new_name).into();
                                    let new_lower: Arc<str> =
                                        alias_cased.to_ascii_lowercase().into();
                                    let orig_lower: Arc<str> =
                                        name_to_string_owned(method).to_ascii_lowercase().into();
                                    let trait_fqcn: Option<Arc<str>> =
                                        trait_name.as_ref().map(|t| {
                                            self.resolve_name(&name_to_string_owned(t)).into()
                                        });
                                    let vis_override =
                                        new_modifier.map(|v| Self::convert_visibility(Some(v)));
                                    trait_aliases.insert(
                                        new_lower,
                                        (trait_fqcn, orig_lower, vis_override, alias_cased),
                                    );
                                }
                            }
                        }
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

        let template_params: Vec<TemplateParam> = class_template_params;

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
            // @final is a soft convention — only the PHP `final` keyword enforces
            // non-extensibility. Docblock-only @final is ignored to avoid FPs when
            // tests or framework code legitimately extends @final-annotated classes.
            is_final: decl.modifiers.is_final,
            is_readonly: decl.modifiers.is_readonly,
            deprecated: class_doc.deprecated.as_deref().map(Arc::from).or_else(|| {
                // Also detect #[Deprecated] / #[\Deprecated] PHP attribute
                if decl.attributes.iter().any(|a| {
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
            is_internal: class_doc.is_internal,
            attribute_flags: parse_attribute_flags(&decl.attributes),
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
            trait_insteadof,
            trait_aliases,
        };

        self.slice.classes.push(std::sync::Arc::new(storage));
        ControlFlow::Continue(())
    }
}

/// PHP `Attribute::TARGET_*` and `IS_REPEATABLE` constants.
const ATTR_TARGET_CLASS: i64 = 1;
const ATTR_TARGET_FUNCTION: i64 = 2;
const ATTR_TARGET_METHOD: i64 = 4;
const ATTR_TARGET_PROPERTY: i64 = 8;
const ATTR_TARGET_CLASS_CONSTANT: i64 = 16;
const ATTR_TARGET_PARAMETER: i64 = 32;
pub(crate) const ATTR_IS_REPEATABLE: i64 = 64;
pub(crate) const ATTR_TARGET_ALL: i64 = 63;

/// Parse the value of `#[Attribute(...)]` flags from a class's attribute list.
/// Returns `None` if no `#[Attribute]` is present, `Some(flags)` otherwise.
pub(crate) fn parse_attribute_flags(attrs: &[php_ast::owned::Attribute]) -> Option<i64> {
    for attr in attrs {
        let last = attr.name.parts.last()?;
        if !last.as_ref().eq_ignore_ascii_case("Attribute") {
            continue;
        }
        // Found `#[Attribute]` or `#[\Attribute]`.
        // Extract the flags from the first argument (if any).
        let flags = if attr.args.is_empty() {
            ATTR_TARGET_ALL
        } else {
            eval_attribute_flags_expr(&attr.args[0].value).unwrap_or(ATTR_TARGET_ALL)
        };
        return Some(flags);
    }
    None
}

/// Recursively evaluate a PHP constant expression as an integer bitmask.
/// Only handles the patterns found in `#[Attribute(...)]` annotations:
/// - `Attribute::TARGET_*` / `Attribute::IS_REPEATABLE` class constants
/// - Integer literals
/// - Bitwise OR of the above
fn eval_attribute_flags_expr(expr: &php_ast::owned::Expr) -> Option<i64> {
    use php_ast::ast::BinaryOp;
    use php_ast::owned::ExprKind;
    match &expr.kind {
        ExprKind::Int(n) => Some(*n),
        ExprKind::Parenthesized(inner) => eval_attribute_flags_expr(inner),
        ExprKind::Binary(b) if b.op == BinaryOp::BitwiseOr => {
            let l = eval_attribute_flags_expr(&b.left)?;
            let r = eval_attribute_flags_expr(&b.right)?;
            Some(l | r)
        }
        ExprKind::ClassConstAccess(access) => {
            // `Attribute::TARGET_CLASS` etc.
            let member = match &access.member.kind {
                ExprKind::Identifier(s) => s.as_ref(),
                _ => return None,
            };
            resolve_attribute_constant(member)
        }
        _ => None,
    }
}

fn resolve_attribute_constant(name: &str) -> Option<i64> {
    match name {
        "TARGET_CLASS" => Some(ATTR_TARGET_CLASS),
        "TARGET_FUNCTION" => Some(ATTR_TARGET_FUNCTION),
        "TARGET_METHOD" => Some(ATTR_TARGET_METHOD),
        "TARGET_PROPERTY" => Some(ATTR_TARGET_PROPERTY),
        "TARGET_CLASS_CONSTANT" => Some(ATTR_TARGET_CLASS_CONSTANT),
        "TARGET_PARAMETER" | "TARGET_CONSTANT" => Some(ATTR_TARGET_PARAMETER),
        "IS_REPEATABLE" => Some(ATTR_IS_REPEATABLE),
        "TARGET_ALL" => Some(ATTR_TARGET_ALL),
        _ => None,
    }
}

/// Merge two overloaded method definitions into one permissive signature.
///
/// PHP stubs represent constructor/method overloads as repeated declarations.
/// We keep the longest param list and mark any param at index ≥ min_required
/// (the minimum required count across both overloads) as optional.
fn merge_method_overloads(existing: &Arc<MethodDef>, new_def: &MethodDef) -> Arc<MethodDef> {
    let count_required = |params: &[FnParam]| {
        params
            .iter()
            .take_while(|p| !p.is_optional && !p.has_default && !p.is_variadic)
            .count()
    };
    let min_required = count_required(&existing.params).min(count_required(&new_def.params));

    let longer = if new_def.params.len() >= existing.params.len() {
        new_def
    } else {
        existing.as_ref()
    };

    let merged_params: Arc<[FnParam]> = longer
        .params
        .iter()
        .enumerate()
        .map(|(i, p)| {
            if i >= min_required {
                FnParam {
                    has_default: true,
                    is_optional: true,
                    ..p.clone()
                }
            } else {
                p.clone()
            }
        })
        .collect::<Vec<_>>()
        .into();

    Arc::new(MethodDef {
        params: merged_params,
        ..longer.clone()
    })
}
