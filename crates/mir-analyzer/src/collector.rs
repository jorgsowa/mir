/// Pass 1 — Definition collector.
///
/// Visits every top-level declaration in the AST and populates the `Codebase`
/// with `ClassStorage`, `FunctionStorage`, etc. No type inference happens here;
/// we only record the *signatures* of all symbols.
use std::sync::Arc;

use php_ast::ast::{
    ClassMemberKind, EnumMemberKind, Program, StmtKind, Visibility as AstVisibility,
};
use std::ops::ControlFlow;

use php_ast::visitor::Visitor;

use crate::parser::{find_preceding_docblock, name_to_string, type_from_hint};
use mir_codebase::storage::{
    ConstantStorage, EnumCaseStorage, FnParam, FunctionStorage, InterfaceStorage, Location,
    MethodStorage, PropertyStorage, TemplateParam, TraitStorage, Visibility,
};
use mir_codebase::{ClassStorage, Codebase};
use mir_issues::{Issue, IssueBuffer, Location as IssueLocation};
use mir_types::Union;

// ---------------------------------------------------------------------------
// DefinitionCollector
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub struct DefinitionCollector<'a> {
    codebase: &'a Codebase,
    file: Arc<str>,
    source: &'a str,
    source_map: &'a php_ast::source_map::SourceMap,
    namespace: Option<String>,
    /// `use` aliases: alias → FQCN
    use_aliases: std::collections::HashMap<String, String>,
    issues: IssueBuffer,
}

impl<'a> DefinitionCollector<'a> {
    pub fn new(
        codebase: &'a Codebase,
        file: Arc<str>,
        source: &'a str,
        source_map: &'a php_ast::source_map::SourceMap,
    ) -> Self {
        Self {
            source_map,
            codebase,
            file,
            source,
            namespace: None,
            use_aliases: std::collections::HashMap::new(),
            issues: IssueBuffer::new(),
        }
    }

    pub fn collect<'arena, 'src>(mut self, program: &Program<'arena, 'src>) -> Vec<Issue> {
        let _ = self.visit_program(program);
        self.issues.into_issues()
    }

    // -----------------------------------------------------------------------
    // FQCN resolution helpers
    // -----------------------------------------------------------------------

    fn resolve_name(&self, name: &str) -> String {
        // If name starts with \, it's globally qualified — strip and return as-is.
        if name.starts_with('\\') {
            return name.trim_start_matches('\\').to_string();
        }
        // Check use aliases
        let first_part = name.split('\\').next().unwrap_or(name);
        if let Some(resolved) = self.use_aliases.get(first_part) {
            if name.contains('\\') {
                let rest = &name[first_part.len()..];
                return format!("{}{}", resolved, rest);
            }
            return resolved.clone();
        }
        // Qualify with namespace
        if let Some(ns) = &self.namespace {
            return format!("{}\\{}", ns, name);
        }
        name.to_string()
    }

    /// Resolve a short class name through use aliases only (no namespace qualification).
    /// Used for docblock types where short names may be template parameters.
    fn resolve_alias_only(&self, name: &str) -> String {
        let name = name.trim_start_matches('\\');
        let first_part = name.split('\\').next().unwrap_or(name);
        if let Some(resolved) = self.use_aliases.get(first_part) {
            if name.contains('\\') {
                let rest = &name[first_part.len()..];
                return format!("{}{}", resolved, rest);
            }
            return resolved.clone();
        }
        name.to_string()
    }

    /// Resolve a type name for use in stored types:
    /// - If the first segment is a known use-alias, expand it regardless of `\` in the name.
    /// - Otherwise, if the name has no `\`, apply full qualification or alias-only.
    /// - If the name already contains `\` and the first segment is NOT a known alias,
    ///   treat it as already fully qualified.
    fn resolve_type_name(&self, name: &Arc<str>, full_qualify: bool) -> Arc<str> {
        let stripped = name.trim_start_matches('\\');
        let first_part = stripped.split('\\').next().unwrap_or(stripped);
        // If the first segment is a known use-alias, always expand it
        if self.use_aliases.contains_key(first_part) {
            return self.resolve_alias_only(stripped).into();
        }
        // No alias match — if already has namespace separator, treat as fully qualified
        if stripped.contains('\\') {
            return Arc::from(stripped);
        }
        // Short name — apply full resolution
        if full_qualify {
            self.resolve_name(stripped).into()
        } else {
            Arc::from(stripped)
        }
    }

    /// Resolve all TNamedObject FQCNs in a Union through the file's import table.
    /// `full_qualify`: if true, also qualifies unresolved short names with the namespace.
    ///                 if false, only resolves explicit use aliases (safe for docblock types
    ///                 where short names may be template parameters like `T` or `A`).
    fn resolve_union_inner(&self, union: Union, full_qualify: bool) -> Union {
        use mir_types::Atomic;
        let from_docblock = union.from_docblock;
        let types: Vec<Atomic> = union
            .types
            .into_iter()
            .map(|a| self.resolve_atomic_inner(a, full_qualify))
            .collect();
        let mut result = mir_types::Union::from_vec(types);
        result.from_docblock = from_docblock;
        result
    }

    fn resolve_atomic_inner(
        &self,
        atomic: mir_types::Atomic,
        full_qualify: bool,
    ) -> mir_types::Atomic {
        use mir_types::Atomic;
        match atomic {
            Atomic::TNamedObject { fqcn, type_params } => {
                let resolved = self.resolve_type_name(&fqcn, full_qualify);
                Atomic::TNamedObject {
                    fqcn: resolved,
                    type_params,
                }
            }
            Atomic::TClassString(Some(cls)) => {
                let resolved = self.resolve_type_name(&cls, full_qualify);
                Atomic::TClassString(Some(resolved))
            }
            Atomic::TArray { key, value } => Atomic::TArray {
                key: Box::new(self.resolve_union_inner(*key, full_qualify)),
                value: Box::new(self.resolve_union_inner(*value, full_qualify)),
            },
            Atomic::TList { value } => Atomic::TList {
                value: Box::new(self.resolve_union_inner(*value, full_qualify)),
            },
            Atomic::TNonEmptyArray { key, value } => Atomic::TNonEmptyArray {
                key: Box::new(self.resolve_union_inner(*key, full_qualify)),
                value: Box::new(self.resolve_union_inner(*value, full_qualify)),
            },
            Atomic::TNonEmptyList { value } => Atomic::TNonEmptyList {
                value: Box::new(self.resolve_union_inner(*value, full_qualify)),
            },
            other => other,
        }
    }

    /// Fill in empty-FQCN sentinels (TSelf/TStaticObject/TParent with fqcn="")
    /// produced by the docblock parser when we now know the class FQCN.
    fn fill_self_static_parent(union: Union, class_fqcn: &str) -> Union {
        use mir_types::Atomic;
        let mut result = Union::empty();
        result.possibly_undefined = union.possibly_undefined;
        result.from_docblock = union.from_docblock;
        for a in union.types {
            let filled = match a {
                Atomic::TSelf { ref fqcn } if fqcn.is_empty() => Atomic::TSelf {
                    fqcn: class_fqcn.into(),
                },
                Atomic::TStaticObject { ref fqcn } if fqcn.is_empty() => Atomic::TStaticObject {
                    fqcn: class_fqcn.into(),
                },
                Atomic::TParent { ref fqcn } if fqcn.is_empty() => Atomic::TParent {
                    fqcn: class_fqcn.into(),
                },
                other => other,
            };
            result.types.push(filled);
        }
        result
    }

    /// Resolve for PHP type hints: applies use aliases AND namespace qualification.
    fn resolve_union(&self, union: Union) -> Union {
        self.resolve_union_inner(union, true)
    }

    /// Resolve for docblock types: applies use aliases only, no namespace qualification.
    /// Template parameters like `T` or `A` are left as-is.
    fn resolve_union_doc(&self, union: Union) -> Union {
        self.resolve_union_inner(union, false)
    }

    fn resolve_union_opt(&self, opt: Option<Union>) -> Option<Union> {
        opt.map(|u| self.resolve_union(u))
    }

    fn location(&self, start: u32, end: u32) -> Location {
        let lc = self.source_map.offset_to_line_col(start);
        Location::with_line_col(self.file.clone(), start, end, lc.line + 1, lc.col as u16)
    }

    #[allow(dead_code)]
    fn issue_location(&self, start: u32) -> IssueLocation {
        let lc = self.source_map.offset_to_line_col(start);
        let (line, col) = (lc.line + 1, lc.col as u16);
        IssueLocation {
            file: self.file.clone(),
            line,
            col_start: col,
            col_end: col,
        }
    }

    // -----------------------------------------------------------------------
    // Visibility conversion
    // -----------------------------------------------------------------------

    fn convert_visibility(v: Option<AstVisibility>) -> Visibility {
        match v {
            Some(AstVisibility::Public) | None => Visibility::Public,
            Some(AstVisibility::Protected) => Visibility::Protected,
            Some(AstVisibility::Private) => Visibility::Private,
        }
    }

    // -----------------------------------------------------------------------
    // Process statements
    // -----------------------------------------------------------------------

    fn process_stmts<'arena, 'src>(
        &mut self,
        stmts: &php_ast::ast::ArenaVec<'arena, php_ast::ast::Stmt<'arena, 'src>>,
    ) {
        for stmt in stmts.iter() {
            let _ = self.visit_stmt(stmt);
        }
    }
}

impl<'a, 'arena, 'src> Visitor<'arena, 'src> for DefinitionCollector<'a> {
    fn visit_stmt(&mut self, stmt: &php_ast::ast::Stmt<'arena, 'src>) -> ControlFlow<()> {
        match &stmt.kind {
            StmtKind::Namespace(ns) => {
                self.namespace = ns.name.as_ref().map(name_to_string);
                match &ns.body {
                    php_ast::ast::NamespaceBody::Braced(stmts) => {
                        // Save and restore use aliases per namespace block
                        let saved_aliases = self.use_aliases.clone();
                        self.use_aliases.clear();
                        self.process_stmts(stmts);
                        self.use_aliases = saved_aliases;
                    }
                    php_ast::ast::NamespaceBody::Simple => {
                        // Simple namespace — affects all subsequent declarations
                    }
                }
            }

            StmtKind::Use(use_decl) => {
                for item in use_decl.uses.iter() {
                    let full_name = name_to_string(&item.name);
                    let alias = item
                        .alias
                        .unwrap_or_else(|| full_name.rsplit('\\').next().unwrap_or(&full_name));
                    self.use_aliases.insert(alias.to_string(), full_name);
                }
            }

            StmtKind::Function(decl) => {
                let short_name = decl.name.to_string();
                let fqn = if let Some(ns) = &self.namespace {
                    format!("{}\\{}", ns, short_name)
                } else {
                    short_name.clone()
                };

                let doc = find_preceding_docblock(self.source, stmt.span.start)
                    .map(|d| crate::parser::DocblockParser::parse(&d))
                    .unwrap_or_default();

                let mut params = Vec::new();
                for p in decl.params.iter() {
                    let ty = doc
                        .get_param_type(p.name)
                        .cloned()
                        .map(|u| self.resolve_union_doc(u))
                        .or_else(|| {
                            self.resolve_union_opt(
                                p.type_hint.as_ref().map(|h| type_from_hint(h, None)),
                            )
                        });
                    params.push(FnParam {
                        name: p.name.into(),
                        ty,
                        default: p.default.as_ref().map(|_| Union::mixed()),
                        is_variadic: p.variadic,
                        is_byref: p.by_ref,
                        is_optional: p.default.is_some() || p.variadic,
                    });
                }

                let return_type = match (doc.return_type.clone(), decl.return_type.as_ref()) {
                    (Some(mut ty), _) => {
                        ty.from_docblock = true;
                        Some(self.resolve_union_doc(ty))
                    }
                    (None, Some(h)) => self.resolve_union_opt(Some(type_from_hint(h, None))),
                    (None, None) => None,
                };

                let template_params = doc
                    .templates
                    .iter()
                    .map(|(name, bound)| TemplateParam {
                        name: name.as_str().into(),
                        bound: bound.clone(),
                        defining_entity: fqn.as_str().into(),
                    })
                    .collect();

                let storage = FunctionStorage {
                    fqn: fqn.clone().into(),
                    short_name: short_name.into(),
                    params,
                    return_type,
                    inferred_return_type: None,
                    template_params,
                    assertions: vec![],
                    throws: doc.throws.iter().map(|t| Arc::from(t.as_str())).collect(),
                    is_deprecated: doc.is_deprecated,
                    is_pure: doc.is_pure,
                    location: Some(self.location(stmt.span.start, stmt.span.end)),
                };

                self.codebase
                    .symbol_to_file
                    .insert(Arc::from(fqn.as_str()), self.file.clone());
                self.codebase.functions.insert(fqn.into(), storage);
            }

            StmtKind::Class(decl) => {
                let name = match decl.name {
                    Some(n) => n.to_string(),
                    None => return ControlFlow::Continue(()), // anonymous class — handled at expression level
                };
                let fqcn = self.resolve_name(&name);
                let short_name = name;

                let parent = decl
                    .extends
                    .as_ref()
                    .map(|n| self.resolve_name(&name_to_string(n)).into());
                let interfaces: Vec<Arc<str>> = decl
                    .implements
                    .iter()
                    .map(|n| self.resolve_name(&name_to_string(n)).into())
                    .collect();

                let mut own_methods = indexmap::IndexMap::new();
                let mut own_properties = indexmap::IndexMap::new();
                let mut own_constants = indexmap::IndexMap::new();
                let mut trait_uses: Vec<Arc<str>> = vec![];

                for member in decl.members.iter() {
                    match &member.kind {
                        ClassMemberKind::Method(m) => {
                            // Constructor promotion: params with visibility create properties.
                            if m.name == "__construct" {
                                for p in m.params.iter() {
                                    if p.visibility.is_some() {
                                        let ty = self.resolve_union_opt(
                                            p.type_hint
                                                .as_ref()
                                                .map(|h| type_from_hint(h, Some(&fqcn))),
                                        );
                                        let prop = PropertyStorage {
                                            name: p.name.into(),
                                            ty,
                                            inferred_ty: None,
                                            visibility: Self::convert_visibility(p.visibility),
                                            is_static: false,
                                            is_readonly: decl.modifiers.is_readonly,
                                            default: p.default.as_ref().map(|_| Union::mixed()),
                                            location: Some(
                                                self.location(member.span.start, member.span.end),
                                            ),
                                        };
                                        own_properties.insert(p.name.into(), prop);
                                    }
                                }
                            }
                            let method = self.build_method_storage(m, &fqcn, Some(&member.span));
                            own_methods
                                .insert(Arc::from(method.name.to_lowercase().as_str()), method);
                        }
                        ClassMemberKind::Property(p) => {
                            let prop = PropertyStorage {
                                name: p.name.into(),
                                ty: self.resolve_union_opt(
                                    p.type_hint.as_ref().map(|h| type_from_hint(h, Some(&fqcn))),
                                ),
                                inferred_ty: None,
                                visibility: Self::convert_visibility(p.visibility),
                                is_static: p.is_static,
                                is_readonly: p.is_readonly || decl.modifiers.is_readonly,
                                default: p.default.as_ref().map(|_| Union::mixed()),
                                location: Some(self.location(member.span.start, member.span.end)),
                            };
                            own_properties.insert(p.name.into(), prop);
                        }
                        ClassMemberKind::ClassConst(c) => {
                            let constant = ConstantStorage {
                                name: c.name.into(),
                                ty: Union::mixed(),
                                visibility: c.visibility.map(|v| Self::convert_visibility(Some(v))),
                                location: Some(self.location(member.span.start, member.span.end)),
                            };
                            own_constants.insert(c.name.into(), constant);
                        }
                        ClassMemberKind::TraitUse(tu) => {
                            for t in tu.traits.iter() {
                                trait_uses.push(self.resolve_name(&name_to_string(t)).into());
                            }
                        }
                    }
                }

                let class_doc = find_preceding_docblock(self.source, stmt.span.start)
                    .map(|d| crate::parser::DocblockParser::parse(&d))
                    .unwrap_or_default();

                let template_params: Vec<TemplateParam> = class_doc
                    .templates
                    .iter()
                    .map(|(name, bound)| TemplateParam {
                        name: name.as_str().into(),
                        bound: bound.clone(),
                        defining_entity: fqcn.as_str().into(),
                    })
                    .collect();

                let storage = ClassStorage {
                    fqcn: fqcn.clone().into(),
                    short_name: short_name.into(),
                    parent,
                    interfaces,
                    traits: trait_uses,
                    own_methods,
                    own_properties,
                    own_constants,
                    template_params,
                    is_abstract: decl.modifiers.is_abstract,
                    is_final: decl.modifiers.is_final,
                    is_readonly: decl.modifiers.is_readonly,
                    all_methods: indexmap::IndexMap::new(),
                    all_parents: vec![],
                    is_deprecated: class_doc.is_deprecated,
                    is_internal: class_doc.is_internal,
                    location: Some(self.location(stmt.span.start, stmt.span.end)),
                };

                self.codebase
                    .symbol_to_file
                    .insert(Arc::from(fqcn.as_str()), self.file.clone());
                self.codebase.classes.insert(fqcn.into(), storage);
            }

            StmtKind::Interface(decl) => {
                let fqcn = self.resolve_name(decl.name);

                let extends: Vec<Arc<str>> = decl
                    .extends
                    .iter()
                    .map(|n| self.resolve_name(&name_to_string(n)).into())
                    .collect();

                let mut own_methods = indexmap::IndexMap::new();
                let mut own_constants = indexmap::IndexMap::new();

                for member in decl.members.iter() {
                    match &member.kind {
                        ClassMemberKind::Method(m) => {
                            let method = self.build_method_storage(m, &fqcn, Some(&member.span));
                            own_methods
                                .insert(Arc::from(method.name.to_lowercase().as_str()), method);
                        }
                        ClassMemberKind::ClassConst(c) => {
                            own_constants.insert(
                                Arc::from(c.name),
                                ConstantStorage {
                                    name: c.name.into(),
                                    ty: Union::mixed(),
                                    visibility: c
                                        .visibility
                                        .map(|v| Self::convert_visibility(Some(v))),
                                    location: Some(
                                        self.location(member.span.start, member.span.end),
                                    ),
                                },
                            );
                        }
                        _ => {}
                    }
                }

                self.codebase
                    .symbol_to_file
                    .insert(Arc::from(fqcn.as_str()), self.file.clone());
                self.codebase.interfaces.insert(
                    fqcn.clone().into(),
                    InterfaceStorage {
                        fqcn: fqcn.into(),
                        short_name: decl.name.into(),
                        extends,
                        own_methods,
                        own_constants,
                        template_params: vec![],
                        all_parents: vec![],
                        location: Some(self.location(stmt.span.start, stmt.span.end)),
                    },
                );
            }

            StmtKind::Trait(decl) => {
                let fqcn = self.resolve_name(decl.name);

                let mut own_methods = indexmap::IndexMap::new();
                let mut own_properties = indexmap::IndexMap::new();
                let mut own_constants = indexmap::IndexMap::new();

                for member in decl.members.iter() {
                    match &member.kind {
                        ClassMemberKind::Method(m) => {
                            // Constructor promotion in traits: params with visibility create properties.
                            if m.name == "__construct" {
                                for p in m.params.iter() {
                                    if p.visibility.is_some() {
                                        let ty = self.resolve_union_opt(
                                            p.type_hint
                                                .as_ref()
                                                .map(|h| type_from_hint(h, Some(&fqcn))),
                                        );
                                        let prop = PropertyStorage {
                                            name: p.name.into(),
                                            ty,
                                            inferred_ty: None,
                                            visibility: Self::convert_visibility(p.visibility),
                                            is_static: false,
                                            is_readonly: p.is_readonly,
                                            default: p.default.as_ref().map(|_| Union::mixed()),
                                            location: Some(
                                                self.location(member.span.start, member.span.end),
                                            ),
                                        };
                                        own_properties.insert(p.name.into(), prop);
                                    }
                                }
                            }
                            let method = self.build_method_storage(m, &fqcn, Some(&member.span));
                            own_methods
                                .insert(Arc::from(method.name.to_lowercase().as_str()), method);
                        }
                        ClassMemberKind::Property(p) => {
                            own_properties.insert(
                                Arc::from(p.name),
                                PropertyStorage {
                                    name: p.name.into(),
                                    ty: self.resolve_union_opt(
                                        p.type_hint
                                            .as_ref()
                                            .map(|h| type_from_hint(h, Some(&fqcn))),
                                    ),
                                    inferred_ty: None,
                                    visibility: Self::convert_visibility(p.visibility),
                                    is_static: p.is_static,
                                    is_readonly: p.is_readonly,
                                    default: None,
                                    location: Some(
                                        self.location(member.span.start, member.span.end),
                                    ),
                                },
                            );
                        }
                        ClassMemberKind::ClassConst(c) => {
                            own_constants.insert(
                                Arc::from(c.name),
                                ConstantStorage {
                                    name: c.name.into(),
                                    ty: Union::mixed(),
                                    visibility: None,
                                    location: Some(
                                        self.location(member.span.start, member.span.end),
                                    ),
                                },
                            );
                        }
                        ClassMemberKind::TraitUse(_) => {}
                    }
                }

                self.codebase
                    .symbol_to_file
                    .insert(Arc::from(fqcn.as_str()), self.file.clone());
                self.codebase.traits.insert(
                    fqcn.clone().into(),
                    TraitStorage {
                        fqcn: fqcn.into(),
                        short_name: decl.name.into(),
                        own_methods,
                        own_properties,
                        own_constants,
                        template_params: vec![],
                        location: Some(self.location(stmt.span.start, stmt.span.end)),
                    },
                );
            }

            StmtKind::Enum(decl) => {
                let fqcn = self.resolve_name(decl.name);

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
                                Arc::from(c.name),
                                EnumCaseStorage {
                                    name: c.name.into(),
                                    value: c.value.as_ref().map(|_| Union::mixed()),
                                    location: Some(
                                        self.location(member.span.start, member.span.end),
                                    ),
                                },
                            );
                        }
                        EnumMemberKind::Method(m) => {
                            let method = self.build_method_storage(m, &fqcn, Some(&member.span));
                            own_methods
                                .insert(Arc::from(method.name.to_lowercase().as_str()), method);
                        }
                        EnumMemberKind::ClassConst(c) => {
                            own_constants.insert(
                                Arc::from(c.name),
                                ConstantStorage {
                                    name: c.name.into(),
                                    ty: Union::mixed(),
                                    visibility: None,
                                    location: Some(
                                        self.location(member.span.start, member.span.end),
                                    ),
                                },
                            );
                        }
                        EnumMemberKind::TraitUse(_) => {}
                    }
                }

                self.codebase
                    .symbol_to_file
                    .insert(Arc::from(fqcn.as_str()), self.file.clone());
                self.codebase.enums.insert(
                    fqcn.clone().into(),
                    mir_codebase::EnumStorage {
                        fqcn: fqcn.into(),
                        short_name: decl.name.into(),
                        scalar_type,
                        interfaces,
                        cases,
                        own_methods,
                        own_constants,
                        location: Some(self.location(stmt.span.start, stmt.span.end)),
                    },
                );
            }

            StmtKind::Const(items) => {
                for item in items.iter() {
                    let fqn: Arc<str> = if let Some(ns) = &self.namespace {
                        format!("{}\\{}", ns, item.name).into()
                    } else {
                        item.name.into()
                    };
                    self.codebase.constants.insert(fqn, Union::mixed());
                }
            }

            StmtKind::Block(stmts) => {
                for stmt in stmts.iter() {
                    let _ = self.visit_stmt(stmt);
                }
            }

            _ => {}
        }
        ControlFlow::Continue(())
    }
}

impl<'a> DefinitionCollector<'a> {
    fn build_method_storage(
        &self,
        m: &php_ast::ast::MethodDecl<'_, '_>,
        class_fqcn: &str,
        span: Option<&php_ast::Span>,
    ) -> MethodStorage {
        let doc = span
            .and_then(|s| find_preceding_docblock(self.source, s.start))
            .map(|d| crate::parser::DocblockParser::parse(&d))
            .unwrap_or_default();

        let mut params = Vec::new();
        for p in m.params.iter() {
            let ty = doc
                .get_param_type(p.name)
                .cloned()
                .map(|u| self.resolve_union_doc(u))
                .or_else(|| {
                    self.resolve_union_opt(
                        p.type_hint
                            .as_ref()
                            .map(|h| type_from_hint(h, Some(class_fqcn))),
                    )
                });
            params.push(FnParam {
                name: p.name.into(),
                ty,
                default: p.default.as_ref().map(|_| Union::mixed()),
                is_variadic: p.variadic,
                is_byref: p.by_ref,
                is_optional: p.default.is_some() || p.variadic,
            });
        }

        let return_type = match (doc.return_type.clone(), m.return_type.as_ref()) {
            (Some(mut ty), _) => {
                ty.from_docblock = true;
                let resolved = self.resolve_union_doc(ty);
                Some(Self::fill_self_static_parent(resolved, class_fqcn))
            }
            (None, Some(h)) => self.resolve_union_opt(Some(type_from_hint(h, Some(class_fqcn)))),
            (None, None) => None,
        };

        let template_params: Vec<TemplateParam> = doc
            .templates
            .iter()
            .map(|(name, bound)| TemplateParam {
                name: name.as_str().into(),
                bound: bound.clone(),
                defining_entity: class_fqcn.into(),
            })
            .collect();

        MethodStorage {
            name: m.name.into(),
            fqcn: class_fqcn.into(),
            params,
            return_type,
            inferred_return_type: None,
            visibility: Self::convert_visibility(m.visibility),
            is_static: m.is_static,
            is_abstract: m.is_abstract,
            is_final: m.is_final,
            is_constructor: m.name == "__construct",
            template_params,
            assertions: vec![],
            throws: doc.throws.iter().map(|t| Arc::from(t.as_str())).collect(),
            is_deprecated: doc.is_deprecated,
            is_internal: doc.is_internal,
            is_pure: doc.is_pure,
            location: span.map(|s| self.location(s.start, s.end)),
        }
    }
}
