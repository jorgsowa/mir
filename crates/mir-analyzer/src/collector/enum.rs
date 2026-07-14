use std::ops::ControlFlow;
use std::sync::Arc;

use mir_codebase::definitions::{ConstantDef, EnumCaseDef};
use mir_issues::{Issue, IssueKind};
use mir_types::Type;
use php_ast::owned::EnumMemberKind;

use super::DefinitionCollector;
use crate::parser::{name_to_string_owned, type_from_hint_owned};

impl DefinitionCollector<'_> {
    pub(super) fn collect_enum(
        &mut self,
        decl: &php_ast::owned::EnumDecl,
        stmt_span: php_ast::Span,
    ) -> ControlFlow<()> {
        let enum_name = decl.name.as_deref().unwrap_or_default().to_string();
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

        let mut interfaces: Vec<Arc<str>> = decl
            .implements
            .iter()
            .map(|n| self.resolve_name(&name_to_string_owned(n)).into())
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
        let self_int_constants: Arc<rustc_hash::FxHashMap<Arc<str>, i64>> = Arc::new(
            decl.body
                .members
                .iter()
                .filter_map(|m| match &m.kind {
                    EnumMemberKind::ClassConst(c) => {
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
                EnumMemberKind::Case(c) => {
                    let case_name = c.name.as_deref().unwrap_or_default();
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
                    if let (Some(backing), Some(case_val_ty)) = (&scalar_type, &value_ty) {
                        if !case_val_ty.is_subtype_structural(backing) {
                            let lc = self.source_map.offset_to_line_col(member.span.start);
                            let line = lc.line + 1;
                            self.issues.add(Issue::new(
                                IssueKind::BackedEnumCaseTypeMismatch {
                                    enum_name: fqcn.clone(),
                                    case_name: case_name.to_string(),
                                    expected: backing.to_string(),
                                    actual: case_val_ty.to_string(),
                                },
                                mir_issues::Location {
                                    file: self.file.clone(),
                                    line,
                                    line_end: line,
                                    col_start: 0,
                                    col_end: 0,
                                },
                            ));
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
                    if let Some(method) =
                        self.build_method_storage(m, &fqcn, Some(&member.span), None, &[])
                    {
                        own_methods.insert(
                            Arc::from(crate::util::php_ident_lowercase(&method.name).as_str()),
                            Arc::new(method),
                        );
                    }
                }
                EnumMemberKind::ClassConst(c) => {
                    let const_name = c.name.as_deref().unwrap_or_default();
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
                    // Prefer @var docblock, then the native type hint, then the
                    // literal value, then mixed — same precedence as class.rs.
                    let hint_ty = self.resolve_union_opt(
                        c.type_hint
                            .as_ref()
                            .map(|h| type_from_hint_owned(h, Some(&fqcn))),
                    );
                    let const_ty = const_doc
                        .var_type
                        .map(|t| self.resolve_union_doc(t))
                        .or(hint_ty)
                        .or_else(|| super::infer_const_value(self, &c.value.kind))
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
                EnumMemberKind::TraitUse(tu) => {
                    for t in tu.traits.iter() {
                        let trait_fqcn: Arc<str> =
                            self.resolve_name(&name_to_string_owned(t)).into();
                        trait_use_locations
                            .push((trait_fqcn.clone(), self.location(t.span.start, t.span.end)));
                        traits.push(trait_fqcn);
                    }
                }
            }
        }

        let type_aliases = self.build_type_aliases(&enum_doc);
        let mut dummy_properties = mir_codebase::definitions::MemberMap::default();
        self.add_docblock_members(
            &enum_doc,
            &type_aliases,
            &fqcn,
            &mut own_methods,
            &mut dummy_properties,
            Some(self.location(stmt_span.start, stmt_span.end)),
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
                cases,
                own_methods,
                own_constants,
                traits,
                trait_use_locations,
                location: Some(self.location(stmt_span.start, stmt_span.end)),
                deprecated: Self::deprecated_from_doc_or_attrs(
                    enum_doc.deprecated.as_deref(),
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
