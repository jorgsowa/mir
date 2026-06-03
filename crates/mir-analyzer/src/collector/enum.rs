use std::sync::Arc;

use mir_codebase::storage::{ConstantDef, EnumCaseDef};
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

        let interfaces: Vec<Arc<str>> = decl
            .implements
            .iter()
            .map(|n| self.resolve_name(&name_to_string_owned(n)).into())
            .collect();

        let mut cases = indexmap::IndexMap::new();
        let mut own_methods = indexmap::IndexMap::new();
        let mut own_constants = indexmap::IndexMap::new();

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
                    cases.insert(
                        Arc::from(case_name),
                        EnumCaseDef {
                            name: Arc::from(case_name),
                            value: c.value.as_ref().map(|_| Type::mixed()),
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
                            Arc::from(method.name.to_lowercase().as_str()),
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
