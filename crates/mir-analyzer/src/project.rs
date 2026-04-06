/// Project-level orchestration: file discovery, pass 1, pass 2.
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rayon::prelude::*;

use crate::cache::{hash_content, AnalysisCache};
use mir_codebase::Codebase;
use mir_issues::Issue;
use mir_types::Union;

use crate::collector::DefinitionCollector;

// ---------------------------------------------------------------------------
// ProjectAnalyzer
// ---------------------------------------------------------------------------

pub struct ProjectAnalyzer {
    pub codebase: Arc<Codebase>,
    /// Optional cache — when `Some`, Pass 2 results are read/written per file.
    pub cache: Option<AnalysisCache>,
    /// Called once after each file completes Pass 2 (used for progress reporting).
    pub on_file_done: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Whether stubs have already been loaded (to avoid double-loading).
    stubs_loaded: std::sync::atomic::AtomicBool,
}

impl ProjectAnalyzer {
    pub fn new() -> Self {
        Self {
            codebase: Arc::new(Codebase::new()),
            cache: None,
            on_file_done: None,
            stubs_loaded: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Create a `ProjectAnalyzer` with a disk-backed cache stored under `cache_dir`.
    pub fn with_cache(cache_dir: &Path) -> Self {
        Self {
            codebase: Arc::new(Codebase::new()),
            cache: Some(AnalysisCache::open(cache_dir)),
            on_file_done: None,
            stubs_loaded: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Expose codebase for external use (e.g., pre-loading stubs from CLI).
    pub fn codebase(&self) -> &Arc<Codebase> {
        &self.codebase
    }

    /// Load PHP built-in stubs. Called automatically by `analyze` if not done yet.
    pub fn load_stubs(&self) {
        if !self
            .stubs_loaded
            .swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            crate::stubs::load_stubs(&self.codebase);
        }
    }

    /// Run the full analysis pipeline on a set of file paths.
    pub fn analyze(&self, paths: &[PathBuf]) -> AnalysisResult {
        let mut all_issues = Vec::new();
        let mut parse_errors = Vec::new();

        // ---- Load PHP built-in stubs (before Pass 1 so user code can override)
        self.load_stubs();

        // ---- Pass 1: read files in parallel ----------------------------------
        let file_data: Vec<(Arc<str>, String)> = paths
            .par_iter()
            .filter_map(|path| match std::fs::read_to_string(path) {
                Ok(src) => Some((Arc::from(path.to_string_lossy().as_ref()), src)),
                Err(e) => {
                    eprintln!("Cannot read {}: {}", path.display(), e);
                    None
                }
            })
            .collect();

        // Definition collection is sequential — DashMap handles concurrent writes,
        // but sequential avoids contention on small projects.
        for (file, src) in &file_data {
            let arena = bumpalo::Bump::new();
            let result = php_rs_parser::parse(&arena, src);

            for err in &result.errors {
                let msg: String = err.to_string();
                parse_errors.push(Issue::new(
                    mir_issues::IssueKind::ParseError { message: msg },
                    mir_issues::Location {
                        file: file.clone(),
                        line: 1,
                        col_start: 0,
                        col_end: 0,
                    },
                ));
            }

            let collector = DefinitionCollector::new(&self.codebase, file.clone(), src);
            let issues = collector.collect(&result.program);
            all_issues.extend(issues);
        }

        all_issues.extend(parse_errors);

        // ---- Finalize codebase (resolve inheritance, build dispatch tables) --
        self.codebase.finalize();

        // ---- Class-level checks (M11) ----------------------------------------
        let analyzed_file_set: std::collections::HashSet<std::sync::Arc<str>> =
            file_data.iter().map(|(f, _)| f.clone()).collect();
        let class_issues =
            crate::class::ClassAnalyzer::with_files(&self.codebase, analyzed_file_set)
                .analyze_all();
        all_issues.extend(class_issues);

        // ---- Pass 2: analyze function/method bodies in parallel (M14) --------
        // Each file is analyzed independently; arena + parse happen inside the
        // rayon closure so there is no cross-thread borrow.
        // When a cache is present, files whose content hash matches a stored
        // entry skip re-analysis entirely (M17).
        let pass2_results: Vec<Vec<Issue>> = file_data
            .par_iter()
            .map(|(file, src)| {
                // Cache lookup
                let issues = if let Some(cache) = &self.cache {
                    let h = hash_content(src);
                    if let Some(cached) = cache.get(file, &h) {
                        cached
                    } else {
                        // Miss — analyze and store
                        let arena = bumpalo::Bump::new();
                        let result = php_rs_parser::parse(&arena, src);
                        let issues = self.analyze_bodies(&result.program, file.clone(), src);
                        cache.put(file, h, issues.clone());
                        issues
                    }
                } else {
                    let arena = bumpalo::Bump::new();
                    let result = php_rs_parser::parse(&arena, src);
                    self.analyze_bodies(&result.program, file.clone(), src)
                };
                if let Some(cb) = &self.on_file_done {
                    cb();
                }
                issues
            })
            .collect();

        for issues in pass2_results {
            all_issues.extend(issues);
        }

        // Persist cache hits/misses to disk
        if let Some(cache) = &self.cache {
            cache.flush();
        }

        // ---- Dead-code detection (M18) --------------------------------------
        let dead_code_issues = crate::dead_code::DeadCodeAnalyzer::new(&self.codebase).analyze();
        all_issues.extend(dead_code_issues);

        AnalysisResult {
            issues: all_issues,
            type_envs: std::collections::HashMap::new(),
        }
    }

    /// Analyze a PHP source string without a real file path.
    /// Useful for tests and LSP single-file mode.
    pub fn analyze_source(source: &str) -> AnalysisResult {
        use crate::collector::DefinitionCollector;
        let analyzer = ProjectAnalyzer::new();
        analyzer.load_stubs();
        let file: Arc<str> = Arc::from("<source>");
        let arena = bumpalo::Bump::new();
        let result = php_rs_parser::parse(&arena, source);
        let mut all_issues = Vec::new();
        let collector = DefinitionCollector::new(&analyzer.codebase, file.clone(), source);
        all_issues.extend(collector.collect(&result.program));
        analyzer.codebase.finalize();
        let mut type_envs = std::collections::HashMap::new();
        all_issues.extend(analyzer.analyze_bodies_typed(
            &result.program,
            file.clone(),
            source,
            &mut type_envs,
        ));
        AnalysisResult {
            issues: all_issues,
            type_envs,
        }
    }

    /// Pass 2: walk all function/method bodies in one file, return issues, and
    /// write inferred return types back to the codebase.
    fn analyze_bodies<'arena, 'src>(
        &self,
        program: &php_ast::ast::Program<'arena, 'src>,
        file: Arc<str>,
        source: &str,
    ) -> Vec<mir_issues::Issue> {
        use php_ast::ast::StmtKind;

        let mut all_issues = Vec::new();

        for stmt in program.stmts.iter() {
            match &stmt.kind {
                StmtKind::Function(decl) => {
                    self.analyze_fn_decl(decl, &file, source, &mut all_issues);
                }
                StmtKind::Class(decl) => {
                    self.analyze_class_decl(decl, &file, source, &mut all_issues);
                }
                StmtKind::Enum(decl) => {
                    self.analyze_enum_decl(decl, &file, source, &mut all_issues);
                }
                StmtKind::Namespace(ns) => {
                    if let php_ast::ast::NamespaceBody::Braced(stmts) = &ns.body {
                        for inner in stmts.iter() {
                            match &inner.kind {
                                StmtKind::Function(decl) => {
                                    self.analyze_fn_decl(decl, &file, source, &mut all_issues);
                                }
                                StmtKind::Class(decl) => {
                                    self.analyze_class_decl(decl, &file, source, &mut all_issues);
                                }
                                StmtKind::Enum(decl) => {
                                    self.analyze_enum_decl(decl, &file, source, &mut all_issues);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        all_issues
    }

    /// Analyze a single function declaration body and collect issues + inferred return type.
    fn analyze_fn_decl<'arena, 'src>(
        &self,
        decl: &php_ast::ast::FunctionDecl<'arena, 'src>,
        file: &Arc<str>,
        source: &str,
        all_issues: &mut Vec<mir_issues::Issue>,
    ) {
        let fn_name = decl.name;
        let body = &decl.body;
        // Check parameter and return type hints for undefined classes.
        for param in decl.params.iter() {
            if let Some(hint) = &param.type_hint {
                check_type_hint_classes(hint, &self.codebase, file, source, all_issues);
            }
        }
        if let Some(hint) = &decl.return_type {
            check_type_hint_classes(hint, &self.codebase, file, source, all_issues);
        }
        use crate::context::Context;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        // Resolve function name using the file's namespace (handles namespaced functions)
        let resolved_fn = self.codebase.resolve_class_name(file.as_ref(), fn_name);
        let func_opt: Option<mir_codebase::storage::FunctionStorage> = self
            .codebase
            .functions
            .get(resolved_fn.as_str())
            .map(|r| r.clone())
            .or_else(|| self.codebase.functions.get(fn_name).map(|r| r.clone()))
            .or_else(|| {
                self.codebase
                    .functions
                    .iter()
                    .find(|e| e.short_name.as_ref() == fn_name)
                    .map(|e| e.value().clone())
            });

        let fqn = func_opt.as_ref().map(|f| f.fqn.clone());
        // Always use the codebase entry when its params match the AST (same count + names).
        // This covers the common case and preserves docblock-enriched types.
        // When names differ (two files define the same unnamespaced function), fall back to
        // the AST params so param variables are always in scope for this file's body.
        let (params, return_ty): (Vec<mir_codebase::FnParam>, _) = match &func_opt {
            Some(f)
                if f.params.len() == decl.params.len()
                    && f.params
                        .iter()
                        .zip(decl.params.iter())
                        .all(|(cp, ap)| cp.name.as_ref() == ap.name) =>
            {
                (f.params.clone(), f.return_type.clone())
            }
            _ => {
                let ast_params = decl
                    .params
                    .iter()
                    .map(|p| mir_codebase::FnParam {
                        name: Arc::from(p.name),
                        ty: None,
                        default: p.default.as_ref().map(|_| mir_types::Union::mixed()),
                        is_variadic: p.variadic,
                        is_byref: p.by_ref,
                        is_optional: p.default.is_some() || p.variadic,
                    })
                    .collect();
                (ast_params, None)
            }
        };

        let mut ctx = Context::for_function(&params, return_ty, None, None, None, false);
        let mut buf = IssueBuffer::new();
        let mut sa = StatementsAnalyzer::new(&self.codebase, file.clone(), source, &mut buf);
        sa.analyze_stmts(body, &mut ctx);
        let inferred = merge_return_types(&sa.return_types);
        drop(sa);

        emit_unused_params(&params, &ctx, false, file, all_issues);
        emit_unused_variables(&ctx, file, all_issues);
        all_issues.extend(buf.into_issues());

        if let Some(fqn) = fqn {
            if let Some(mut func) = self.codebase.functions.get_mut(fqn.as_ref()) {
                func.inferred_return_type = Some(inferred);
            }
        }
    }

    /// Analyze all method bodies on a class declaration and collect issues + inferred return types.
    fn analyze_class_decl<'arena, 'src>(
        &self,
        decl: &php_ast::ast::ClassDecl<'arena, 'src>,
        file: &Arc<str>,
        source: &str,
        all_issues: &mut Vec<mir_issues::Issue>,
    ) {
        use crate::context::Context;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let class_name = decl.name.unwrap_or("<anonymous>");
        // Resolve the FQCN using the file's namespace/imports — avoids ambiguity
        // when multiple classes share the same short name across namespaces.
        let resolved = self.codebase.resolve_class_name(file.as_ref(), class_name);
        let fqcn: &str = &resolved;
        let parent_fqcn = self
            .codebase
            .classes
            .get(fqcn)
            .and_then(|c| c.parent.clone());

        for member in decl.members.iter() {
            let php_ast::ast::ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };

            // Check parameter and return type hints for undefined classes (even abstract methods).
            for param in method.params.iter() {
                if let Some(hint) = &param.type_hint {
                    check_type_hint_classes(hint, &self.codebase, file, source, all_issues);
                }
            }
            if let Some(hint) = &method.return_type {
                check_type_hint_classes(hint, &self.codebase, file, source, all_issues);
            }

            let Some(body) = &method.body else { continue };

            let method_storage = self.codebase.get_method(fqcn, method.name);
            let (params, return_ty) = method_storage
                .as_ref()
                .map(|m| (m.params.clone(), m.return_type.clone()))
                .unwrap_or_default();

            let is_ctor = method.name == "__construct";
            let mut ctx = Context::for_method(
                &params,
                return_ty,
                Some(Arc::from(fqcn)),
                parent_fqcn.clone(),
                Some(Arc::from(fqcn)),
                false,
                is_ctor,
            );

            let mut buf = IssueBuffer::new();
            let mut sa = StatementsAnalyzer::new(&self.codebase, file.clone(), source, &mut buf);
            sa.analyze_stmts(body, &mut ctx);
            let inferred = merge_return_types(&sa.return_types);
            drop(sa);

            emit_unused_params(&params, &ctx, is_ctor, file, all_issues);
            emit_unused_variables(&ctx, file, all_issues);
            all_issues.extend(buf.into_issues());

            if let Some(mut cls) = self.codebase.classes.get_mut(fqcn) {
                if let Some(m) = cls.own_methods.get_mut(method.name) {
                    m.inferred_return_type = Some(inferred);
                }
            }
        }
    }

    /// Like `analyze_bodies` but also populates `type_envs` with per-scope type environments.
    fn analyze_bodies_typed<'arena, 'src>(
        &self,
        program: &php_ast::ast::Program<'arena, 'src>,
        file: Arc<str>,
        source: &str,
        type_envs: &mut std::collections::HashMap<
            crate::type_env::ScopeId,
            crate::type_env::TypeEnv,
        >,
    ) -> Vec<mir_issues::Issue> {
        use php_ast::ast::StmtKind;
        let mut all_issues = Vec::new();
        for stmt in program.stmts.iter() {
            match &stmt.kind {
                StmtKind::Function(decl) => {
                    self.analyze_fn_decl_typed(decl, &file, source, &mut all_issues, type_envs);
                }
                StmtKind::Class(decl) => {
                    self.analyze_class_decl_typed(decl, &file, source, &mut all_issues, type_envs);
                }
                StmtKind::Enum(decl) => {
                    self.analyze_enum_decl(decl, &file, source, &mut all_issues);
                }
                StmtKind::Namespace(ns) => {
                    if let php_ast::ast::NamespaceBody::Braced(stmts) = &ns.body {
                        for inner in stmts.iter() {
                            match &inner.kind {
                                StmtKind::Function(decl) => {
                                    self.analyze_fn_decl_typed(
                                        decl,
                                        &file,
                                        source,
                                        &mut all_issues,
                                        type_envs,
                                    );
                                }
                                StmtKind::Class(decl) => {
                                    self.analyze_class_decl_typed(
                                        decl,
                                        &file,
                                        source,
                                        &mut all_issues,
                                        type_envs,
                                    );
                                }
                                StmtKind::Enum(decl) => {
                                    self.analyze_enum_decl(decl, &file, source, &mut all_issues);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        all_issues
    }

    /// Like `analyze_fn_decl` but also captures a `TypeEnv` for the function scope.
    fn analyze_fn_decl_typed<'arena, 'src>(
        &self,
        decl: &php_ast::ast::FunctionDecl<'arena, 'src>,
        file: &Arc<str>,
        source: &str,
        all_issues: &mut Vec<mir_issues::Issue>,
        type_envs: &mut std::collections::HashMap<
            crate::type_env::ScopeId,
            crate::type_env::TypeEnv,
        >,
    ) {
        use crate::context::Context;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let fn_name = decl.name;
        let body = &decl.body;

        for param in decl.params.iter() {
            if let Some(hint) = &param.type_hint {
                check_type_hint_classes(hint, &self.codebase, file, source, all_issues);
            }
        }
        if let Some(hint) = &decl.return_type {
            check_type_hint_classes(hint, &self.codebase, file, source, all_issues);
        }

        let resolved_fn = self.codebase.resolve_class_name(file.as_ref(), fn_name);
        let func_opt: Option<mir_codebase::storage::FunctionStorage> = self
            .codebase
            .functions
            .get(resolved_fn.as_str())
            .map(|r| r.clone())
            .or_else(|| self.codebase.functions.get(fn_name).map(|r| r.clone()))
            .or_else(|| {
                self.codebase
                    .functions
                    .iter()
                    .find(|e| e.short_name.as_ref() == fn_name)
                    .map(|e| e.value().clone())
            });

        let fqn = func_opt.as_ref().map(|f| f.fqn.clone());
        let (params, return_ty): (Vec<mir_codebase::FnParam>, _) = match &func_opt {
            Some(f)
                if f.params.len() == decl.params.len()
                    && f.params
                        .iter()
                        .zip(decl.params.iter())
                        .all(|(cp, ap)| cp.name.as_ref() == ap.name) =>
            {
                (f.params.clone(), f.return_type.clone())
            }
            _ => {
                let ast_params = decl
                    .params
                    .iter()
                    .map(|p| mir_codebase::FnParam {
                        name: Arc::from(p.name),
                        ty: None,
                        default: p.default.as_ref().map(|_| mir_types::Union::mixed()),
                        is_variadic: p.variadic,
                        is_byref: p.by_ref,
                        is_optional: p.default.is_some() || p.variadic,
                    })
                    .collect();
                (ast_params, None)
            }
        };

        let mut ctx = Context::for_function(&params, return_ty, None, None, None, false);
        let mut buf = IssueBuffer::new();
        let mut sa = StatementsAnalyzer::new(&self.codebase, file.clone(), source, &mut buf);
        sa.analyze_stmts(body, &mut ctx);
        let inferred = merge_return_types(&sa.return_types);
        drop(sa);

        // Capture TypeEnv for this scope
        let scope_name = fqn.clone().unwrap_or_else(|| Arc::from(fn_name));
        type_envs.insert(
            crate::type_env::ScopeId::Function {
                file: file.clone(),
                name: scope_name,
            },
            crate::type_env::TypeEnv::new(ctx.vars.clone()),
        );

        emit_unused_params(&params, &ctx, false, file, all_issues);
        emit_unused_variables(&ctx, file, all_issues);
        all_issues.extend(buf.into_issues());

        if let Some(fqn) = fqn {
            if let Some(mut func) = self.codebase.functions.get_mut(fqn.as_ref()) {
                func.inferred_return_type = Some(inferred);
            }
        }
    }

    /// Like `analyze_class_decl` but also captures a `TypeEnv` per method scope.
    fn analyze_class_decl_typed<'arena, 'src>(
        &self,
        decl: &php_ast::ast::ClassDecl<'arena, 'src>,
        file: &Arc<str>,
        source: &str,
        all_issues: &mut Vec<mir_issues::Issue>,
        type_envs: &mut std::collections::HashMap<
            crate::type_env::ScopeId,
            crate::type_env::TypeEnv,
        >,
    ) {
        use crate::context::Context;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let class_name = decl.name.unwrap_or("<anonymous>");
        let resolved = self.codebase.resolve_class_name(file.as_ref(), class_name);
        let fqcn: &str = &resolved;
        let parent_fqcn = self
            .codebase
            .classes
            .get(fqcn)
            .and_then(|c| c.parent.clone());

        for member in decl.members.iter() {
            let php_ast::ast::ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };

            for param in method.params.iter() {
                if let Some(hint) = &param.type_hint {
                    check_type_hint_classes(hint, &self.codebase, file, source, all_issues);
                }
            }
            if let Some(hint) = &method.return_type {
                check_type_hint_classes(hint, &self.codebase, file, source, all_issues);
            }

            let Some(body) = &method.body else { continue };

            let method_storage = self.codebase.get_method(fqcn, method.name);
            let (params, return_ty) = method_storage
                .as_ref()
                .map(|m| (m.params.clone(), m.return_type.clone()))
                .unwrap_or_default();

            let is_ctor = method.name == "__construct";
            let mut ctx = Context::for_method(
                &params,
                return_ty,
                Some(Arc::from(fqcn)),
                parent_fqcn.clone(),
                Some(Arc::from(fqcn)),
                false,
                is_ctor,
            );

            let mut buf = IssueBuffer::new();
            let mut sa = StatementsAnalyzer::new(&self.codebase, file.clone(), source, &mut buf);
            sa.analyze_stmts(body, &mut ctx);
            let inferred = merge_return_types(&sa.return_types);
            drop(sa);

            // Capture TypeEnv for this method scope
            type_envs.insert(
                crate::type_env::ScopeId::Method {
                    class: Arc::from(fqcn),
                    method: Arc::from(method.name),
                },
                crate::type_env::TypeEnv::new(ctx.vars.clone()),
            );

            emit_unused_params(&params, &ctx, is_ctor, file, all_issues);
            emit_unused_variables(&ctx, file, all_issues);
            all_issues.extend(buf.into_issues());

            if let Some(mut cls) = self.codebase.classes.get_mut(fqcn) {
                if let Some(m) = cls.own_methods.get_mut(method.name) {
                    m.inferred_return_type = Some(inferred);
                }
            }
        }
    }

    /// Discover all `.php` files under a directory, recursively.
    pub fn discover_files(root: &Path) -> Vec<PathBuf> {
        if root.is_file() {
            return vec![root.to_path_buf()];
        }
        let mut files = Vec::new();
        collect_php_files(root, &mut files);
        files
    }

    /// Pass 1 only: collect type definitions from `paths` into the codebase without
    /// analyzing method bodies or emitting issues. Used to load vendor types.
    pub fn collect_types_only(&self, paths: &[PathBuf]) {
        let file_data: Vec<(Arc<str>, String)> = paths
            .par_iter()
            .filter_map(|path| {
                std::fs::read_to_string(path)
                    .ok()
                    .map(|src| (Arc::from(path.to_string_lossy().as_ref()), src))
            })
            .collect();

        for (file, src) in &file_data {
            let arena = bumpalo::Bump::new();
            let result = php_rs_parser::parse(&arena, src);
            let collector = DefinitionCollector::new(&self.codebase, file.clone(), src);
            // Ignore any issues emitted during vendor collection
            let _ = collector.collect(&result.program);
        }
    }

    /// Check type hints in enum methods for undefined classes.
    fn analyze_enum_decl<'arena, 'src>(
        &self,
        decl: &php_ast::ast::EnumDecl<'arena, 'src>,
        file: &Arc<str>,
        source: &str,
        all_issues: &mut Vec<mir_issues::Issue>,
    ) {
        use php_ast::ast::EnumMemberKind;
        for member in decl.members.iter() {
            let EnumMemberKind::Method(method) = &member.kind else {
                continue;
            };
            for param in method.params.iter() {
                if let Some(hint) = &param.type_hint {
                    check_type_hint_classes(hint, &self.codebase, file, source, all_issues);
                }
            }
            if let Some(hint) = &method.return_type {
                check_type_hint_classes(hint, &self.codebase, file, source, all_issues);
            }
        }
    }
}

impl Default for ProjectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Type-hint class existence checker
// ---------------------------------------------------------------------------

/// Walk a `TypeHint` AST node and emit `UndefinedClass` for any named class
/// that does not exist in the codebase.  Skips PHP built-in type keywords.
fn check_type_hint_classes<'arena, 'src>(
    hint: &php_ast::ast::TypeHint<'arena, 'src>,
    codebase: &Codebase,
    file: &Arc<str>,
    source: &str,
    issues: &mut Vec<mir_issues::Issue>,
) {
    use php_ast::ast::TypeHintKind;
    match &hint.kind {
        TypeHintKind::Named(name) => {
            let name_str = crate::parser::name_to_string(name);
            // Skip built-in pseudo-types that are not real classes.
            if is_pseudo_type(&name_str) {
                return;
            }
            let resolved = codebase.resolve_class_name(file.as_ref(), &name_str);
            if !codebase.type_exists(&resolved) {
                let (line, col) = crate::parser::span_to_line_col(source, hint.span);
                issues.push(mir_issues::Issue::new(
                    mir_issues::IssueKind::UndefinedClass { name: resolved },
                    mir_issues::Location {
                        file: file.clone(),
                        line,
                        col_start: col,
                        col_end: col,
                    },
                ));
            }
        }
        TypeHintKind::Nullable(inner) => {
            check_type_hint_classes(inner, codebase, file, source, issues);
        }
        TypeHintKind::Union(parts) | TypeHintKind::Intersection(parts) => {
            for part in parts.iter() {
                check_type_hint_classes(part, codebase, file, source, issues);
            }
        }
        TypeHintKind::Keyword(_, _) => {} // built-in keyword, always valid
    }
}

/// Returns true for names that are PHP pseudo-types / special identifiers, not
/// real classes.
fn is_pseudo_type(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "self"
            | "static"
            | "parent"
            | "null"
            | "true"
            | "false"
            | "never"
            | "void"
            | "mixed"
            | "object"
            | "callable"
            | "iterable"
    )
}

/// Emit `UnusedParam` issues for params that were never read in `ctx`.
/// Skips variadic params, `_`-prefixed names, and constructors.
fn emit_unused_params(
    params: &[mir_codebase::FnParam],
    ctx: &crate::context::Context,
    is_ctor: bool,
    file: &Arc<str>,
    issues: &mut Vec<mir_issues::Issue>,
) {
    if is_ctor {
        return;
    }
    for p in params {
        if p.is_variadic {
            continue;
        }
        let name = p.name.as_ref().trim_start_matches('$');
        if name.starts_with('_') {
            continue;
        }
        if !ctx.read_vars.contains(name) {
            issues.push(mir_issues::Issue::new(
                mir_issues::IssueKind::UnusedParam {
                    name: name.to_string(),
                },
                mir_issues::Location {
                    file: file.clone(),
                    line: 1,
                    col_start: 0,
                    col_end: 0,
                },
            ));
        }
    }
}

fn emit_unused_variables(
    ctx: &crate::context::Context,
    file: &Arc<str>,
    issues: &mut Vec<mir_issues::Issue>,
) {
    // Superglobals are always "used" — skip them
    const SUPERGLOBALS: &[&str] = &[
        "_SERVER", "_GET", "_POST", "_REQUEST", "_SESSION", "_COOKIE", "_FILES", "_ENV", "GLOBALS",
    ];
    for name in &ctx.assigned_vars {
        if ctx.param_names.contains(name) {
            continue;
        }
        if SUPERGLOBALS.contains(&name.as_str()) {
            continue;
        }
        if name.starts_with('_') {
            continue;
        }
        if !ctx.read_vars.contains(name) {
            issues.push(mir_issues::Issue::new(
                mir_issues::IssueKind::UnusedVariable { name: name.clone() },
                mir_issues::Location {
                    file: file.clone(),
                    line: 1,
                    col_start: 0,
                    col_end: 0,
                },
            ));
        }
    }
}

/// Merge a list of return types into a single `Union`.
/// Returns `void` if the list is empty.
pub fn merge_return_types(return_types: &[Union]) -> Union {
    if return_types.is_empty() {
        return Union::single(mir_types::Atomic::TVoid);
    }
    return_types
        .iter()
        .fold(Union::empty(), |acc, t| Union::merge(&acc, t))
}

pub(crate) fn collect_php_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            // Skip symlinks — they can form cycles (e.g. .pnpm-store)
            if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if matches!(
                    name,
                    "vendor" | ".git" | "node_modules" | ".cache" | ".pnpm-store"
                ) {
                    continue;
                }
                collect_php_files(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("php") {
                out.push(path);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// AnalysisResult
// ---------------------------------------------------------------------------

pub struct AnalysisResult {
    pub issues: Vec<Issue>,
    pub type_envs: std::collections::HashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>,
}

impl AnalysisResult {
    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == mir_issues::Severity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == mir_issues::Severity::Warning)
            .count()
    }
}
