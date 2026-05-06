use std::sync::Arc;

use mir_codebase::storage::{ConstantStorage, EnumCaseStorage};
use mir_types::Union;
use php_ast::ast::EnumMemberKind;

use super::DefinitionCollector;
use crate::parser::name_to_string;

impl DefinitionCollector<'_> {
    pub(super) fn collect_enum<'arena, 'src>(
        &mut self,
        decl: &php_ast::ast::EnumDecl<'arena, 'src>,
        stmt_span: php_ast::Span,
    ) {
        let enum_name = decl.name.to_string();
        let fqcn = self.resolve_name(&enum_name);

        let scalar_type = decl
            .scalar_type
            .as_ref()
            .map(|n| crate::parser::docblock::parse_type_string(&name_to_string(n)));

        let interfaces: Vec<Arc<str>> = decl
            .implements
            .iter()
            .map(|n| self.resolve_name(&name_to_string(n)).into())
            .collect();

        let mut cases = indexmap::IndexMap::new();
        let mut own_methods = indexmap::IndexMap::new();
        let mut own_constants = indexmap::IndexMap::new();

        for member in decl.members.iter() {
            match &member.kind {
                EnumMemberKind::Case(c) => {
                    cases.insert(
                        Arc::from(c.name.to_string()),
                        EnumCaseStorage {
                            name: Arc::from(c.name.to_string()),
                            value: c.value.as_ref().map(|_| Union::mixed()),
                            location: Some(self.location(member.span.start, member.span.end)),
                        },
                    );
                }
                EnumMemberKind::Method(m) => {
                    if let Some(method) =
                        self.build_method_storage(m, &fqcn, Some(&member.span), None)
                    {
                        own_methods.insert(
                            Arc::from(method.name.to_lowercase().as_str()),
                            Arc::new(method),
                        );
                    }
                }
                EnumMemberKind::ClassConst(c) => {
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
                EnumMemberKind::TraitUse(_) => {}
            }
        }

        self.slice.enums.push(mir_codebase::EnumStorage {
            fqcn: fqcn.into(),
            short_name: Arc::from(decl.name.to_string()),
            scalar_type,
            interfaces,
            cases,
            own_methods,
            own_constants,
            location: Some(self.location(stmt_span.start, stmt_span.end)),
        });
    }
}
