use std::sync::Arc;

use mir_codebase::storage::{ConstantDef, EnumCaseDef};
use mir_issues::{Issue, IssueKind};
use mir_types::Type;
use php_ast::owned::EnumMemberKind;

use super::DefinitionCollector;
use crate::parser::name_to_string_owned;

impl DefinitionCollector<'_> {
    pub(super) fn collect_enum(
        &mut self,
        decl: &php_ast::owned::EnumDecl,
        stmt_span: php_ast::Span,
    ) {
        let enum_name = decl.name.as_deref().unwrap_or_default().to_string();
        let fqcn = self.resolve_name(&enum_name);

        let enum_doc = decl
            .doc_comment
            .as_ref()
            .map(|c| crate::parser::DocblockParser::parse(&c.text))
            .unwrap_or_default();

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

        let mut cases = indexmap::IndexMap::new();
        let mut own_methods = indexmap::IndexMap::new();
        let mut own_constants = indexmap::IndexMap::new();

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
                EnumMemberKind::Case(c) => {
                    let case_name = c.name.as_deref().unwrap_or_default();
                    let case_doc = c
                        .doc_comment
                        .as_ref()
                        .map(|d| crate::parser::DocblockParser::parse(&d.text))
                        .unwrap_or_default();
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
                        .and_then(|expr| super::infer_const_value(&expr.kind));

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
                    own_constants.insert(
                        Arc::from(const_name),
                        ConstantDef {
                            name: Arc::from(const_name),
                            ty: Type::mixed(),
                            visibility: None,
                            is_final: c.is_final,
                            location: Some(self.location(member.span.start, member.span.end)),
                            deprecated: None,
                        },
                    );
                }
                EnumMemberKind::TraitUse(_) => {}
            }
        }

        let type_aliases = self.build_type_aliases(&enum_doc);
        let mut dummy_properties = indexmap::IndexMap::new();
        self.add_docblock_members(
            &enum_doc,
            &type_aliases,
            &fqcn,
            &mut own_methods,
            &mut dummy_properties,
            Some(self.location(stmt_span.start, stmt_span.end)),
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
                location: Some(self.location(stmt_span.start, stmt_span.end)),
            }));
    }
}
