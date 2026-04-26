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

use crate::parser::{name_to_string, type_from_hint};
use crate::php_version::PhpVersion;
use mir_codebase::storage::{
    Assertion, AssertionKind, ConstantStorage, EnumCaseStorage, FnParam, FunctionStorage,
    InterfaceStorage, Location, MethodStorage, PropertyStorage, StubSlice, TemplateParam,
    TraitStorage, Visibility,
};
use mir_codebase::{ClassStorage, Codebase};
use mir_issues::{Issue, IssueBuffer};
use mir_types::Union;

// ---------------------------------------------------------------------------
// DefinitionCollector
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub struct DefinitionCollector<'a> {
    /// Optional codebase target. When `Some`, [`Self::collect`] will inject the
    /// accumulated slice into it (backward-compat shim for existing callers).
    /// When `None`, only [`Self::collect_slice`] is valid and the collector is
    /// a pure function from AST to [`StubSlice`].
    codebase: Option<&'a Codebase>,
    slice: StubSlice,
    file: Arc<str>,
    source: &'a str,
    source_map: &'a php_rs_parser::source_map::SourceMap,
    namespace: Option<String>,
    /// `use` aliases: alias → FQCN
    use_aliases: std::collections::HashMap<String, String>,
    issues: IssueBuffer,
    /// When `Some`, stub symbols annotated with `@since`/`@removed` are filtered
    /// against this target version. `None` disables filtering (user code).
    php_version: Option<PhpVersion>,
}

impl<'a> DefinitionCollector<'a> {
    /// Backward-compat constructor: the collector will inject its accumulated
    /// slice into `codebase` when [`Self::collect`] is called.
    pub fn new(
        codebase: &'a Codebase,
        file: Arc<str>,
        source: &'a str,
        source_map: &'a php_rs_parser::source_map::SourceMap,
    ) -> Self {
        let mut s = Self::new_for_slice(file, source, source_map);
        s.codebase = Some(codebase);
        s
    }

    /// Pure-function constructor: the collector accumulates a [`StubSlice`]
    /// without touching any shared state. Use [`Self::collect_slice`] to
    /// retrieve it.
    pub fn new_for_slice(
        file: Arc<str>,
        source: &'a str,
        source_map: &'a php_rs_parser::source_map::SourceMap,
    ) -> Self {
        let slice = StubSlice {
            file: Some(file.clone()),
            ..StubSlice::default()
        };
        Self {
            source_map,
            codebase: None,
            slice,
            file,
            source,
            namespace: None,
            use_aliases: std::collections::HashMap::new(),
            issues: IssueBuffer::new(),
            php_version: None,
        }
    }

    /// Enable `@since`/`@removed` filtering against the given target PHP
    /// version. Used by the stub loader so that symbols introduced after, or
    /// removed at or before, the target version are not registered.
    pub fn with_php_version(mut self, version: PhpVersion) -> Self {
        self.php_version = Some(version);
        self
    }

    /// Returns `true` if a docblock's `@since`/`@removed` tags allow this
    /// symbol to exist at the configured target version. When no target is
    /// configured (user code), always returns `true`.
    fn version_allows(&self, doc: &crate::parser::ParsedDocblock) -> bool {
        match self.php_version {
            Some(v) => v.includes_symbol(doc.since.as_deref(), doc.removed.as_deref()),
            None => true,
        }
    }

    /// Shim: build the slice then inject it into the target codebase (if one
    /// was supplied via [`Self::new`]). Returns the issues accumulated during
    /// Pass 1.
    pub fn collect<'arena, 'src>(mut self, program: &Program<'arena, 'src>) -> Vec<Issue> {
        let _ = self.visit_program(program);
        let issues = self.issues.into_issues();
        if let Some(codebase) = self.codebase {
            codebase.inject_stub_slice(self.slice);
        }
        issues
    }

    /// Pure variant: returns the collected slice and any issues.
    pub fn collect_slice<'arena, 'src>(
        mut self,
        program: &Program<'arena, 'src>,
    ) -> (StubSlice, Vec<Issue>) {
        let _ = self.visit_program(program);
        (self.slice, self.issues.into_issues())
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
                return format!("{resolved}{rest}");
            }
            return resolved.clone();
        }
        // Qualify with namespace
        if let Some(ns) = &self.namespace {
            return format!("{ns}\\{name}");
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
                return format!("{resolved}{rest}");
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

    fn resolve_union_doc_with_aliases(
        &self,
        union: Union,
        aliases: &std::collections::HashMap<String, Union>,
    ) -> Union {
        if aliases.is_empty() {
            return self.resolve_union_doc(union);
        }

        use mir_types::Atomic;
        let from_docblock = union.from_docblock;
        let mut result = Union::empty();
        result.possibly_undefined = union.possibly_undefined;
        result.from_docblock = from_docblock;

        for atomic in union.types {
            match atomic {
                Atomic::TNamedObject { fqcn, type_params } if type_params.is_empty() => {
                    if let Some(alias_ty) = aliases.get(fqcn.as_ref()) {
                        result = Union::merge(&result, alias_ty);
                    } else {
                        result.add_type(self.resolve_atomic_inner(
                            Atomic::TNamedObject { fqcn, type_params },
                            false,
                        ));
                    }
                }
                other => result.add_type(self.resolve_atomic_inner(other, false)),
            }
        }

        result
    }

    fn resolve_union_opt(&self, opt: Option<Union>) -> Option<Union> {
        opt.map(|u| self.resolve_union(u))
    }

    fn build_assertions(&self, doc: &crate::parser::ParsedDocblock) -> Vec<Assertion> {
        let mut assertions = Vec::new();
        assertions.extend(doc.assertions.iter().map(|(param, ty)| Assertion {
            kind: AssertionKind::Assert,
            param: Arc::from(param.as_str()),
            ty: self.resolve_union_doc(ty.clone()),
        }));
        assertions.extend(doc.assertions_if_true.iter().map(|(param, ty)| Assertion {
            kind: AssertionKind::AssertIfTrue,
            param: Arc::from(param.as_str()),
            ty: self.resolve_union_doc(ty.clone()),
        }));
        assertions.extend(doc.assertions_if_false.iter().map(|(param, ty)| Assertion {
            kind: AssertionKind::AssertIfFalse,
            param: Arc::from(param.as_str()),
            ty: self.resolve_union_doc(ty.clone()),
        }));
        assertions
    }

    fn location(&self, start: u32, end: u32) -> Location {
        let lc = self.source_map.offset_to_line_col(start);
        let line = lc.line + 1;
        let byte_offset = start as usize;
        let line_start = if byte_offset == 0 {
            0
        } else {
            self.source[..byte_offset]
                .rfind('\n')
                .map(|p| p + 1)
                .unwrap_or(0)
        };
        let col = self.source[line_start..byte_offset].chars().count() as u16;
        Location::with_line_col(self.file.clone(), start, end, line, col)
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

    fn build_type_aliases(
        &self,
        doc: &crate::parser::ParsedDocblock,
    ) -> std::collections::HashMap<String, Union> {
        let mut aliases = std::collections::HashMap::new();
        for alias in &doc.type_aliases {
            if alias.name.is_empty() || alias.type_expr.is_empty() {
                continue;
            }
            let mut ty = crate::parser::docblock::parse_type_string(&alias.type_expr);
            ty.from_docblock = true;
            aliases.insert(alias.name.clone(), self.resolve_union_doc(ty));
        }

        // Resolve @psalm-import-type declarations.
        // Look first in the codebase (cross-file) then in the slice being built
        // (same-file, for classes defined earlier in the same file).
        for import in &doc.import_types {
            if import.from_class.is_empty() {
                continue;
            }
            let from_resolved =
                self.resolve_type_name(&Arc::from(import.from_class.as_str()), true);
            // Try codebase first (cross-file classes already loaded).
            let resolved = if let Some(cb) = self.codebase {
                cb.classes
                    .get(from_resolved.as_ref())
                    .and_then(|src| src.type_aliases.get(import.original.as_str()).cloned())
            } else {
                None
            };
            // Fall back to slice (same-file, collected earlier in this pass).
            let resolved = resolved.or_else(|| {
                self.slice
                    .classes
                    .iter()
                    .find(|cls| cls.fqcn.as_ref() == from_resolved.as_ref())
                    .and_then(|cls| cls.type_aliases.get(import.original.as_str()).cloned())
            });
            if let Some(ty) = resolved {
                aliases.insert(import.local.clone(), ty);
            }
        }

        aliases
    }

    fn add_docblock_members(
        &self,
        doc: &crate::parser::ParsedDocblock,
        aliases: &std::collections::HashMap<String, Union>,
        class_fqcn: &str,
        own_methods: &mut indexmap::IndexMap<Arc<str>, Arc<MethodStorage>>,
        own_properties: &mut indexmap::IndexMap<Arc<str>, PropertyStorage>,
        location: Option<Location>,
    ) {
        for prop in &doc.properties {
            if prop.name.is_empty() || own_properties.contains_key(prop.name.as_str()) {
                continue;
            }
            let ty = if prop.type_hint.is_empty() {
                None
            } else {
                let mut parsed = crate::parser::docblock::parse_type_string(&prop.type_hint);
                parsed.from_docblock = true;
                Some(self.resolve_union_doc_with_aliases(parsed, aliases))
            };
            own_properties.insert(
                Arc::from(prop.name.as_str()),
                PropertyStorage {
                    name: Arc::from(prop.name.as_str()),
                    ty,
                    inferred_ty: None,
                    visibility: Visibility::Public,
                    is_static: false,
                    is_readonly: prop.read_only,
                    default: None,
                    location: location.clone(),
                },
            );
        }

        for method in &doc.methods {
            if method.name.is_empty() {
                continue;
            }
            let key = Arc::from(method.name.to_lowercase().as_str());
            if own_methods.contains_key(&key) {
                continue;
            }
            let return_type = if method.return_type.is_empty() {
                None
            } else {
                let mut parsed = crate::parser::docblock::parse_type_string(&method.return_type);
                parsed.from_docblock = true;
                Some(Self::fill_self_static_parent(
                    self.resolve_union_doc_with_aliases(parsed, aliases),
                    class_fqcn,
                ))
            };
            let params = method
                .params
                .iter()
                .map(|p| {
                    let ty = if p.type_hint.is_empty() {
                        None
                    } else {
                        let mut parsed = crate::parser::docblock::parse_type_string(&p.type_hint);
                        parsed.from_docblock = true;
                        Some(self.resolve_union_doc_with_aliases(parsed, aliases))
                    };
                    FnParam {
                        name: Arc::from(p.name.as_str()),
                        ty,
                        default: if p.is_optional {
                            Some(Union::mixed())
                        } else {
                            None
                        },
                        is_variadic: p.is_variadic,
                        is_byref: p.is_byref,
                        is_optional: p.is_optional,
                    }
                })
                .collect();
            own_methods.insert(
                key,
                Arc::new(MethodStorage {
                    name: Arc::from(method.name.as_str()),
                    fqcn: Arc::from(class_fqcn),
                    params,
                    return_type,
                    inferred_return_type: None,
                    visibility: Visibility::Public,
                    is_static: method.is_static,
                    is_abstract: false,
                    is_final: false,
                    is_constructor: false,
                    template_params: vec![],
                    assertions: vec![],
                    throws: vec![],
                    deprecated: None,
                    is_internal: false,
                    is_pure: false,
                    location: location.clone(),
                }),
            );
        }
    }

    // -----------------------------------------------------------------------
    // Process statements
    // -----------------------------------------------------------------------

    fn process_stmts<'arena, 'src>(
        &mut self,
        stmts: &php_ast::ast::ArenaVec<'arena, php_ast::ast::Stmt<'arena, 'src>>,
    ) -> ControlFlow<()> {
        for stmt in stmts.iter() {
            self.visit_stmt(stmt)?;
        }
        ControlFlow::Continue(())
    }

    // -----------------------------------------------------------------------
    // Global variable registry
    // -----------------------------------------------------------------------

    /// Scan a single statement: if it is `global $x` with a preceding
    /// `/** @var Type $x */` docblock, register the type in the codebase.
    fn try_collect_global_var_annotation(&mut self, stmt: &php_ast::ast::Stmt<'_, '_>) {
        let php_ast::ast::StmtKind::Global(vars) = &stmt.kind else {
            return;
        };
        let Some(doc_text) = crate::parser::find_preceding_docblock(self.source, stmt.span.start)
        else {
            return;
        };
        let parsed = crate::parser::DocblockParser::parse(&doc_text);
        let Some(var_type) = parsed.var_type else {
            return;
        };
        let resolved_ty = self.resolve_union_doc(var_type);

        for var in vars.iter() {
            if let php_ast::ast::ExprKind::Variable(raw_name) = &var.kind {
                let name = raw_name.as_str().trim_start_matches('$');
                // If @var specifies a variable name, only register when it matches.
                if let Some(ref ann_name) = parsed.var_name {
                    if ann_name != name {
                        continue;
                    }
                }
                self.slice
                    .global_vars
                    .push((Arc::from(name), resolved_ty.clone()));
            }
        }
    }

    /// Scan a list of statements and register any `@var`-annotated `global`
    /// declarations. Used for function bodies where the visitor does not recurse.
    fn scan_stmts_for_global_vars<'arena, 'src>(
        &mut self,
        stmts: &php_ast::ast::ArenaVec<'arena, php_ast::ast::Stmt<'arena, 'src>>,
    ) {
        for stmt in stmts.iter() {
            self.try_collect_global_var_annotation(stmt);
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
                        let flow = self.process_stmts(stmts);
                        self.use_aliases = saved_aliases;
                        flow?;
                    }
                    php_ast::ast::NamespaceBody::Simple => {
                        // Simple namespace — affects all subsequent declarations
                    }
                }
            }

            StmtKind::Use(use_decl) => {
                for item in use_decl.uses.iter() {
                    let full_name = name_to_string(&item.name)
                        .trim_start_matches('\\')
                        .to_string();
                    let alias = item
                        .alias
                        .unwrap_or_else(|| full_name.rsplit('\\').next().unwrap_or(&full_name));
                    self.use_aliases.insert(alias.to_string(), full_name);
                }
            }

            StmtKind::Function(decl) => {
                let short_name = decl.name.to_string();
                let fqn = if let Some(ns) = &self.namespace {
                    format!("{ns}\\{short_name}")
                } else {
                    short_name.clone()
                };

                let doc = decl
                    .doc_comment
                    .as_ref()
                    .map(|c| crate::parser::DocblockParser::parse(c.text))
                    .unwrap_or_default();

                if !self.version_allows(&doc) {
                    return ControlFlow::Continue(());
                }

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
                    .map(|(name, bound, variance)| TemplateParam {
                        name: name.as_str().into(),
                        bound: bound.clone(),
                        defining_entity: fqn.as_str().into(),
                        variance: *variance,
                    })
                    .collect();

                let storage = FunctionStorage {
                    fqn: fqn.clone().into(),
                    short_name: short_name.into(),
                    params,
                    return_type,
                    inferred_return_type: None,
                    template_params,
                    assertions: self.build_assertions(&doc),
                    throws: doc.throws.iter().map(|t| Arc::from(t.as_str())).collect(),
                    deprecated: doc.deprecated.as_deref().map(Arc::from),
                    is_pure: doc.is_pure,
                    location: Some(self.location(stmt.span.start, stmt.span.end)),
                };

                self.slice.functions.push(storage);

                // Scan the function body for `@var`-annotated global declarations.
                self.scan_stmts_for_global_vars(&decl.body);
            }

            StmtKind::Global(_) => {
                // Top-level `global $x` — unusual in PHP but valid.
                self.try_collect_global_var_annotation(stmt);
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

                // The php-rs-parser sometimes attaches a class-level docblock
                // to the first property/method instead of the class itself
                // (same bug as traits). Fall back to scanning the source for
                // the nearest `/** */` before the `class` keyword.
                let class_doc = decl
                    .doc_comment
                    .as_ref()
                    .map(|c| crate::parser::DocblockParser::parse(c.text))
                    .or_else(|| {
                        crate::parser::find_preceding_docblock(self.source, stmt.span.start)
                            .map(|t| crate::parser::DocblockParser::parse(&t))
                    })
                    .unwrap_or_default();

                if !self.version_allows(&class_doc) {
                    return ControlFlow::Continue(());
                }

                let type_aliases = self.build_type_aliases(&class_doc);

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
                            if let Some(method) = self.build_method_storage(
                                m,
                                &fqcn,
                                Some(&member.span),
                                Some(&type_aliases),
                            ) {
                                own_methods.insert(
                                    Arc::from(method.name.to_lowercase().as_str()),
                                    Arc::new(method),
                                );
                            }
                        }
                        ClassMemberKind::Property(p) => {
                            let prop_doc = p
                                .doc_comment
                                .as_ref()
                                .map(|c| crate::parser::DocblockParser::parse(c.text))
                                .or_else(|| {
                                    crate::parser::find_preceding_docblock(
                                        self.source,
                                        member.span.start,
                                    )
                                    .map(|t| crate::parser::DocblockParser::parse(&t))
                                })
                                .unwrap_or_default();
                            if !self.version_allows(&prop_doc) {
                                continue;
                            }
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
                            let const_doc = c
                                .doc_comment
                                .as_ref()
                                .map(|c| crate::parser::DocblockParser::parse(c.text))
                                .or_else(|| {
                                    crate::parser::find_preceding_docblock(
                                        self.source,
                                        member.span.start,
                                    )
                                    .map(|t| crate::parser::DocblockParser::parse(&t))
                                })
                                .unwrap_or_default();
                            if !self.version_allows(&const_doc) {
                                continue;
                            }
                            let constant = ConstantStorage {
                                name: c.name.into(),
                                ty: Union::mixed(),
                                visibility: c.visibility.map(|v| Self::convert_visibility(Some(v))),
                                is_final: c.is_final,
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

                self.add_docblock_members(
                    &class_doc,
                    &type_aliases,
                    &fqcn,
                    &mut own_methods,
                    &mut own_properties,
                    Some(self.location(stmt.span.start, stmt.span.end)),
                );

                let template_params: Vec<TemplateParam> = class_doc
                    .templates
                    .iter()
                    .map(|(name, bound, variance)| TemplateParam {
                        name: name.as_str().into(),
                        bound: bound.clone(),
                        defining_entity: fqcn.as_str().into(),
                        variance: *variance,
                    })
                    .collect();

                let extends_type_args: Vec<mir_types::Union> = class_doc
                    .extends
                    .as_ref()
                    .and_then(|ty| {
                        if let Some(mir_types::Atomic::TNamedObject { type_params, .. }) =
                            ty.types.first()
                        {
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

                let implements_type_args: Vec<(Arc<str>, Vec<mir_types::Union>)> = class_doc
                    .implements
                    .iter()
                    .filter_map(|ty| {
                        if let Some(mir_types::Atomic::TNamedObject { fqcn, type_params }) =
                            ty.types.first()
                        {
                            Some((
                                self.resolve_type_name(fqcn, true),
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

                let storage = ClassStorage {
                    fqcn: fqcn.clone().into(),
                    short_name: short_name.into(),
                    parent,
                    interfaces,
                    traits: trait_uses,
                    own_methods,
                    own_properties,
                    own_constants,
                    mixins: class_doc
                        .mixins
                        .iter()
                        .map(|m| self.resolve_type_name(&Arc::from(m.as_str()), true))
                        .collect(),
                    template_params,
                    extends_type_args,
                    implements_type_args,
                    is_abstract: decl.modifiers.is_abstract,
                    is_final: decl.modifiers.is_final,
                    is_readonly: decl.modifiers.is_readonly,
                    all_parents: vec![],
                    deprecated: class_doc.deprecated.as_deref().map(Arc::from),
                    is_internal: class_doc.is_internal,
                    location: Some(self.location(stmt.span.start, stmt.span.end)),
                    type_aliases: type_aliases
                        .iter()
                        .map(|(k, v)| (Arc::from(k.as_str()), v.clone()))
                        .collect(),
                    pending_import_types: class_doc
                        .import_types
                        .iter()
                        .map(|imp| {
                            let from_resolved =
                                self.resolve_type_name(&Arc::from(imp.from_class.as_str()), true);
                            (
                                Arc::from(imp.local.as_str()),
                                Arc::from(imp.original.as_str()),
                                from_resolved,
                            )
                        })
                        .collect(),
                };

                self.slice.classes.push(storage);
            }

            StmtKind::Interface(decl) => {
                let fqcn = self.resolve_name(decl.name);

                let iface_doc = decl
                    .doc_comment
                    .as_ref()
                    .map(|c| crate::parser::DocblockParser::parse(c.text))
                    .or_else(|| {
                        crate::parser::find_preceding_docblock(self.source, stmt.span.start)
                            .map(|t| crate::parser::DocblockParser::parse(&t))
                    })
                    .unwrap_or_default();

                if !self.version_allows(&iface_doc) {
                    return ControlFlow::Continue(());
                }

                let template_params: Vec<TemplateParam> = iface_doc
                    .templates
                    .iter()
                    .map(|(name, bound, variance)| TemplateParam {
                        name: name.as_str().into(),
                        bound: bound.clone(),
                        defining_entity: fqcn.as_str().into(),
                        variance: *variance,
                    })
                    .collect();

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
                            if let Some(method) =
                                self.build_method_storage(m, &fqcn, Some(&member.span), None)
                            {
                                own_methods.insert(
                                    Arc::from(method.name.to_lowercase().as_str()),
                                    Arc::new(method),
                                );
                            }
                        }
                        ClassMemberKind::ClassConst(c) => {
                            let const_doc = c
                                .doc_comment
                                .as_ref()
                                .map(|c| crate::parser::DocblockParser::parse(c.text))
                                .or_else(|| {
                                    crate::parser::find_preceding_docblock(
                                        self.source,
                                        member.span.start,
                                    )
                                    .map(|t| crate::parser::DocblockParser::parse(&t))
                                })
                                .unwrap_or_default();
                            if !self.version_allows(&const_doc) {
                                continue;
                            }
                            own_constants.insert(
                                Arc::from(c.name),
                                ConstantStorage {
                                    name: c.name.into(),
                                    ty: Union::mixed(),
                                    visibility: c
                                        .visibility
                                        .map(|v| Self::convert_visibility(Some(v))),
                                    is_final: c.is_final,
                                    location: Some(
                                        self.location(member.span.start, member.span.end),
                                    ),
                                },
                            );
                        }
                        _ => {}
                    }
                }

                self.slice.interfaces.push(InterfaceStorage {
                    fqcn: fqcn.into(),
                    short_name: decl.name.into(),
                    extends,
                    own_methods,
                    own_constants,
                    template_params,
                    all_parents: vec![],
                    location: Some(self.location(stmt.span.start, stmt.span.end)),
                });
            }

            StmtKind::Trait(decl) => {
                let fqcn = self.resolve_name(decl.name);

                // The php-rs-parser calls `take_doc_comment` for the trait *after* it has
                // already parsed the trait body.  Method declarations inside the body also
                // call `take_doc_comment`, which can steal the trait-level docblock when the
                // trait has a non-empty body.  Fall back to scanning the raw source for the
                // nearest `/** */` before the `trait` keyword.
                let trait_doc = decl
                    .doc_comment
                    .as_ref()
                    .map(|c| crate::parser::DocblockParser::parse(c.text))
                    .or_else(|| {
                        crate::parser::find_preceding_docblock(self.source, stmt.span.start)
                            .map(|t| crate::parser::DocblockParser::parse(&t))
                    })
                    .unwrap_or_default();

                if !self.version_allows(&trait_doc) {
                    return ControlFlow::Continue(());
                }

                let trait_template_params: Vec<TemplateParam> = trait_doc
                    .templates
                    .iter()
                    .map(|(name, bound, variance)| TemplateParam {
                        name: name.as_str().into(),
                        bound: bound.clone(),
                        defining_entity: fqcn.as_str().into(),
                        variance: *variance,
                    })
                    .collect();

                let mut own_methods = indexmap::IndexMap::new();
                let mut own_properties = indexmap::IndexMap::new();
                let mut own_constants = indexmap::IndexMap::new();
                let mut trait_uses: Vec<Arc<str>> = vec![];

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
                            if let Some(method) =
                                self.build_method_storage(m, &fqcn, Some(&member.span), None)
                            {
                                own_methods.insert(
                                    Arc::from(method.name.to_lowercase().as_str()),
                                    Arc::new(method),
                                );
                            }
                        }
                        ClassMemberKind::Property(p) => {
                            let prop_doc = p
                                .doc_comment
                                .as_ref()
                                .map(|c| crate::parser::DocblockParser::parse(c.text))
                                .or_else(|| {
                                    crate::parser::find_preceding_docblock(
                                        self.source,
                                        member.span.start,
                                    )
                                    .map(|t| crate::parser::DocblockParser::parse(&t))
                                })
                                .unwrap_or_default();
                            if !self.version_allows(&prop_doc) {
                                continue;
                            }
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
                            let const_doc = c
                                .doc_comment
                                .as_ref()
                                .map(|c| crate::parser::DocblockParser::parse(c.text))
                                .or_else(|| {
                                    crate::parser::find_preceding_docblock(
                                        self.source,
                                        member.span.start,
                                    )
                                    .map(|t| crate::parser::DocblockParser::parse(&t))
                                })
                                .unwrap_or_default();
                            if !self.version_allows(&const_doc) {
                                continue;
                            }
                            own_constants.insert(
                                Arc::from(c.name),
                                ConstantStorage {
                                    name: c.name.into(),
                                    ty: Union::mixed(),
                                    visibility: None,
                                    is_final: c.is_final,
                                    location: Some(
                                        self.location(member.span.start, member.span.end),
                                    ),
                                },
                            );
                        }
                        ClassMemberKind::TraitUse(tu) => {
                            for t in tu.traits.iter() {
                                trait_uses.push(self.resolve_name(&name_to_string(t)).into());
                            }
                        }
                    }
                }

                let require_extends: Vec<Arc<str>> = trait_doc
                    .require_extends
                    .iter()
                    .map(|s| self.resolve_type_name(&Arc::from(s.as_str()), true))
                    .collect();
                let require_implements: Vec<Arc<str>> = trait_doc
                    .require_implements
                    .iter()
                    .map(|s| self.resolve_type_name(&Arc::from(s.as_str()), true))
                    .collect();

                self.slice.traits.push(TraitStorage {
                    fqcn: fqcn.into(),
                    short_name: decl.name.into(),
                    own_methods,
                    own_properties,
                    own_constants,
                    template_params: trait_template_params,
                    traits: trait_uses,
                    location: Some(self.location(stmt.span.start, stmt.span.end)),
                    require_extends,
                    require_implements,
                });
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
                                Arc::from(c.name),
                                ConstantStorage {
                                    name: c.name.into(),
                                    ty: Union::mixed(),
                                    visibility: None,
                                    is_final: c.is_final,
                                    location: Some(
                                        self.location(member.span.start, member.span.end),
                                    ),
                                },
                            );
                        }
                        EnumMemberKind::TraitUse(_) => {}
                    }
                }

                self.slice.enums.push(mir_codebase::EnumStorage {
                    fqcn: fqcn.into(),
                    short_name: decl.name.into(),
                    scalar_type,
                    interfaces,
                    cases,
                    own_methods,
                    own_constants,
                    location: Some(self.location(stmt.span.start, stmt.span.end)),
                });
            }

            StmtKind::Const(items) => {
                for item in items.iter() {
                    let const_doc = item
                        .doc_comment
                        .as_ref()
                        .map(|c| crate::parser::DocblockParser::parse(c.text))
                        .or_else(|| {
                            crate::parser::find_preceding_docblock(self.source, item.span.start)
                                .map(|t| crate::parser::DocblockParser::parse(&t))
                        })
                        .unwrap_or_default();
                    if !self.version_allows(&const_doc) {
                        continue;
                    }
                    let fqn: Arc<str> = if let Some(ns) = &self.namespace {
                        format!("{}\\{}", ns, item.name).into()
                    } else {
                        item.name.into()
                    };
                    self.slice.constants.push((fqn, Union::mixed()));
                }
            }

            StmtKind::Block(stmts) => {
                return self.process_stmts(stmts);
            }

            // Collect top-level define('NAME', value) calls as global constants.
            // phpstorm-stubs uses this form extensively in *_defines.php files.
            StmtKind::Expression(expr) => {
                if let php_ast::ast::ExprKind::FunctionCall(call) = &expr.kind {
                    if let php_ast::ast::ExprKind::Identifier(fn_name) = &call.name.kind {
                        if fn_name.eq_ignore_ascii_case("define") {
                            if let Some(name_arg) = call.args.first() {
                                if let php_ast::ast::ExprKind::String(name) = &name_arg.value.kind {
                                    // Check for @since/@removed on the docblock preceding this define().
                                    let define_doc = crate::parser::find_preceding_docblock(
                                        self.source,
                                        stmt.span.start,
                                    )
                                    .map(|t| crate::parser::DocblockParser::parse(&t))
                                    .unwrap_or_default();
                                    if self.version_allows(&define_doc) {
                                        let fqn: Arc<str> = Arc::from(&**name);
                                        self.slice.constants.push((fqn, Union::mixed()));
                                    }
                                }
                            }
                        }
                    }
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
        aliases: Option<&std::collections::HashMap<String, Union>>,
    ) -> Option<MethodStorage> {
        let doc = m
            .doc_comment
            .as_ref()
            .map(|c| crate::parser::DocblockParser::parse(c.text))
            .unwrap_or_default();

        if !self.version_allows(&doc) {
            return None;
        }

        let mut params = Vec::new();
        for p in m.params.iter() {
            let ty = doc
                .get_param_type(p.name)
                .cloned()
                .map(|u| {
                    aliases
                        .map(|a| self.resolve_union_doc_with_aliases(u.clone(), a))
                        .unwrap_or_else(|| self.resolve_union_doc(u))
                })
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
                let resolved = aliases
                    .map(|a| self.resolve_union_doc_with_aliases(ty.clone(), a))
                    .unwrap_or_else(|| self.resolve_union_doc(ty));
                Some(Self::fill_self_static_parent(resolved, class_fqcn))
            }
            (None, Some(h)) => self.resolve_union_opt(Some(type_from_hint(h, Some(class_fqcn)))),
            (None, None) => None,
        };

        let template_params: Vec<TemplateParam> = doc
            .templates
            .iter()
            .map(|(name, bound, variance)| TemplateParam {
                name: name.as_str().into(),
                bound: bound.clone(),
                defining_entity: class_fqcn.into(),
                variance: *variance,
            })
            .collect();

        Some(MethodStorage {
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
            assertions: self.build_assertions(&doc),
            throws: doc.throws.iter().map(|t| Arc::from(t.as_str())).collect(),
            deprecated: doc.deprecated.as_deref().map(Arc::from),
            is_internal: doc.is_internal,
            is_pure: doc.is_pure,
            location: span.map(|s| self.location(s.start, s.end)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mir_codebase::codebase_from_parts;

    const SAMPLE: &str = r#"<?php
namespace App\Demo;

use Stringable;

/**
 * @template T
 */
class Widget implements Stringable {
    public function __construct(public string $name) {}
    public function render(): string { return $this->name; }
}

interface Renderable {
    public function render(): string;
}

trait Colored {
    public string $color;
}

enum Size {
    case Small;
    case Large;
}

function make_widget(string $n): Widget {
    /** @var int $counter */
    global $counter;
    return new Widget($n);
}

const MY_CONST = 42;
"#;

    fn parse_and_collect_old(file: &str, src: &str, codebase: &Codebase) {
        let arena = bumpalo::Bump::new();
        let result = php_rs_parser::parse(&arena, src);
        let collector =
            DefinitionCollector::new(codebase, Arc::from(file), src, &result.source_map);
        let _ = collector.collect(&result.program);
        codebase.finalize();
    }

    fn parse_and_collect_slice(file: &str, src: &str) -> StubSlice {
        let arena = bumpalo::Bump::new();
        let result = php_rs_parser::parse(&arena, src);
        let collector =
            DefinitionCollector::new_for_slice(Arc::from(file), src, &result.source_map);
        let (slice, _) = collector.collect_slice(&result.program);
        slice
    }

    #[test]
    fn codebase_from_parts_produces_same_result_as_mutation() {
        let file = "test.php";

        let slice = parse_and_collect_slice(file, SAMPLE);
        let cb_new = codebase_from_parts(vec![slice]);

        let cb_old = Codebase::new();
        parse_and_collect_old(file, SAMPLE, &cb_old);

        fn sorted<T: Ord + Clone, I: IntoIterator<Item = T>>(xs: I) -> Vec<T> {
            let mut v: Vec<T> = xs.into_iter().collect();
            v.sort();
            v
        }

        let ck = |cb: &Codebase| sorted(cb.classes.iter().map(|e| e.key().clone()));
        let ik = |cb: &Codebase| sorted(cb.interfaces.iter().map(|e| e.key().clone()));
        let tk = |cb: &Codebase| sorted(cb.traits.iter().map(|e| e.key().clone()));
        let ek = |cb: &Codebase| sorted(cb.enums.iter().map(|e| e.key().clone()));
        let fk = |cb: &Codebase| sorted(cb.functions.iter().map(|e| e.key().clone()));
        let nk = |cb: &Codebase| sorted(cb.constants.iter().map(|e| e.key().clone()));
        let sk = |cb: &Codebase| {
            sorted(
                cb.symbol_to_file
                    .iter()
                    .map(|e| (e.key().clone(), e.value().clone())),
            )
        };
        let gk = |cb: &Codebase| sorted(cb.global_vars.iter().map(|e| e.key().clone()));

        assert_eq!(ck(&cb_new), ck(&cb_old), "classes differ");
        assert_eq!(ik(&cb_new), ik(&cb_old), "interfaces differ");
        assert_eq!(tk(&cb_new), tk(&cb_old), "traits differ");
        assert_eq!(ek(&cb_new), ek(&cb_old), "enums differ");
        assert_eq!(fk(&cb_new), fk(&cb_old), "functions differ");
        assert_eq!(nk(&cb_new), nk(&cb_old), "constants differ");
        assert_eq!(sk(&cb_new), sk(&cb_old), "symbol_to_file differs");
        assert_eq!(gk(&cb_new), gk(&cb_old), "global_vars differ");

        // Sanity: file-based fields actually got populated.
        assert!(!cb_new.symbol_to_file.is_empty());
        assert!(cb_new.global_vars.contains_key("counter"));

        // Deep-equal one concrete entry per symbol kind to catch any drift in
        // storage contents that key-only comparison would miss.
        let fqcn = "App\\Demo\\Widget";
        assert_eq!(
            cb_new.classes.get(fqcn).unwrap().value(),
            cb_old.classes.get(fqcn).unwrap().value(),
            "ClassStorage differs for {fqcn}"
        );
        let fn_fqn = "App\\Demo\\make_widget";
        assert_eq!(
            cb_new.functions.get(fn_fqn).unwrap().value(),
            cb_old.functions.get(fn_fqn).unwrap().value(),
            "FunctionStorage differs for {fn_fqn}"
        );
        let iface = "App\\Demo\\Renderable";
        assert_eq!(
            cb_new.interfaces.get(iface).unwrap().value(),
            cb_old.interfaces.get(iface).unwrap().value(),
            "InterfaceStorage differs for {iface}"
        );
        let tr = "App\\Demo\\Colored";
        assert_eq!(
            cb_new.traits.get(tr).unwrap().value(),
            cb_old.traits.get(tr).unwrap().value(),
            "TraitStorage differs for {tr}"
        );
        let enum_fqcn = "App\\Demo\\Size";
        assert_eq!(
            cb_new.enums.get(enum_fqcn).unwrap().value(),
            cb_old.enums.get(enum_fqcn).unwrap().value(),
            "EnumStorage differs for {enum_fqcn}"
        );
    }

    #[test]
    fn trait_require_extends_is_collected() {
        let src = r#"<?php
class Model {}

/**
 * @psalm-require-extends Model
 */
trait HasTimestamps {}
"#;
        let cb = Codebase::new();
        parse_and_collect_old("test.php", src, &cb);
        let tr = cb
            .traits
            .get("HasTimestamps")
            .expect("HasTimestamps should be collected");
        assert_eq!(
            tr.require_extends,
            vec![std::sync::Arc::from("Model")],
            "require_extends should contain Model"
        );
    }

    #[test]
    fn trait_require_extends_via_project_analyzer() {
        let src = r#"<?php
/** @psalm-require-extends Model */
trait HasTimestamps {
    public function touch(): void {}
}

class Model {}

class NotAModel {
    use HasTimestamps;
}
"#;
        let result = crate::test_utils::check(src);
        assert!(
            result.iter().any(|i| i.kind.name() == "InvalidTraitUse"),
            "Expected InvalidTraitUse issue"
        );
    }
}
