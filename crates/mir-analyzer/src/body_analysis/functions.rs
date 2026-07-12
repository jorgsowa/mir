use super::*;

impl<'a> BodyAnalyzer<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn analyze_fn_decl(
        &self,
        decl: &php_ast::owned::FunctionDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        crate::attributes::check_function_attributes(
            decl,
            self.db,
            file,
            source,
            source_map,
            all_issues,
            self.mode == AnalysisMode::Full,
            Some(&mut *all_symbols),
        );
        let fn_name = decl.name.as_deref().unwrap_or("").to_string();
        for param in decl.params.iter() {
            if let Some(hint) = &param.type_hint {
                self.check_and_record_type_hint_classes(
                    hint,
                    file,
                    source,
                    source_map,
                    all_issues,
                    Some(&mut *all_symbols),
                );
            }
            if let Some(default_expr) = &param.default {
                check_expr_for_undefined_classes(
                    default_expr,
                    self.db,
                    file,
                    source,
                    source_map,
                    all_issues,
                    self.php_version,
                );
            }
        }
        if let Some(hint) = &decl.return_type {
            self.check_and_record_type_hint_classes(
                hint,
                file,
                source,
                source_map,
                all_issues,
                Some(&mut *all_symbols),
            );
        }
        use crate::flow_state::FlowState;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let resolved = lookup_function_node_for_decl(self.db, file.as_ref(), &fn_name);
        let fqn = resolved.as_ref().map(|(f, _)| f.clone());
        #[allow(clippy::type_complexity)]
        let (params, return_ty, template_params, declared_throws): (
            Arc<[mir_codebase::DeclaredParam]>,
            _,
            Vec<_>,
            Arc<[Arc<str>]>,
        ) = match &resolved {
            Some((_, storage)) => {
                if storage.params.len() == decl.params.len()
                    && storage
                        .params
                        .iter()
                        .zip(decl.params.iter())
                        .all(|(cp, ap)| ap.name.as_deref().unwrap_or("") == cp.name.as_ref())
                {
                    (
                        Arc::clone(&storage.params),
                        storage.return_type.as_deref().cloned(),
                        storage.template_params.clone(),
                        Arc::from(storage.throws.as_slice()),
                    )
                } else {
                    (
                        Arc::from(ast_derived_fn_params(&decl.params)),
                        None,
                        vec![],
                        Arc::from([]),
                    )
                }
            }
            None => (
                Arc::from(ast_derived_fn_params(&decl.params)),
                None,
                vec![],
                Arc::from([]),
            ),
        };

        if self.mode == AnalysisMode::Full {
            self.emit_missing_fn_types(
                decl,
                resolved.as_ref().map(|(_, s)| s),
                file,
                source,
                source_map,
                all_issues,
            );
        }

        self.check_and_record_throws_classes(
            &declared_throws,
            fn_header_name_span(source, decl),
            file,
            source,
            source_map,
            all_issues,
        );

        // A docblock @return that conflicts with the native hint must not
        // make the function's own valid `return` statements look invalid —
        // the native hint is runtime truth. This only affects body-statement
        // checking below; the MismatchingDocblockReturnType check further
        // down in this function still compares against the raw, unfiltered
        // docblock value (via `stored`, independent of `return_ty`).
        let return_ty = super::return_ty_for_body_check(
            self.db,
            file.as_ref(),
            return_ty,
            decl.return_type.as_ref(),
            None,
        );
        let declared_return = return_ty.clone();
        let mut ctx = FlowState::for_method_with_templates(
            &params,
            return_ty,
            declared_throws,
            None,
            None,
            None,
            crate::body_analysis::is_strict_types_file(source),
            false,
            true,
            Some(&template_params),
        );
        ctx.is_in_pure_fn = resolved.as_ref().map(|(_, s)| s.is_pure).unwrap_or(false);
        seed_param_locations(&mut ctx, &decl.params, source, source_map);
        record_param_symbols(all_symbols, file, source, &decl.params, &ctx);
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
        ctx.is_generator = body_has_yield(&decl.body.stmts);
        sa.analyze_stmts(&decl.body.stmts, &mut ctx);
        let inferred = merge_return_types(&sa.return_types);
        let inferred = if sa.yielded_types.is_empty() {
            inferred
        } else {
            build_generator_return_type(&sa.yielded_types, inferred)
        };
        let body_diverges = ctx.diverges;
        drop(sa);

        emit_unused_params(&params, &ctx, "", file, all_issues);
        emit_unused_variables(&ctx, file, all_issues);
        all_issues.extend(buf.into_all_issues());

        if self.mode == AnalysisMode::Full && !ctx.is_generator {
            crate::diagnostics::check_missing_return(
                declared_return.as_ref(),
                body_diverges,
                &decl.body.span,
                file,
                source,
                source_map,
                all_issues,
            );
        }

        if let Some(fqn) = fqn {
            self.record_function_inference(&fqn, &inferred);
        }
    }

    /// Missing type declarations (Psalm parity): a top-level function with
    /// neither a native hint nor a docblock type. `stored` carries the
    /// docblock-resolved types, so absent there + absent in the AST = missing.
    fn emit_missing_fn_types(
        &self,
        decl: &php_ast::owned::FunctionDecl,
        stored: Option<&Arc<mir_codebase::definitions::FunctionDef>>,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        issues: &mut Vec<Issue>,
    ) {
        let fn_name = decl.name.as_deref().unwrap_or("");
        let stored_params_match = stored.is_some_and(|s| s.params.len() == decl.params.len());
        if decl.return_type.is_none()
            && stored.is_none_or(|s| s.return_type.is_none())
            && !fn_name.is_empty()
        {
            let span = fn_header_name_span(source, decl);
            let (line, col_start) =
                crate::diagnostics::offset_to_line_col(source, span.start, source_map);
            let (line_end, col_end) =
                crate::diagnostics::offset_to_line_col(source, span.end, source_map);
            issues.push(mir_issues::Issue::new(
                mir_issues::IssueKind::MissingReturnType {
                    fn_name: fn_name.to_string(),
                },
                mir_issues::Location {
                    file: file.clone(),
                    line,
                    line_end,
                    col_start,
                    col_end: crate::diagnostics::clamp_col_end(line, line_end, col_start, col_end),
                },
            ));
        }
        for (i, ast_param) in decl.params.iter().enumerate() {
            let stored_ty_present = stored_params_match
                && stored.is_some_and(|s| s.params.get(i).is_some_and(|p| p.ty.is_some()));
            if ast_param.type_hint.is_none() && !stored_ty_present {
                let param_name = ast_param
                    .name
                    .as_deref()
                    .unwrap_or("")
                    .trim_start_matches('$')
                    .to_string();
                let span = param_name_span(source, ast_param);
                let (line, col_start) =
                    crate::diagnostics::offset_to_line_col(source, span.start, source_map);
                let (line_end, col_end) =
                    crate::diagnostics::offset_to_line_col(source, span.end, source_map);
                issues.push(mir_issues::Issue::new(
                    mir_issues::IssueKind::MissingParamType {
                        fn_name: fn_name.to_string(),
                        param: param_name,
                    },
                    mir_issues::Location {
                        file: file.clone(),
                        line,
                        line_end,
                        col_start,
                        col_end: crate::diagnostics::clamp_col_end(
                            line, line_end, col_start, col_end,
                        ),
                    },
                ));
            }
        }

        // Docblock signature mismatches (Psalm parity): a docblock type that
        // contradicts the native hint. The stored type is the docblock-resolved
        // one (collector prefers docblock and marks `from_docblock`); the hint
        // is converted and namespace-resolved here for the comparison.
        let Some(stored) = stored else { return };
        let template_names: Vec<&str> = stored
            .template_params
            .iter()
            .map(|tp| tp.name.as_ref())
            .collect();
        if let (Some(hint), Some(doc_ty)) = (&decl.return_type, stored.return_type.as_deref()) {
            if doc_ty.from_docblock
                && !docblock_type_unresolvable(doc_ty, &template_names)
                && !fn_name.is_empty()
            {
                let hint_ty = crate::expr::helpers::resolve_named_objects_in_union(
                    crate::parser::type_from_hint_owned(hint, None),
                    self.db,
                    file.as_ref(),
                );
                if !hint_ty.is_mixed()
                    && !doc_ty.is_mixed()
                    && docblock_conflicts_with_hint(self.db, doc_ty, &hint_ty)
                {
                    let span = fn_header_name_span(source, decl);
                    let (line, col_start) =
                        crate::diagnostics::offset_to_line_col(source, span.start, source_map);
                    let (line_end, col_end) =
                        crate::diagnostics::offset_to_line_col(source, span.end, source_map);
                    issues.push(mir_issues::Issue::new(
                        mir_issues::IssueKind::MismatchingDocblockReturnType {
                            declared: doc_ty.to_string(),
                            inferred: hint_ty.to_string(),
                        },
                        mir_issues::Location {
                            file: file.clone(),
                            line,
                            line_end,
                            col_start,
                            col_end: crate::diagnostics::clamp_col_end(
                                line, line_end, col_start, col_end,
                            ),
                        },
                    ));
                }
            }
        }
        // UndefinedDocblockClass: docblock @return type references a non-existent class.
        if let Some(doc_ty) = stored.return_type.as_deref().filter(|t| t.from_docblock) {
            let span = fn_header_name_span(source, decl);
            let (line, col_start) =
                crate::diagnostics::offset_to_line_col(source, span.start, source_map);
            let (line_end, col_end) =
                crate::diagnostics::offset_to_line_col(source, span.end, source_map);
            for atomic in &doc_ty.types {
                if let mir_types::Atomic::TNamedObject { fqcn, .. } = atomic {
                    if template_names.iter().any(|t| *t == fqcn.as_ref()) {
                        continue;
                    }
                    if !crate::db::class_exists(self.db, fqcn.as_ref()) {
                        issues.push(mir_issues::Issue::new(
                            mir_issues::IssueKind::UndefinedDocblockClass {
                                name: fqcn.to_string(),
                            },
                            mir_issues::Location {
                                file: file.clone(),
                                line,
                                line_end,
                                col_start,
                                col_end: crate::diagnostics::clamp_col_end(
                                    line, line_end, col_start, col_end,
                                ),
                            },
                        ));
                    } else if self.mode == AnalysisMode::Full {
                        self.db.record_reference_location(crate::db::RefLoc {
                            symbol_key: Arc::from(format!("cls:{fqcn}")),
                            file: file.clone(),
                            line,
                            col_start,
                            col_end: crate::diagnostics::clamp_col_end(
                                line, line_end, col_start, col_end,
                            ),
                        });
                    }
                }
            }
        }
        // Param docblock types are not flagged `from_docblock` in storage, so
        // re-parse the doc comment to know which params have a docblock type.
        let doc = decl
            .doc_comment
            .as_ref()
            .map(|c| crate::parser::DocblockParser::parse(&c.text))
            .unwrap_or_default();
        {
            for ast_param in decl.params.iter() {
                let raw_name = ast_param.name.as_deref().unwrap_or_default();
                let (Some(hint), Some(doc_raw)) =
                    (&ast_param.type_hint, doc.get_param_type(raw_name))
                else {
                    continue;
                };
                let doc_ty = crate::expr::helpers::resolve_named_objects_in_union(
                    doc_raw.clone(),
                    self.db,
                    file.as_ref(),
                );
                if docblock_type_unresolvable(&doc_ty, &template_names) {
                    continue;
                }
                let hint_ty = crate::expr::helpers::resolve_named_objects_in_union(
                    crate::parser::type_from_hint_owned(hint, None),
                    self.db,
                    file.as_ref(),
                );
                if hint_ty.is_mixed()
                    || doc_ty.is_mixed()
                    || !docblock_conflicts_with_hint(self.db, &doc_ty, &hint_ty)
                {
                    continue;
                }
                let param_name = ast_param
                    .name
                    .as_deref()
                    .unwrap_or("")
                    .trim_start_matches('$')
                    .to_string();
                let span = param_name_span(source, ast_param);
                let (line, col_start) =
                    crate::diagnostics::offset_to_line_col(source, span.start, source_map);
                let (line_end, col_end) =
                    crate::diagnostics::offset_to_line_col(source, span.end, source_map);
                issues.push(mir_issues::Issue::new(
                    mir_issues::IssueKind::MismatchingDocblockParamType {
                        param: param_name,
                        declared: doc_ty.to_string(),
                        inferred: hint_ty.to_string(),
                    },
                    mir_issues::Location {
                        file: file.clone(),
                        line,
                        line_end,
                        col_start,
                        col_end: crate::diagnostics::clamp_col_end(
                            line, line_end, col_start, col_end,
                        ),
                    },
                ));
            }
        }
        // UndefinedDocblockClass: @param docblock references a non-existent class.
        // Runs separately from the MismatchingDocblockParamType loop because that
        // loop requires both a native hint and a docblock type, while this check
        // only needs a docblock type.
        {
            let fn_span = fn_header_name_span(source, decl);
            let (fn_line, fn_col_start) =
                crate::diagnostics::offset_to_line_col(source, fn_span.start, source_map);
            let (fn_line_end, fn_col_end) =
                crate::diagnostics::offset_to_line_col(source, fn_span.end, source_map);
            // Type alias names defined on this function (@psalm-type / @psalm-import-type).
            // These are not class names and must not be flagged as UndefinedDocblockClass.
            let type_alias_names: rustc_hash::FxHashSet<&str> = doc
                .type_aliases
                .iter()
                .map(|a| a.name.as_str())
                .chain(doc.import_types.iter().map(|i| i.local.as_str()))
                .collect();
            for ast_param in decl.params.iter() {
                let raw_name = ast_param.name.as_deref().unwrap_or_default();
                let Some(doc_raw) = doc.get_param_type(raw_name) else {
                    continue;
                };
                let doc_ty = crate::expr::helpers::resolve_named_objects_in_union(
                    doc_raw.clone(),
                    self.db,
                    file.as_ref(),
                );
                for atomic in &doc_ty.types {
                    if let mir_types::Atomic::TNamedObject { fqcn, .. } = atomic {
                        // Skip pseudo-types (non-falsy-string), callables (pure-callable(…)),
                        // class-constants (Ns\C::A), float-literals (0.3), and namespace-resolved
                        // template params (App\T where T is a declared template).
                        let looks_like_class = !fqcn.contains('-')
                            && !fqcn.contains('(')
                            && !fqcn.contains("::")
                            && !fqcn.contains('.')
                            && !fqcn.starts_with(|c: char| c.is_ascii_digit());
                        let last_segment = fqcn.rsplit('\\').next().unwrap_or(fqcn.as_ref());
                        let is_template = template_names
                            .iter()
                            .any(|t| *t == fqcn.as_ref() || *t == last_segment);
                        let is_alias = type_alias_names.contains(last_segment)
                            || type_alias_names.contains(fqcn.as_ref());
                        if !looks_like_class || is_template || is_alias {
                            continue;
                        }
                        if !crate::db::class_exists(self.db, fqcn.as_ref()) {
                            issues.push(mir_issues::Issue::new(
                                mir_issues::IssueKind::UndefinedDocblockClass {
                                    name: fqcn.to_string(),
                                },
                                mir_issues::Location {
                                    file: file.clone(),
                                    line: fn_line,
                                    line_end: fn_line_end,
                                    col_start: fn_col_start,
                                    col_end: crate::diagnostics::clamp_col_end(
                                        fn_line,
                                        fn_line_end,
                                        fn_col_start,
                                        fn_col_end,
                                    ),
                                },
                            ));
                        } else if self.mode == AnalysisMode::Full {
                            self.db.record_reference_location(crate::db::RefLoc {
                                symbol_key: Arc::from(format!("cls:{fqcn}")),
                                file: file.clone(),
                                line: fn_line,
                                col_start: fn_col_start,
                                col_end: crate::diagnostics::clamp_col_end(
                                    fn_line,
                                    fn_line_end,
                                    fn_col_start,
                                    fn_col_end,
                                ),
                            });
                        }
                    }
                }
            }
        }
    }

    /// Pure entry point: run the same analysis as [`Self::analyze_fn_decl`] for
    /// one function decl, but return the result instead of mutating
    /// caller-owned buffers. Used by the `infer_function` salsa tracked query.
    ///
    /// `ResolvedSymbol`s observed during the walk are intentionally dropped —
    /// symbols are re-walked on demand to keep the cache small.
    ///
    /// Ref-loc isolation: the walk is bracketed by a push/pop of a staging
    /// frame, so only refs produced by *this* call are returned — refs
    /// already staged on the handle (or recorded by a nested tracked query)
    /// are unaffected.
    pub(crate) fn analyze_fn_decl_pure(
        &self,
        decl: &php_ast::owned::FunctionDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
    ) -> crate::db::FunctionInferenceResult {
        use crate::flow_state::FlowState;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        // Isolate this walk's refs in a fresh staging frame; popped at exit.
        self.db.push_ref_loc_frame();

        let mut issues: Vec<Issue> = Vec::new();
        let mut discarded_symbols: Vec<ResolvedSymbol> = Vec::new();

        let fn_name = decl.name.as_deref().unwrap_or("").to_string();
        for param in decl.params.iter() {
            if let Some(hint) = &param.type_hint {
                self.check_and_record_type_hint_classes(
                    hint,
                    file,
                    source,
                    source_map,
                    &mut issues,
                    Some(&mut discarded_symbols),
                );
            }
            if let Some(default_expr) = &param.default {
                check_expr_for_undefined_classes(
                    default_expr,
                    self.db,
                    file,
                    source,
                    source_map,
                    &mut issues,
                    self.php_version,
                );
            }
        }
        if let Some(hint) = &decl.return_type {
            self.check_and_record_type_hint_classes(
                hint,
                file,
                source,
                source_map,
                &mut issues,
                Some(&mut discarded_symbols),
            );
        }

        let resolved = lookup_function_node_for_decl(self.db, file.as_ref(), &fn_name);
        if self.mode == AnalysisMode::Full {
            self.emit_missing_fn_types(
                decl,
                resolved.as_ref().map(|(_, s)| s),
                file,
                source,
                source_map,
                &mut issues,
            );
        }
        #[allow(clippy::type_complexity)]
        let (params, return_ty, template_params, declared_throws): (
            Arc<[mir_codebase::DeclaredParam]>,
            _,
            Vec<_>,
            Arc<[Arc<str>]>,
        ) = match &resolved {
            Some((_, storage))
                if storage.params.len() == decl.params.len()
                    && storage
                        .params
                        .iter()
                        .zip(decl.params.iter())
                        .all(|(cp, ap)| ap.name.as_deref().unwrap_or("") == cp.name.as_ref()) =>
            {
                (
                    Arc::clone(&storage.params),
                    storage.return_type.as_deref().cloned(),
                    storage.template_params.clone(),
                    Arc::from(storage.throws.as_slice()),
                )
            }
            _ => (
                Arc::from(ast_derived_fn_params(&decl.params)),
                None,
                vec![],
                Arc::from([]),
            ),
        };

        self.check_and_record_throws_classes(
            &declared_throws,
            fn_header_name_span(source, decl),
            file,
            source,
            source_map,
            &mut issues,
        );

        let mut ctx = FlowState::for_method_with_templates(
            &params,
            return_ty,
            declared_throws,
            None,
            None,
            None,
            crate::body_analysis::is_strict_types_file(source),
            false,
            true,
            Some(&template_params),
        );
        seed_param_locations(&mut ctx, &decl.params, source, source_map);

        let mut buf = IssueBuffer::new();
        let mut sa = StatementsAnalyzer::new(
            self.db,
            file.clone(),
            source,
            source_map,
            &mut buf,
            &mut discarded_symbols,
            self.php_version,
            self.mode,
        );
        sa.collect_symbols = self.collect_symbols;
        // The symbol buffer above is dropped at return — skip building it.
        sa.collect_symbols = false;
        ctx.is_generator = body_has_yield(&decl.body.stmts);
        sa.analyze_stmts(&decl.body.stmts, &mut ctx);
        let inferred = merge_return_types(&sa.return_types);
        let inferred = if sa.yielded_types.is_empty() {
            inferred
        } else {
            build_generator_return_type(&sa.yielded_types, inferred)
        };
        drop(sa);

        emit_unused_params(&params, &ctx, "", file, &mut issues);
        emit_unused_variables(&ctx, file, &mut issues);
        issues.extend(buf.into_all_issues());

        let ref_locs = self.db.pop_ref_loc_frame();

        crate::db::FunctionInferenceResult {
            issues,
            ref_locs,
            return_type: Some(inferred),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn analyze_fn_decl_typed(
        &self,
        decl: &php_ast::owned::FunctionDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        type_envs: &mut FxHashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        use crate::flow_state::FlowState;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let fn_name = decl.name.as_deref().unwrap_or("").to_string();

        for param in decl.params.iter() {
            if let Some(hint) = &param.type_hint {
                self.check_and_record_type_hint_classes(
                    hint,
                    file,
                    source,
                    source_map,
                    all_issues,
                    Some(&mut *all_symbols),
                );
            }
        }
        if let Some(hint) = &decl.return_type {
            self.check_and_record_type_hint_classes(
                hint,
                file,
                source,
                source_map,
                all_issues,
                Some(&mut *all_symbols),
            );
        }

        let resolved = lookup_function_node_for_decl(self.db, file.as_ref(), &fn_name);
        if self.mode == AnalysisMode::Full {
            self.emit_missing_fn_types(
                decl,
                resolved.as_ref().map(|(_, s)| s),
                file,
                source,
                source_map,
                all_issues,
            );
        }
        let fqn = resolved.as_ref().map(|(f, _)| f.clone());
        let (params, return_ty, declared_throws): (
            Arc<[mir_codebase::DeclaredParam]>,
            _,
            Arc<[Arc<str>]>,
        ) = match &resolved {
            Some((_, storage)) => {
                if storage.params.len() == decl.params.len()
                    && storage
                        .params
                        .iter()
                        .zip(decl.params.iter())
                        .all(|(cp, ap)| ap.name.as_deref().unwrap_or("") == cp.name.as_ref())
                {
                    (
                        Arc::clone(&storage.params),
                        storage.return_type.as_deref().cloned(),
                        Arc::from(storage.throws.as_slice()),
                    )
                } else {
                    (
                        Arc::from(ast_derived_fn_params(&decl.params)),
                        None,
                        Arc::from([]),
                    )
                }
            }
            None => (
                Arc::from(ast_derived_fn_params(&decl.params)),
                None,
                Arc::from([]),
            ),
        };

        self.check_and_record_throws_classes(
            &declared_throws,
            fn_header_name_span(source, decl),
            file,
            source,
            source_map,
            all_issues,
        );

        let mut ctx = FlowState::for_function(
            &params,
            return_ty,
            declared_throws,
            None,
            None,
            None,
            crate::body_analysis::is_strict_types_file(source),
            true,
        );
        ctx.is_in_pure_fn = resolved.as_ref().map(|(_, s)| s.is_pure).unwrap_or(false);
        seed_param_locations(&mut ctx, &decl.params, source, source_map);
        record_param_symbols(all_symbols, file, source, &decl.params, &ctx);
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
        ctx.is_generator = body_has_yield(&decl.body.stmts);
        sa.analyze_stmts(&decl.body.stmts, &mut ctx);
        let inferred = merge_return_types(&sa.return_types);
        let inferred = if sa.yielded_types.is_empty() {
            inferred
        } else {
            build_generator_return_type(&sa.yielded_types, inferred)
        };
        drop(sa);

        let scope_name = fqn.clone().unwrap_or_else(|| Arc::from(fn_name));
        type_envs.insert(
            crate::type_env::ScopeId::Function {
                file: file.clone(),
                name: scope_name,
            },
            crate::type_env::TypeEnv::new(ctx.vars.clone()),
        );

        emit_unused_params(&params, &ctx, "", file, all_issues);
        emit_unused_variables(&ctx, file, all_issues);
        all_issues.extend(buf.into_all_issues());

        if let Some(fqn) = fqn {
            self.record_function_inference(&fqn, &inferred);
        }
    }
}
