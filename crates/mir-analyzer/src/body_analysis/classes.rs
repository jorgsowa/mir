use super::*;

impl<'a> BodyAnalyzer<'a> {
    #[allow(clippy::too_many_arguments)]
    /// Property-member checks shared by the class and trait paths: type-hint
    /// class resolution when a hint is present, `MissingPropertyType`
    /// otherwise (Full mode).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn check_property_member(
        &self,
        prop: &php_ast::owned::PropertyDecl,
        member_span: &php_ast::Span,
        fqcn: &str,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
    ) {
        if let Some(hint) = &prop.type_hint {
            self.check_and_record_type_hint_classes(hint, file, source, source_map, all_issues);
        } else if self.mode == AnalysisMode::Full {
            let prop_name = prop.name.as_deref().unwrap_or("").to_string();
            let (line, col_start) =
                crate::diagnostics::offset_to_line_col(source, member_span.start, source_map);
            let (line_end, col_end) =
                crate::diagnostics::offset_to_line_col(source, member_span.end, source_map);
            all_issues.push(mir_issues::Issue::new(
                mir_issues::IssueKind::MissingPropertyType {
                    class: fqcn.to_string(),
                    property: prop_name,
                },
                mir_issues::Location {
                    file: file.clone(),
                    line,
                    line_end,
                    col_start,
                    col_end: col_end.max(col_start + 1),
                },
            ));
        }
    }

    /// Analyze one class-like member method: hint checks, optional parameter
    /// default-value analysis, FlowState construction, body statement
    /// analysis, unused-param/-var emission, optional return checks, and
    /// inference recording.
    ///
    /// One shared core replaces the six previously copy-pasted blocks
    /// (class / trait / enum × plain / typed). [`MethodScopeCx`] captures the
    /// container-kind divergences so each call site's behavior — including
    /// issue emission *order* — is reproduced exactly.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn analyze_method_scope(
        &self,
        method: &php_ast::owned::MethodDecl,
        cx: &MethodScopeCx,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
        type_envs: Option<&mut FxHashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>>,
    ) {
        use crate::flow_state::FlowState;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let fqcn: &str = cx.fqcn.as_ref();

        for param in method.params.iter() {
            if let Some(hint) = &param.type_hint {
                self.check_and_record_type_hint_classes(hint, file, source, source_map, all_issues);
            }
        }
        if let Some(hint) = &method.return_type {
            self.check_and_record_type_hint_classes(hint, file, source, source_map, all_issues);
        }

        if cx.analyze_param_defaults && method.params.iter().any(|p| p.default.is_some()) {
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
            let mut default_ctx = FlowState::new();
            default_ctx.self_fqcn = Some(cx.fqcn.clone());
            default_ctx.parent_fqcn = cx.parent_fqcn.clone();
            default_ctx.static_fqcn = Some(cx.fqcn.clone());
            for p in method.params.iter() {
                if let Some(default) = &p.default {
                    let mut ea = sa.expr_analyzer(&default_ctx);
                    let _ = ea.analyze(default, &mut default_ctx);
                }
            }
            drop(sa);
            all_issues.extend(buf.into_all_issues());
        }

        let Some(body) = &method.body else { return };
        let method_name = method.name.as_deref().unwrap_or("");

        let (params, return_ty, template_params, declared_throws) =
            method_chain_signature(self.db, fqcn, method_name);

        let declared_return = return_ty.clone();
        let is_ctor = cx.detect_ctor && method_name == "__construct";
        let templates: Option<&[mir_codebase::storage::TemplateParam]> = if cx.with_templates {
            Some(&template_params)
        } else {
            None
        };
        let mut ctx = FlowState::for_method_with_templates(
            &params,
            return_ty,
            declared_throws,
            Some(cx.fqcn.clone()),
            cx.parent_fqcn.clone(),
            Some(cx.fqcn.clone()),
            cx.strict_types,
            is_ctor,
            method.is_static,
            templates,
        );
        // Set is_in_pure_fn if the method is annotated @pure.
        if let Some((_, method_storage)) = crate::db::find_method_in_chain(
            self.db,
            crate::db::Fqcn::from_str(self.db, fqcn),
            &method_name.to_ascii_lowercase(),
        ) {
            ctx.is_in_pure_fn = method_storage.is_pure;
        }

        seed_param_locations(&mut ctx, &method.params, source, source_map);
        record_param_symbols(all_symbols, file, source, &method.params, &ctx);

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
        ctx.is_generator = body_has_yield(&body.stmts);
        sa.analyze_stmts(&body.stmts, &mut ctx);
        let inferred = merge_return_types(&sa.return_types);
        let body_diverges = ctx.diverges;
        drop(sa);

        if let Some(type_envs) = type_envs {
            type_envs.insert(
                crate::type_env::ScopeId::Method {
                    class: cx.fqcn.clone(),
                    method: Arc::from(method_name),
                },
                crate::type_env::TypeEnv::new(ctx.vars.clone()),
            );
        }

        emit_unused_params(&params, &ctx, method_name, file, all_issues);
        emit_unused_variables(&ctx, file, all_issues);
        all_issues.extend(buf.into_all_issues());

        if cx.check_returns && self.mode == AnalysisMode::Full && !is_ctor && !ctx.is_generator {
            crate::diagnostics::check_missing_return(
                declared_return.as_ref(),
                body_diverges,
                &body.span,
                file,
                source,
                source_map,
                all_issues,
            );
        }

        if cx.check_returns
            && self.mode == AnalysisMode::Full
            && method_name.eq_ignore_ascii_case("__tostring")
        {
            crate::diagnostics::check_to_string_return(
                fqcn,
                declared_return.as_ref(),
                &inferred,
                &body.span,
                file,
                source,
                source_map,
                all_issues,
            );
        }

        self.record_method_inference(fqcn, method_name, &inferred);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn analyze_class_decl(
        &self,
        decl: &php_ast::owned::ClassDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
        guards: &rustc_hash::FxHashSet<std::sync::Arc<str>>,
    ) {
        crate::attributes::check_class_attributes(
            decl, self.db, file, source, source_map, all_issues,
        );

        let class_name_owned = decl
            .name
            .as_ref()
            .and_then(|i| i.as_deref())
            .unwrap_or("<anonymous>")
            .to_string();
        let class_name = class_name_owned.as_str();
        let resolved = resolve_name(self.db, file.as_ref(), class_name);
        let fqcn: &str = &resolved;
        let here = crate::db::Fqcn::from_str(self.db, fqcn);
        let parent_fqcn =
            crate::db::find_class_like(self.db, here).and_then(|c| c.parent().cloned());

        crate::attributes::check_parent_in_class_attrs(
            &decl.attributes,
            parent_fqcn.is_some(),
            file,
            source,
            source_map,
            all_issues,
        );

        if let Some(parent) = &decl.extends {
            let parent_str = crate::parser::name_to_string_owned(parent);
            let parent_resolved = resolve_name(self.db, file.as_ref(), &parent_str);
            if !guards.contains(parent_resolved.as_str()) {
                crate::diagnostics::check_name_class_for_extends(
                    parent,
                    self.db,
                    file,
                    source,
                    source_map,
                    all_issues,
                    self.php_version,
                );
            }
        }
        for iface in decl.implements.iter() {
            let iface_str = crate::parser::name_to_string_owned(iface);
            let iface_resolved = resolve_name(self.db, file.as_ref(), &iface_str);
            if !guards.contains(iface_resolved.as_str()) {
                check_name_class(
                    iface,
                    self.db,
                    file,
                    source,
                    source_map,
                    all_issues,
                    self.php_version,
                );
            }
        }

        let scope_cx = MethodScopeCx {
            fqcn: Arc::from(fqcn),
            parent_fqcn: parent_fqcn.clone(),
            detect_ctor: true,
            with_templates: true,
            check_returns: true,
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
    pub(super) fn analyze_class_decl_typed(
        &self,
        decl: &php_ast::owned::ClassDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        type_envs: &mut FxHashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>,
        all_symbols: &mut Vec<ResolvedSymbol>,
        guards: &rustc_hash::FxHashSet<std::sync::Arc<str>>,
    ) {
        let class_name_owned = decl
            .name
            .as_ref()
            .and_then(|i| i.as_deref())
            .unwrap_or("<anonymous>")
            .to_string();
        let class_name = class_name_owned.as_str();
        let resolved = resolve_name(self.db, file.as_ref(), class_name);
        let fqcn: &str = &resolved;
        let here = crate::db::Fqcn::from_str(self.db, fqcn);
        let parent_fqcn =
            crate::db::find_class_like(self.db, here).and_then(|c| c.parent().cloned());

        if let Some(parent) = &decl.extends {
            let parent_str = crate::parser::name_to_string_owned(parent);
            let parent_resolved = resolve_name(self.db, file.as_ref(), &parent_str);
            if !guards.contains(parent_resolved.as_str()) {
                crate::diagnostics::check_name_class_for_extends(
                    parent,
                    self.db,
                    file,
                    source,
                    source_map,
                    all_issues,
                    self.php_version,
                );
            }
        }
        for iface in decl.implements.iter() {
            let iface_str = crate::parser::name_to_string_owned(iface);
            let iface_resolved = resolve_name(self.db, file.as_ref(), &iface_str);
            if !guards.contains(iface_resolved.as_str()) {
                check_name_class(
                    iface,
                    self.db,
                    file,
                    source,
                    source_map,
                    all_issues,
                    self.php_version,
                );
            }
        }

        let scope_cx = MethodScopeCx {
            fqcn: Arc::from(fqcn),
            parent_fqcn: parent_fqcn.clone(),
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

    /// Emit `InvalidTraitUse` issues if this class violates any `@psalm-require-extends` /
    /// `@psalm-require-implements` constraint declared on the traits it uses.
    fn check_trait_constraints(&self, fqcn: &str, file: &Arc<str>, all_issues: &mut Vec<Issue>) {
        let here = crate::db::Fqcn::from_str(self.db, fqcn);
        let Some(class) = crate::db::find_class_like(self.db, here) else {
            return;
        };
        let trait_list: Vec<Arc<str>> = class.class_traits().to_vec();
        let trait_locs: Vec<(Arc<str>, mir_types::Location)> = class.trait_use_locations().to_vec();
        let class_all_parents: Vec<Arc<str>> = crate::db::class_ancestors(self.db, here).0;

        for trait_fqcn in trait_list.iter() {
            let tr_short: Arc<str> = trait_fqcn
                .rsplit('\\')
                .next()
                .map(Arc::from)
                .unwrap_or_else(|| trait_fqcn.clone());

            let make_loc = || {
                trait_locs
                    .iter()
                    .find(|(f, _)| f.as_ref() == trait_fqcn.as_ref())
                    .map(|(_, loc)| mir_issues::Location {
                        file: loc.file.clone(),
                        line: loc.line,
                        line_end: loc.line_end,
                        col_start: loc.col_start,
                        col_end: loc.col_end,
                    })
                    .unwrap_or_else(|| mir_issues::Location {
                        file: file.clone(),
                        line: 1,
                        line_end: 1,
                        col_start: 0,
                        col_end: 0,
                    })
            };

            let trait_here = crate::db::Fqcn::from_str(self.db, trait_fqcn.as_ref());
            let trait_class = match crate::db::find_class_like(self.db, trait_here) {
                None => {
                    all_issues.push(mir_issues::Issue::new(
                        mir_issues::IssueKind::UndefinedTrait {
                            name: tr_short.to_string(),
                        },
                        make_loc(),
                    ));
                    continue;
                }
                Some(c) => c,
            };

            if !trait_class.is_trait() {
                let (article, kind) = if trait_class.is_interface() {
                    ("an", "interface")
                } else if trait_class.is_enum() {
                    ("an", "enum")
                } else {
                    ("a", "class")
                };
                all_issues.push(mir_issues::Issue::new(
                    mir_issues::IssueKind::InvalidTraitUse {
                        trait_name: tr_short.to_string(),
                        reason: format!("{tr_short} is {article} {kind}, not a trait"),
                    },
                    make_loc(),
                ));
                continue;
            }

            let (req_ext, req_impl): (Vec<Arc<str>>, Vec<Arc<str>>) = match &trait_class {
                crate::db::ClassLike::Trait(t) => {
                    (t.require_extends.to_vec(), t.require_implements.to_vec())
                }
                _ => (vec![], vec![]),
            };
            if req_ext.is_empty() && req_impl.is_empty() {
                continue;
            }

            for req in req_ext.iter() {
                let satisfies = fqcn == req.as_ref()
                    || class_all_parents.iter().any(|p| p.as_ref() == req.as_ref());
                if !satisfies {
                    all_issues.push(mir_issues::Issue::new(
                        mir_issues::IssueKind::InvalidTraitUse {
                            trait_name: tr_short.to_string(),
                            reason: format!(
                                "Class {fqcn} uses trait {tr_short} but does not extend {req}"
                            ),
                        },
                        make_loc(),
                    ));
                }
            }

            for req in req_impl.iter() {
                let satisfies = class_all_parents.iter().any(|p| p.as_ref() == req.as_ref());
                if !satisfies {
                    all_issues.push(mir_issues::Issue::new(
                        mir_issues::IssueKind::InvalidTraitUse {
                            trait_name: tr_short.to_string(),
                            reason: format!(
                                "Class {fqcn} uses trait {tr_short} but does not implement {req}"
                            ),
                        },
                        make_loc(),
                    ));
                }
            }
        }
    }
}
