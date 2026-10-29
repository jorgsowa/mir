use std::collections::hash_map::Entry;
use std::ops::ControlFlow;
use std::sync::Arc;

use mir_codebase::definitions::{ConstantDef, EnumCaseDef, PropertyDef};
use mir_issues::{Issue, IssueKind};
use mir_types::{Atomic, Type};
use php_ast::owned::EnumMemberKind;
use rustc_hash::FxHashMap;

use super::DefinitionCollector;
use crate::parser::{name_to_string_owned, type_from_hint_owned};

impl DefinitionCollector<'_> {
    pub(super) fn collect_enum(
        &mut self,
        decl: &php_ast::owned::EnumDecl,
        stmt_span: php_ast::Span,
    ) -> ControlFlow<()> {
        let enum_name = decl.name.as_str().unwrap_or_default().to_string();
        let fqcn = self.declared_fqn(&enum_name);

        let enum_doc = decl
            .doc_comment
            .as_ref()
            .map(|c| crate::parser::DocblockParser::parse(&c.text))
            .unwrap_or_default();

        let enum_doc_span = decl
            .doc_comment
            .as_ref()
            .map(|c| c.span.start)
            .unwrap_or(stmt_span.start);
        self.emit_docblock_issues(&enum_doc, enum_doc_span);

        if !self.version_allows(&enum_doc) {
            return ControlFlow::Continue(());
        }

        let scalar_type = decl
            .scalar_type
            .as_ref()
            .map(|n| crate::parser::docblock::parse_type_string(&name_to_string_owned(n)));
        // Any other backing type is the parser's `invalid-enum-backing-type`.
        let checked_backing_type = decl.scalar_type.as_ref().is_some_and(|n| {
            let name = name_to_string_owned(n);
            name.eq_ignore_ascii_case("int") || name.eq_ignore_ascii_case("string")
        });

        let mut interfaces: Vec<Arc<str>> = decl
            .implements
            .iter()
            .map(|n| self.resolve_ast_name(n).into())
            .collect();

        // PHP automatically makes every enum implement UnitEnum (and backed enums
        // implement IntBackedEnum / StringBackedEnum, which extend BackedEnum → UnitEnum).
        // Inject the appropriate interface so method resolution finds cases() / from() /
        // tryFrom() in the stubs without the user having to write `implements BackedEnum`.
        let implicit_iface: Arc<str> = match decl.scalar_type.as_ref() {
            None => Arc::from("UnitEnum"),
            Some(n) if name_to_string_owned(n).eq_ignore_ascii_case("string") => {
                Arc::from("StringBackedEnum")
            }
            _ => Arc::from("IntBackedEnum"),
        };
        if !interfaces.contains(&implicit_iface) {
            interfaces.push(implicit_iface);
        }

        let mut cases = mir_codebase::definitions::MemberMap::default();
        let mut own_methods = mir_codebase::definitions::MemberMap::default();
        let mut own_constants = mir_codebase::definitions::MemberMap::default();
        let mut traits: Vec<Arc<str>> = Vec::new();
        let mut trait_use_locations: Vec<(Arc<str>, mir_types::Location)> = Vec::new();

        // See `class.rs` for why this runs before the loop: it lets
        // `int-mask-of<self::*>` in a method docblock below resolve against
        // the enum's own literal-int `const` declarations (not its cases —
        // those aren't class constants).
        let own_consts: Vec<(&str, &php_ast::owned::Expr)> = decl
            .body
            .members
            .iter()
            .filter_map(|m| match &m.kind {
                EnumMemberKind::ClassConst(c) => Some((c.name.as_str()?, &c.value)),
                _ => None,
            })
            .collect();
        let prev_own_constants = self.enter_own_constants(
            &fqcn,
            None,
            decl.body
                .members
                .iter()
                .filter_map(|m| match &m.kind {
                    EnumMemberKind::Case(c) => c.name.as_str().map(str::to_string),
                    _ => None,
                })
                .collect(),
            &own_consts,
        );
        let self_int_constants = self.own_literal_constants(&own_consts);
        let _int_mask_guard =
            crate::parser::docblock::SelfIntConstantsGuard::activate(&fqcn, &self_int_constants);

        // Hoisted above the member loop (mirroring class.rs) so an enum
        // method's own `@param`/`@return`/assertion tags can resolve a
        // same-file `@psalm-type` alias declared on the enum itself — a
        // method body is otherwise processed with `aliases: None` below,
        // silently leaving any alias reference in its docblock unexpanded.
        let type_aliases = self.build_type_aliases_deferring_imports(&enum_doc);

        let mut declared_properties = Vec::new();
        let mut case_values: FxHashMap<EnumCaseValue, &str> = FxHashMap::default();
        for member in decl.body.members.iter() {
            match &member.kind {
                EnumMemberKind::Case(c) => {
                    let case_name = c.name.as_str().unwrap_or_default();
                    let case_doc = c
                        .doc_comment
                        .as_ref()
                        .map(|d| crate::parser::DocblockParser::parse(&d.text))
                        .unwrap_or_default();
                    let case_doc_span = c
                        .doc_comment
                        .as_ref()
                        .map(|d| d.span.start)
                        .unwrap_or(member.span.start);
                    self.emit_docblock_issues(&case_doc, case_doc_span);
                    if !self.version_allows(&case_doc) {
                        continue;
                    }
                    let case_deprecated =
                        case_doc.deprecated.as_deref().map(Arc::from).or_else(|| {
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
                        });

                    // Extract the literal type from the case value expression (e.g. `= 'foo'`, `= 42`).
                    // Falls back to mixed when the expression is not a simple literal (class constant,
                    // const expression) — those can't be statically checked at collection time.
                    let value_ty = c
                        .value
                        .as_ref()
                        .and_then(|expr| super::infer_const_value(self, &expr.kind));

                    // Validate the case value's type against the backing scalar type.
                    if let (Some(backing), Some(case_val_ty), true) =
                        (&scalar_type, &value_ty, checked_backing_type)
                    {
                        if !case_val_ty.is_subtype_structural(backing) {
                            let value_span = c.value.as_ref().map_or(member.span, |e| e.span);
                            self.issues.add(Issue::new(
                                IssueKind::BackedEnumCaseTypeMismatch {
                                    enum_name: fqcn.clone(),
                                    case_name: case_name.to_string(),
                                    expected: backing.to_string(),
                                    actual: php_value_type_name(case_val_ty)
                                        .map_or_else(|| case_val_ty.to_string(), str::to_string),
                                },
                                self.location(value_span.start, value_span.end),
                            ));
                        }
                    }

                    if let Some(value_key) = value_ty
                        .as_ref()
                        .filter(|_| scalar_type.is_some())
                        .and_then(enum_case_value_key)
                    {
                        match case_values.entry(value_key) {
                            Entry::Occupied(first) => {
                                let value_span = c.value.as_ref().map_or(member.span, |e| e.span);
                                self.issues.add(Issue::new(
                                    IssueKind::DuplicateEnumCaseValue {
                                        enum_name: fqcn.clone(),
                                        first_case: first.get().to_string(),
                                        case_name: case_name.to_string(),
                                    },
                                    self.location(value_span.start, value_span.end),
                                ));
                            }
                            Entry::Vacant(slot) => {
                                slot.insert(case_name);
                            }
                        }
                    }

                    cases.insert(
                        Arc::from(case_name),
                        EnumCaseDef {
                            name: Arc::from(case_name),
                            // Store the inferred literal type; fall back to mixed for non-literal values.
                            value: Some(value_ty.unwrap_or_else(Type::mixed)),
                            location: Some(self.location(member.span.start, member.span.end)),
                            deprecated: case_deprecated,
                        },
                    );
                }
                EnumMemberKind::Method(m) => {
                    if let Some(method) = self.build_method_storage(
                        m,
                        &fqcn,
                        Some(&member.span),
                        Some(&type_aliases),
                        &[],
                    ) {
                        own_methods.insert(
                            Arc::from(crate::util::php_ident_lowercase(&method.name)),
                            Arc::new(method),
                        );
                    }
                    if let Some(body) = &m.body {
                        self.scan_stmts_for_defines(&body.stmts);
                    }
                }
                EnumMemberKind::ClassConst(c) => {
                    let const_name = c.name.as_str().unwrap_or_default();
                    let const_doc = self.parse_docblock_from_node(c.doc_comment.as_ref());
                    let const_doc_span = c
                        .doc_comment
                        .as_ref()
                        .map(|d| d.span.start)
                        .unwrap_or(member.span.start);
                    self.emit_docblock_issues(&const_doc, const_doc_span);
                    if !self.version_allows(&const_doc) {
                        continue;
                    }
                    // PHP 8.3: typed enum constants (`const int FOO = 1;`).
                    // Prefer @var docblock, then a same-kind literal narrowing
                    // of the native hint, then the bare hint, then mixed —
                    // same precedence as class.rs.
                    let hint_ty = self.resolve_union_opt(
                        c.type_hint
                            .as_ref()
                            .map(|h| type_from_hint_owned(h, Some(&fqcn))),
                    );
                    let literal_value = const_doc
                        .var_type
                        .is_some()
                        .then(|| super::infer_const_value(self, &c.value.kind))
                        .flatten();
                    let const_ty = const_doc
                        .var_type
                        .map(|t| self.resolve_union_doc_with_aliases(t, &type_aliases))
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
                            literal_value,
                            visibility: c
                                .modifiers
                                .visibility
                                .map(|v| Self::convert_visibility(Some(v))),
                            is_final: c.modifiers.is_final,
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
                // PHP rejects this (the parser reports `enum-property`); registering it
                // keeps that the only issue. `name` / `value` already exist on every enum.
                EnumMemberKind::Property(p)
                    if !matches!(p.name.as_str(), Some("name" | "value")) =>
                {
                    let prop_doc = self.parse_docblock_from_node(p.doc_comment.as_ref());
                    let hint_ty = self.resolve_union_opt(
                        p.type_hint
                            .as_ref()
                            .map(|h| type_from_hint_owned(h, Some(&fqcn))),
                    );
                    let ty = prop_doc
                        .var_type
                        .map(|t| self.resolve_union_doc_with_aliases(t, &type_aliases))
                        .or_else(|| hint_ty.clone());
                    let default_ty = p
                        .default
                        .as_ref()
                        .map(|d| super::literal_types::literal_type(d).unwrap_or_else(Type::mixed));
                    let prop_name = p.name.as_str().unwrap_or_default();
                    declared_properties.push(PropertyDef {
                        name: Arc::from(prop_name),
                        ty: mir_codebase::definitions::wrap_property_type(ty),
                        inferred_ty: None,
                        native_ty: mir_codebase::definitions::wrap_property_type(hint_ty),
                        visibility: Self::convert_visibility(p.modifiers.visibility),
                        set_visibility: Self::convert_set_visibility(&p.modifiers),
                        is_static: p.modifiers.is_static,
                        is_readonly: p.modifiers.is_readonly || prop_doc.is_readonly,
                        has_native_readonly: p.modifiers.is_readonly,
                        is_override: self.has_override_attribute(&p.attributes),
                        default: mir_codebase::definitions::wrap_property_type(default_ty),
                        location: Some(self.location(member.span.start, member.span.end)),
                        deprecated: Self::deprecated_from_doc_or_attrs(
                            prop_doc.deprecated.as_deref(),
                            &p.attributes,
                        ),
                        has_native_type: p.type_hint.is_some(),
                        get_only_hook: super::is_get_only_hook(&p.hooks),
                        abstract_hooks: super::abstract_hooks(&p.hooks),
                        virtual_hooks: self
                            .virtual_hooks(&p.hooks, p.name.as_str().unwrap_or_default()),
                        from_docblock: false,
                    });
                }
                EnumMemberKind::TraitUse(tu) => {
                    for t in tu.traits.iter() {
                        let trait_fqcn: Arc<str> = self.resolve_ast_name(t).into();
                        trait_use_locations
                            .push((trait_fqcn.clone(), self.location(t.span.start, t.span.end)));
                        traits.push(trait_fqcn);
                    }
                }
                _ => {}
            }
        }

        // `@implements Interface<T1, T2>` — enums have no `@template` of their
        // own, so there's nothing to substitute into the type args (unlike
        // class.rs's `implements_type_args`, which resolves them against the
        // class's own template params); just FQN-qualify the names.
        let implements_type_args: Vec<(Arc<str>, Vec<Type>)> = enum_doc
            .implements
            .iter()
            .filter_map(|ty| {
                if let Some(mir_types::Atomic::TNamedObject {
                    fqcn: iface,
                    type_params,
                }) = ty.types.first()
                {
                    Some((
                        self.resolve_type_name(iface.as_str(), true).into(),
                        type_params
                            .iter()
                            .map(|tp| {
                                Self::fill_self_static_parent(
                                    self.resolve_union_doc_with_templates(
                                        super::expand_aliases_only(tp.clone(), &type_aliases),
                                        &rustc_hash::FxHashSet::default(),
                                        &fqcn,
                                        &[],
                                    ),
                                    &fqcn,
                                )
                            })
                            .collect(),
                    ))
                } else {
                    None
                }
            })
            .collect();

        let trait_use_type_args: Vec<(Arc<str>, Vec<mir_types::Type>)> = enum_doc
            .uses
            .iter()
            .filter_map(|ty| {
                if let Some(mir_types::Atomic::TNamedObject {
                    fqcn: used_trait,
                    type_params,
                }) = ty.types.first()
                {
                    Some((
                        self.resolve_type_name(used_trait.as_str(), true).into(),
                        type_params
                            .iter()
                            .map(|tp| {
                                Self::fill_self_static_parent(
                                    self.resolve_union_doc_with_templates(
                                        super::expand_aliases_only(tp.clone(), &type_aliases),
                                        &rustc_hash::FxHashSet::default(),
                                        &fqcn,
                                        &[],
                                    ),
                                    &fqcn,
                                )
                            })
                            .collect(),
                    ))
                } else {
                    None
                }
            })
            .collect();

        let mut own_properties = mir_codebase::definitions::MemberMap::default();
        for prop in declared_properties {
            own_properties.insert(prop.name.clone(), prop);
        }
        self.add_docblock_members(
            &enum_doc,
            &type_aliases,
            &fqcn,
            &mut own_methods,
            &mut own_properties,
            Some(self.location(stmt_span.start, stmt_span.end)),
            decl.doc_comment.as_ref().map(|c| c.span.start),
            &rustc_hash::FxHashSet::default(),
            &[],
        );

        self.slice
            .enums
            .push(std::sync::Arc::new(mir_codebase::EnumDef {
                fqcn: fqcn.into(),
                short_name: Arc::from(enum_name.as_str()),
                scalar_type,
                interfaces,
                implements_type_args,
                cases,
                own_methods,
                own_constants,
                traits,
                trait_use_locations,
                trait_use_type_args,
                location: Some(self.location(stmt_span.start, stmt_span.end)),
                deprecated: Self::deprecated_from_doc_or_attrs(
                    enum_doc.deprecated.as_deref(),
                    &decl.attributes,
                ),
                type_aliases: Self::stored_type_aliases(&type_aliases),
                own_properties,
            }));
        self.own_constants = prev_own_constants;
        ControlFlow::Continue(())
    }
}

#[derive(PartialEq, Eq, Hash)]
pub(crate) enum EnumCaseValue {
    Int(i64),
    String(Arc<str>),
}

pub(crate) fn enum_case_value_key(ty: &Type) -> Option<EnumCaseValue> {
    match ty.types.as_slice() {
        [Atomic::TLiteralInt(i)] => Some(EnumCaseValue::Int(*i)),
        [Atomic::TLiteralString(s)] => Some(EnumCaseValue::String(s.clone())),
        _ => None,
    }
}

/// The type name PHP's `zend_zval_type_name` gives a constant case value.
pub(crate) fn php_value_type_name(ty: &mir_types::Type) -> Option<&'static str> {
    use mir_types::Atomic;
    let [atomic] = ty.types.as_slice() else {
        return None;
    };
    Some(match atomic {
        Atomic::TInt
        | Atomic::TLiteralInt(_)
        | Atomic::TIntRange { .. }
        | Atomic::TPositiveInt
        | Atomic::TNegativeInt
        | Atomic::TNonNegativeInt => "int",
        Atomic::TString
        | Atomic::TLiteralString(_)
        | Atomic::TNonEmptyString
        | Atomic::TNumericString
        | Atomic::TCallableString
        | Atomic::TClassString(_)
        | Atomic::TInterfaceString(_)
        | Atomic::TEnumString
        | Atomic::TTraitString => "string",
        Atomic::TFloat | Atomic::TIntegralFloat | Atomic::TLiteralFloat(..) => "float",
        Atomic::TBool | Atomic::TTrue | Atomic::TFalse => "bool",
        Atomic::TNull => "null",
        Atomic::TArray { .. } | Atomic::TKeyedArray { .. } | Atomic::TList { .. } => "array",
        Atomic::TObject
        | Atomic::TNamedObject { .. }
        | Atomic::TLiteralEnumCase { .. }
        | Atomic::TSelf { .. }
        | Atomic::TStaticObject { .. } => "object",
        _ => return None,
    })
}
