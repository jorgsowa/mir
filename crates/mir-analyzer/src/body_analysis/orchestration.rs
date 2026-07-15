use super::*;

impl<'a> BodyAnalyzer<'a> {
    /// body analysis: walk all function/method bodies in one file, return issues, and
    /// write inferred return types back to the codebase.
    pub(crate) fn analyze_bodies(
        &self,
        program: &php_ast::owned::Program,
        file: Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
    ) -> (Vec<Issue>, Vec<ResolvedSymbol>) {
        let mut all_issues = Vec::new();
        let mut all_symbols = Vec::new();

        if self.mode == AnalysisMode::Full {
            check_duplicate_declarations(
                &program.stmts,
                &file,
                source,
                source_map,
                &mut all_issues,
            );
        }

        self.analyze_top_level_stmts(
            &program.stmts,
            &file,
            source,
            source_map,
            &mut all_issues,
            &mut all_symbols,
        );

        // Analyze top-level executable statements in global scope. The
        // inference-only sweep only primes function/method return types; top-
        // level diagnostics and references are produced by the main sweep.
        self.analyze_global_exec(
            program,
            &file,
            source,
            source_map,
            &mut all_issues,
            &mut all_symbols,
        );

        (all_issues, all_symbols)
    }

    /// Analyze top-level executable statements in global scope (Full mode
    /// only). Extracted from [`Self::analyze_bodies`] so the per-scope
    /// tracked query can run it as its own scope.
    pub(crate) fn analyze_global_exec(
        &self,
        program: &php_ast::owned::Program,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        if self.mode != AnalysisMode::Full {
            return;
        }
        use php_ast::owned::StmtKind;

        use crate::flow_state::FlowState;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let mut ctx = FlowState::new();
        let mut buf = IssueBuffer::new();
        let mut sa = StatementsAnalyzer::new(
            self.db,
            file.clone(),
            source,
            source_map,
            &mut buf,
            all_symbols,
            self.php_version,
            self.mode,
        );
        sa.collect_symbols = self.collect_symbols;
        // Braced namespace bodies carry ordinary executable statements
        // (`namespace Shop { $o = new Order(1); }`); walk them like top-level
        // code. Declarations stay skipped at every level — they're analyzed
        // by their own scopes.
        //
        // Only when the file's namespaces are uniform, though: name
        // resolution is file-scoped (`resolve_name` reads the file's first
        // namespace), so exec code inside a second, *different* namespace
        // block would resolve against the wrong prefix and emit bogus
        // diagnostics. Multi-namespace files keep the old skip.
        let mut ns_names: Vec<Option<String>> = Vec::new();
        for stmt in program.stmts.iter() {
            if let StmtKind::Namespace(ns) = &stmt.kind {
                ns_names.push(ns.name.as_ref().map(crate::parser::name_to_string_owned));
            }
        }
        let uniform_namespace = {
            let mut distinct = ns_names.clone();
            distinct.sort();
            distinct.dedup();
            distinct.len() <= 1
        };
        fn exec_stmts(
            sa: &mut crate::stmt::StatementsAnalyzer<'_>,
            ctx: &mut crate::flow_state::FlowState,
            stmts: &[php_ast::owned::Stmt],
            recurse_namespaces: bool,
        ) {
            use php_ast::owned::StmtKind;
            for stmt in stmts.iter() {
                match &stmt.kind {
                    StmtKind::Function(_)
                    | StmtKind::Class(_)
                    | StmtKind::Enum(_)
                    | StmtKind::Interface(_)
                    | StmtKind::Trait(_)
                    | StmtKind::Use(_) => {}
                    StmtKind::Namespace(ns) => {
                        if recurse_namespaces {
                            if let php_ast::owned::NamespaceBody::Braced(body) = &ns.body {
                                exec_stmts(sa, ctx, &body.stmts, recurse_namespaces);
                            }
                        }
                    }
                    // Process Declare so that `declare(strict_types=1)` updates
                    // ctx.strict_types before later executable stmts are analyzed.
                    _ => {
                        sa.analyze_stmt(stmt, ctx);
                    }
                }
            }
        }
        exec_stmts(&mut sa, &mut ctx, &program.stmts, uniform_namespace);
        drop(sa);
        crate::diagnostics::emit_unused_variables(&ctx, file, all_issues);
        all_issues.extend(buf.into_all_issues());
    }

    /// Like `analyze_bodies` but also populates `type_envs` with per-scope type environments.
    pub(crate) fn analyze_bodies_typed(
        &self,
        program: &php_ast::owned::Program,
        file: Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        type_envs: &mut FxHashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) -> Vec<Issue> {
        use php_ast::owned::StmtKind;
        let mut all_issues = Vec::new();
        if self.mode == AnalysisMode::Full {
            check_duplicate_declarations(
                &program.stmts,
                &file,
                source,
                source_map,
                &mut all_issues,
            );
        }
        self.analyze_top_level_stmts_typed(
            &program.stmts,
            &file,
            source,
            source_map,
            &mut all_issues,
            type_envs,
            all_symbols,
        );

        // Analyze top-level executable statements in global scope.
        {
            use crate::flow_state::FlowState;
            use crate::stmt::StatementsAnalyzer;
            use mir_issues::IssueBuffer;

            let mut ctx = FlowState::new();
            let mut buf = IssueBuffer::new();
            let mut sa = StatementsAnalyzer::new(
                self.db,
                file.clone(),
                source,
                source_map,
                &mut buf,
                all_symbols,
                self.php_version,
                self.mode,
            );
            sa.collect_symbols = self.collect_symbols;
            for stmt in program.stmts.iter() {
                match &stmt.kind {
                    StmtKind::Function(_)
                    | StmtKind::Class(_)
                    | StmtKind::Enum(_)
                    | StmtKind::Interface(_)
                    | StmtKind::Trait(_)
                    | StmtKind::Namespace(_)
                    | StmtKind::Use(_) => {}
                    _ => {
                        sa.analyze_stmt(stmt, &mut ctx);
                    }
                }
            }
            drop(sa);
            crate::diagnostics::emit_unused_variables(&ctx, &file, &mut all_issues);
            all_issues.extend(buf.into_all_issues());
        }

        all_issues
    }

    fn analyze_top_level_stmts(
        &self,
        stmts: &[php_ast::owned::Stmt],
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        use php_ast::owned::StmtKind;
        let mut guards: rustc_hash::FxHashSet<std::sync::Arc<str>> =
            rustc_hash::FxHashSet::default();
        for stmt in stmts.iter() {
            match &stmt.kind {
                StmtKind::Function(decl) => {
                    self.analyze_fn_decl(decl, file, source, source_map, all_issues, all_symbols);
                }
                StmtKind::Class(decl) => {
                    self.analyze_class_decl(
                        decl,
                        file,
                        source,
                        source_map,
                        all_issues,
                        all_symbols,
                        &guards,
                    );
                }
                StmtKind::Enum(decl) => {
                    self.analyze_enum_decl(decl, file, source, source_map, all_issues, all_symbols);
                }
                StmtKind::Interface(decl) => {
                    self.analyze_interface_decl(
                        decl,
                        file,
                        source,
                        source_map,
                        all_issues,
                        &guards,
                        all_symbols,
                    );
                }
                StmtKind::Trait(decl) => {
                    self.analyze_trait_decl(
                        decl,
                        file,
                        source,
                        source_map,
                        all_issues,
                        all_symbols,
                    );
                }
                StmtKind::Namespace(ns) => {
                    if let php_ast::owned::NamespaceBody::Braced(inner) = &ns.body {
                        self.analyze_top_level_stmts(
                            &inner.stmts,
                            file,
                            source,
                            source_map,
                            all_issues,
                            all_symbols,
                        );
                    }
                }
                StmtKind::Use(use_decl) => {
                    check_use_decl_casing(
                        use_decl,
                        self.db,
                        file,
                        source,
                        source_map,
                        all_issues,
                        Some(&mut *all_symbols),
                    );
                }
                _ => {}
            }
            accumulate_class_exists_guard(stmt, self.db, file.as_ref(), &mut guards);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn analyze_top_level_stmts_typed(
        &self,
        stmts: &[php_ast::owned::Stmt],
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        type_envs: &mut rustc_hash::FxHashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        use php_ast::owned::StmtKind;
        let mut guards: rustc_hash::FxHashSet<std::sync::Arc<str>> =
            rustc_hash::FxHashSet::default();
        for stmt in stmts.iter() {
            match &stmt.kind {
                StmtKind::Function(decl) => {
                    self.analyze_fn_decl_typed(
                        decl,
                        file,
                        source,
                        source_map,
                        all_issues,
                        type_envs,
                        all_symbols,
                    );
                }
                StmtKind::Class(decl) => {
                    self.analyze_class_decl_typed(
                        decl,
                        file,
                        source,
                        source_map,
                        all_issues,
                        type_envs,
                        all_symbols,
                        &guards,
                    );
                }
                StmtKind::Enum(decl) => {
                    self.analyze_enum_decl_typed(
                        decl,
                        file,
                        source,
                        source_map,
                        all_issues,
                        type_envs,
                        all_symbols,
                    );
                }
                StmtKind::Interface(decl) => {
                    self.analyze_interface_decl(
                        decl,
                        file,
                        source,
                        source_map,
                        all_issues,
                        &guards,
                        all_symbols,
                    );
                }
                StmtKind::Trait(decl) => {
                    self.analyze_trait_decl_typed(
                        decl,
                        file,
                        source,
                        source_map,
                        all_issues,
                        type_envs,
                        all_symbols,
                    );
                }
                StmtKind::Namespace(ns) => {
                    if let php_ast::owned::NamespaceBody::Braced(inner) = &ns.body {
                        self.analyze_top_level_stmts_typed(
                            &inner.stmts,
                            file,
                            source,
                            source_map,
                            all_issues,
                            type_envs,
                            all_symbols,
                        );
                    }
                }
                StmtKind::Use(use_decl) => {
                    check_use_decl_casing(
                        use_decl,
                        self.db,
                        file,
                        source,
                        source_map,
                        all_issues,
                        Some(&mut *all_symbols),
                    );
                }
                _ => {}
            }
            accumulate_class_exists_guard(stmt, self.db, file.as_ref(), &mut guards);
        }
    }
}

/// If `stmt` is an `if (!class_exists('X')) { throw/return; }` guard, insert
/// the proven-to-exist FQCN into `guards` so the immediately following class
/// declaration can skip the UndefinedClass check for that name.
fn accumulate_class_exists_guard(
    stmt: &php_ast::owned::Stmt,
    db: &dyn crate::db::MirDatabase,
    file: &str,
    guards: &mut rustc_hash::FxHashSet<std::sync::Arc<str>>,
) {
    use php_ast::ast::UnaryPrefixOp;
    use php_ast::owned::{ExprKind, StmtKind};

    let StmtKind::If(if_stmt) = &stmt.kind else {
        return;
    };
    // No else/elseif — we only handle the simple guard pattern.
    if !if_stmt.elseif_branches.is_empty() || if_stmt.else_branch.is_some() {
        return;
    }
    // Condition: `!class_exists(...)` / `!interface_exists(...)` / `!trait_exists(...)`
    let ExprKind::UnaryPrefix(u) = &if_stmt.condition.kind else {
        return;
    };
    if u.op != UnaryPrefixOp::BooleanNot {
        return;
    }
    let ExprKind::FunctionCall(call) = &u.operand.kind else {
        return;
    };
    let fn_name = match &call.name.kind {
        ExprKind::Identifier(name) => name.as_ref(),
        _ => return,
    };
    if !matches!(
        fn_name.trim_start_matches('\\'),
        "class_exists" | "interface_exists" | "trait_exists"
    ) {
        return;
    }
    // Then-body must diverge (throw or return).
    if !then_branch_diverges(&if_stmt.then_branch) {
        return;
    }
    if let Some(arg) = call.args.first() {
        if let Some(fqcn) = crate::narrowing::extract_class_fqcn_from_expr(&arg.value, db, file) {
            guards.insert(fqcn);
        }
    }
}

fn then_branch_diverges(stmt: &php_ast::owned::Stmt) -> bool {
    use php_ast::owned::StmtKind;
    match &stmt.kind {
        StmtKind::Throw(_) => true,
        StmtKind::Return(_) => true,
        StmtKind::Block(block) => block
            .stmts
            .iter()
            .any(|s| matches!(s.kind, StmtKind::Throw(_) | StmtKind::Return(_))),
        _ => false,
    }
}
