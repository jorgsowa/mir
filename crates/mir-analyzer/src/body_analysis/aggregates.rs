use super::*;

impl<'a> BodyAnalyzer<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn analyze_trait_decl(
        &self,
        decl: &php_ast::owned::TraitDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        crate::attributes::check_trait_attributes(
            decl,
            self.db,
            file,
            source,
            source_map,
            all_issues,
            self.mode == AnalysisMode::Full,
            Some(&mut *all_symbols),
        );

        let resolved = resolve_name(self.db, file.as_ref(), decl.name.as_deref().unwrap_or(""));
        let fqcn: &str = &resolved;
        self.check_class_generic_type_args(
            &decl.doc_comment,
            fqcn,
            file,
            source,
            source_map,
            all_issues,
        );

        let scope_cx = MethodScopeCx {
            fqcn: Arc::from(fqcn),
            parent_fqcn: None,
            detect_ctor: true,
            with_templates: false,
            check_returns: false,
            analyze_param_defaults: true,
            strict_types: crate::body_analysis::is_strict_types_file(source),
        };
        for member in decl.body.members.iter() {
            if let php_ast::owned::ClassMemberKind::Property(prop) = &member.kind {
                self.check_property_member(
                    prop,
                    &member.span,
                    fqcn,
                    file,
                    source,
                    source_map,
                    all_issues,
                );
                continue;
            }
            let php_ast::owned::ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };
            self.analyze_method_scope(
                method,
                &scope_cx,
                file,
                source,
                source_map,
                all_issues,
                all_symbols,
                None,
            );
        }
        self.check_trait_constraints(fqcn, file, all_issues);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn analyze_trait_decl_typed(
        &self,
        decl: &php_ast::owned::TraitDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        type_envs: &mut FxHashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        crate::attributes::check_trait_attributes(
            decl,
            self.db,
            file,
            source,
            source_map,
            all_issues,
            self.mode == AnalysisMode::Full,
            Some(&mut *all_symbols),
        );

        let resolved = resolve_name(self.db, file.as_ref(), decl.name.as_deref().unwrap_or(""));
        let fqcn: &str = &resolved;
        self.check_class_generic_type_args(
            &decl.doc_comment,
            fqcn,
            file,
            source,
            source_map,
            all_issues,
        );

        let scope_cx = MethodScopeCx {
            fqcn: Arc::from(fqcn),
            parent_fqcn: None,
            detect_ctor: true,
            with_templates: false,
            check_returns: false,
            analyze_param_defaults: true,
            strict_types: crate::body_analysis::is_strict_types_file(source),
        };
        for member in decl.body.members.iter() {
            if let php_ast::owned::ClassMemberKind::Property(prop) = &member.kind {
                self.check_property_member(
                    prop,
                    &member.span,
                    fqcn,
                    file,
                    source,
                    source_map,
                    all_issues,
                );
                continue;
            }
            let php_ast::owned::ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };
            self.analyze_method_scope(
                method,
                &scope_cx,
                file,
                source,
                source_map,
                all_issues,
                all_symbols,
                Some(&mut *type_envs),
            );
        }
        self.check_trait_constraints(fqcn, file, all_issues);
    }

    /// Analyze each enum case's value expression against a minimal FlowState
    /// scoped to the enum itself. Case values are constant expressions that
    /// may reference class constants (`case Active = Config::VALUE;`) or
    /// other classes — the enum-analysis loop previously matched only
    /// `EnumMemberKind::Method`, so `Case` values were never walked and an
    /// undefined reference there (a genuine PHP fatal on first touch of the
    /// enum) went completely unflagged.
    #[allow(clippy::too_many_arguments)]
    fn analyze_enum_case_values(
        &self,
        decl: &php_ast::owned::EnumDecl,
        fqcn: &str,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        use crate::flow_state::FlowState;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;
        use php_ast::owned::EnumMemberKind;

        let has_case_value = decl
            .body
            .members
            .iter()
            .any(|m| matches!(&m.kind, EnumMemberKind::Case(c) if c.value.is_some()));
        if !has_case_value {
            return;
        }
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
        let mut ctx = FlowState::new();
        ctx.self_fqcn = Some(Arc::from(fqcn));
        ctx.static_fqcn = Some(Arc::from(fqcn));
        for member in decl.body.members.iter() {
            if let EnumMemberKind::Case(case) = &member.kind {
                if let Some(value) = &case.value {
                    let mut ea = sa.expr_analyzer(&ctx);
                    let _ = ea.analyze(value, &mut ctx);
                }
            }
        }
        drop(sa);
        all_issues.extend(buf.into_all_issues());
    }

    pub(crate) fn analyze_enum_decl(
        &self,
        decl: &php_ast::owned::EnumDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        use php_ast::owned::EnumMemberKind;

        crate::attributes::check_enum_attributes(
            decl,
            self.db,
            file,
            source,
            source_map,
            all_issues,
            self.mode == AnalysisMode::Full,
            Some(&mut *all_symbols),
        );

        for iface in decl.implements.iter() {
            check_name_class(
                iface,
                self.db,
                file,
                source,
                source_map,
                all_issues,
                self.php_version,
                self.mode == AnalysisMode::Full,
                all_symbols,
            );
        }

        let enum_name = decl.name.as_deref().unwrap_or("<anonymous>");
        let resolved = resolve_name(self.db, file.as_ref(), enum_name);
        let fqcn: &str = &resolved;

        let scope_cx = MethodScopeCx {
            fqcn: Arc::from(fqcn),
            parent_fqcn: None,
            detect_ctor: false,
            with_templates: false,
            check_returns: false,
            analyze_param_defaults: true,
            strict_types: crate::body_analysis::is_strict_types_file(source),
        };
        for member in decl.body.members.iter() {
            let EnumMemberKind::Method(method) = &member.kind else {
                continue;
            };
            self.analyze_method_scope(
                method,
                &scope_cx,
                file,
                source,
                source_map,
                all_issues,
                all_symbols,
                None,
            );
        }

        self.analyze_enum_case_values(
            decl,
            fqcn,
            file,
            source,
            source_map,
            all_issues,
            all_symbols,
        );
        self.check_trait_constraints(fqcn, file, all_issues);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn analyze_enum_decl_typed(
        &self,
        decl: &php_ast::owned::EnumDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        type_envs: &mut rustc_hash::FxHashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        use php_ast::owned::EnumMemberKind;

        crate::attributes::check_enum_attributes(
            decl,
            self.db,
            file,
            source,
            source_map,
            all_issues,
            self.mode == AnalysisMode::Full,
            Some(&mut *all_symbols),
        );

        // Single pass: same analysis as the untyped path, additionally
        // recording type environments for LSP hover/go-to-def. (Previously
        // this ran the full body analysis twice and discarded the second
        // run's issues — the shared core makes one run produce both.)
        for iface in decl.implements.iter() {
            check_name_class(
                iface,
                self.db,
                file,
                source,
                source_map,
                all_issues,
                self.php_version,
                self.mode == AnalysisMode::Full,
                all_symbols,
            );
        }

        let enum_name = decl.name.as_deref().unwrap_or("<anonymous>");
        let resolved = resolve_name(self.db, file.as_ref(), enum_name);
        let fqcn: &str = &resolved;

        let scope_cx = MethodScopeCx {
            fqcn: Arc::from(fqcn),
            parent_fqcn: None,
            detect_ctor: false,
            with_templates: false,
            check_returns: false,
            analyze_param_defaults: true,
            strict_types: crate::body_analysis::is_strict_types_file(source),
        };
        for member in decl.body.members.iter() {
            let EnumMemberKind::Method(method) = &member.kind else {
                continue;
            };
            self.analyze_method_scope(
                method,
                &scope_cx,
                file,
                source,
                source_map,
                all_issues,
                all_symbols,
                Some(&mut *type_envs),
            );
        }

        self.analyze_enum_case_values(
            decl,
            fqcn,
            file,
            source,
            source_map,
            all_issues,
            all_symbols,
        );
        self.check_trait_constraints(fqcn, file, all_issues);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn analyze_interface_decl(
        &self,
        decl: &php_ast::owned::InterfaceDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        guards: &rustc_hash::FxHashSet<std::sync::Arc<str>>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        crate::attributes::check_interface_attributes(
            decl,
            self.db,
            file,
            source,
            source_map,
            all_issues,
            self.mode == AnalysisMode::Full,
            None,
        );
        let iface_name = decl.name.as_deref().unwrap_or("<anonymous>");
        let iface_fqcn = resolve_name(self.db, file.as_ref(), iface_name);
        self.check_class_generic_type_args(
            &decl.doc_comment,
            &iface_fqcn,
            file,
            source,
            source_map,
            all_issues,
        );
        use php_ast::owned::ClassMemberKind;
        for parent in decl.extends.iter() {
            // Suppress UndefinedClass for a parent guarded by
            // `class_exists`/`interface_exists`/`trait_exists`, mirroring
            // `analyze_class_decl`'s extends/implements handling.
            let parent_str = crate::parser::name_to_string_owned(parent);
            let parent_resolved = resolve_name(self.db, file.as_ref(), &parent_str);
            if guards.contains(parent_resolved.as_str()) {
                continue;
            }
            check_name_class(
                parent,
                self.db,
                file,
                source,
                source_map,
                all_issues,
                self.php_version,
                self.mode == AnalysisMode::Full,
                all_symbols,
            );
        }
        let iface_fqcn_ref = crate::db::Fqcn::from_str(self.db, &iface_fqcn);

        for member in decl.body.members.iter() {
            let ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };
            for param in method.params.iter() {
                if let Some(hint) = &param.type_hint {
                    self.check_and_record_type_hint_classes(
                        hint, file, source, source_map, all_issues, None,
                    );
                }
            }
            if let Some(hint) = &method.return_type {
                self.check_and_record_type_hint_classes(
                    hint, file, source, source_map, all_issues, None,
                );
            }

            let method_name = method.name.as_deref().unwrap_or("");
            let stored = crate::db::find_method_in_class(self.db, iface_fqcn_ref, method_name);

            if self.mode == AnalysisMode::Full {
                let stored_return = stored.as_ref().and_then(|m| m.return_type.as_deref());
                if method.return_type.is_none() && stored_return.is_none() {
                    let fn_name = format!("{iface_fqcn}::{method_name}");
                    let (line, col_start) = crate::diagnostics::offset_to_line_col(
                        source,
                        member.span.start,
                        source_map,
                    );
                    let (line_end, col_end) =
                        crate::diagnostics::offset_to_line_col(source, member.span.end, source_map);
                    all_issues.push(mir_issues::Issue::new(
                        mir_issues::IssueKind::MissingReturnType { fn_name },
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

                if let Some(stored_method) = &stored {
                    let fn_name = format!("{iface_fqcn}::{method_name}");
                    for (ast_param, stored_param) in
                        method.params.iter().zip(stored_method.params.iter())
                    {
                        if ast_param.type_hint.is_none() && stored_param.ty.is_none() {
                            let param_name = ast_param
                                .name
                                .as_deref()
                                .unwrap_or("")
                                .trim_start_matches('$')
                                .to_string();
                            let span = param_name_span(source, ast_param);
                            let (line, col_start) = crate::diagnostics::offset_to_line_col(
                                source, span.start, source_map,
                            );
                            let (line_end, col_end) = crate::diagnostics::offset_to_line_col(
                                source, span.end, source_map,
                            );
                            all_issues.push(mir_issues::Issue::new(
                                mir_issues::IssueKind::MissingParamType {
                                    fn_name: fn_name.clone(),
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
                }
            }
        }
    }
}
