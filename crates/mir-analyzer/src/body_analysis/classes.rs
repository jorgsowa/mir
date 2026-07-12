use super::*;

/// Recursively collects every `TNamedObject` FQCN in `ty`, including ones
/// nested inside its own type-argument list (e.g. `Box<Wrapper<Foo>>` yields
/// `Box`, `Wrapper`, and `Foo`).
fn collect_named_object_fqcns(ty: &mir_types::Type, out: &mut Vec<mir_types::Name>) {
    for atomic in &ty.types {
        if let mir_types::Atomic::TNamedObject { fqcn, type_params } = atomic {
            out.push(*fqcn);
            for tp in type_params.iter() {
                collect_named_object_fqcns(tp, out);
            }
        }
    }
}

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
        } else {
            self.check_property_docblock_classes(
                prop,
                member_span,
                fqcn,
                file,
                source,
                source_map,
                all_issues,
            );
            if self.mode == AnalysisMode::Full {
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
                        col_end: crate::diagnostics::clamp_col_end(
                            line, line_end, col_start, col_end,
                        ),
                    },
                ));
            }
        }
    }

    /// `UndefinedDocblockClass`/`cls:` usage for a property's `@var` docblock
    /// type when it has no native type hint (the native-hint path is checked
    /// via `check_and_record_type_hint_classes` instead). Reuses the
    /// collector-resolved `PropertyDef.ty`, which already has `@var` applied.
    #[allow(clippy::too_many_arguments)]
    fn check_property_docblock_classes(
        &self,
        prop: &php_ast::owned::PropertyDecl,
        member_span: &php_ast::Span,
        fqcn: &str,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
    ) {
        if self.mode != AnalysisMode::Full {
            return;
        }
        let prop_name = prop.name.as_deref().unwrap_or("");
        if prop_name.is_empty() {
            return;
        }
        let key = crate::db::Fqcn::from_str(self.db, fqcn);
        let Some(def) = crate::db::find_property_in_class(self.db, key, prop_name) else {
            return;
        };
        let Some(ty) = def.ty.as_deref() else {
            return;
        };
        let (line, col_start) =
            crate::diagnostics::offset_to_line_col(source, member_span.start, source_map);
        let (line_end, col_end) =
            crate::diagnostics::offset_to_line_col(source, member_span.end, source_map);
        for atomic in &ty.types {
            if let mir_types::Atomic::TNamedObject { fqcn: cls_fqcn, .. } = atomic {
                if crate::diagnostics::is_pseudo_type(cls_fqcn.as_ref()) {
                    continue;
                }
                if !crate::db::class_exists(self.db, cls_fqcn.as_ref()) {
                    all_issues.push(mir_issues::Issue::new(
                        mir_issues::IssueKind::UndefinedDocblockClass {
                            name: cls_fqcn.to_string(),
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
                        symbol_key: Arc::from(format!("cls:{cls_fqcn}")),
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

    /// `UndefinedDocblockClass` for a method's own `@return` docblock type.
    /// Free functions already get this check (`analyze_fn_decl`) against
    /// their collector-resolved signature; methods never did, so
    /// `/** @return UndefinedClass */` on a method silently passed even
    /// though the identical tag on a free function is flagged.
    ///
    /// Deliberately reuses `method_chain_signature`'s already-resolved
    /// return type (the same value `FlowState` is built from) rather than
    /// re-parsing the raw docblock: that value has already had `@template`
    /// references substituted to `TTemplateParam` and `@psalm-type`/
    /// `@phpstan-type` aliases expanded by the collector, so any
    /// `TNamedObject` still present genuinely names a class — no need to
    /// re-derive template/alias awareness here. `@param` docblock types are
    /// NOT checked here for the same reason function.rs gives for re-parsing
    /// them separately: unlike the return type, storage doesn't reliably
    /// flag which param types came from a docblock, and reusing the
    /// resolved type unconditionally would double-report a native-hint
    /// class already caught by `check_and_record_type_hint_classes`.
    fn check_method_docblock_classes(
        &self,
        method: &php_ast::owned::MethodDecl,
        fqcn: &str,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
    ) {
        if self.mode != AnalysisMode::Full {
            return;
        }
        let method_name = method.name.as_deref().unwrap_or("");
        if method_name.is_empty() {
            return;
        }
        let (_, return_ty, _, _) = method_chain_signature(self.db, fqcn, method_name);
        let Some(doc_ty) = return_ty.filter(|t| t.from_docblock) else {
            return;
        };
        if !doc_ty
            .types
            .iter()
            .any(|a| matches!(a, mir_types::Atomic::TNamedObject { .. }))
        {
            return;
        }
        let header_span = method_header_name_span(source, method);
        let (line, col_start) =
            crate::diagnostics::offset_to_line_col(source, header_span.start, source_map);
        let (line_end, col_end) =
            crate::diagnostics::offset_to_line_col(source, header_span.end, source_map);
        let header_location = mir_issues::Location {
            file: file.clone(),
            line,
            line_end,
            col_start,
            col_end: crate::diagnostics::clamp_col_end(line, line_end, col_start, col_end),
        };
        for atomic in &doc_ty.types {
            if let mir_types::Atomic::TNamedObject { fqcn: cls_fqcn, .. } = atomic {
                // `static<T, ...>` (a templated late-static-binding return) can
                // surface as a literal `TNamedObject("static")` rather than
                // `TStaticObject` — a method-only shape a free function's
                // return type can never produce, so this pseudo-name was
                // never filtered before. `self`/`parent` guarded the same way
                // for consistency, though only reachable via `@return` here.
                if matches!(
                    crate::util::php_ident_lowercase(cls_fqcn.as_ref()).as_str(),
                    "self" | "static" | "parent"
                ) {
                    continue;
                }
                if !crate::db::class_exists(self.db, cls_fqcn.as_ref()) {
                    all_issues.push(mir_issues::Issue::new(
                        mir_issues::IssueKind::UndefinedDocblockClass {
                            name: cls_fqcn.to_string(),
                        },
                        header_location.clone(),
                    ));
                } else {
                    self.db.record_reference_location(crate::db::RefLoc {
                        symbol_key: Arc::from(format!("cls:{cls_fqcn}")),
                        file: file.clone(),
                        line: header_location.line,
                        col_start: header_location.col_start,
                        col_end: header_location.col_end,
                    });
                }
            }
        }
    }

    /// `UndefinedDocblockClass`/`cls:` usage for class-level magic docblock
    /// tags — `@mixin`, `@property`/`@property-read`/`@property-write`, and
    /// `@method` — each of which names a class/interface/trait that must
    /// exist and is a real (virtual) reference to it. None of these tags
    /// correspond to a native AST member (`@mixin` names a class, and
    /// `@property`/`@method` are synthesized by the collector into
    /// `own_properties`/`own_methods` — see `add_docblock_members` — only
    /// when no real member of that name exists), so the per-member loop in
    /// `analyze_class_decl` over `decl.body.members` never sees them.
    /// Distinguishes synthesized members from real ones via
    /// `PropertyDef::from_docblock`/`MethodDef::is_virtual`, which a real
    /// declared member never sets.
    fn check_class_docblock_magic_members(
        &self,
        decl: &php_ast::owned::ClassDecl,
        fqcn: &str,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
    ) {
        if self.mode != AnalysisMode::Full {
            return;
        }
        let Some(doc_comment) = &decl.doc_comment else {
            return;
        };
        let (line, col_start) =
            crate::diagnostics::offset_to_line_col(source, doc_comment.span.start, source_map);
        let (line_end, col_end) =
            crate::diagnostics::offset_to_line_col(source, doc_comment.span.end, source_map);
        let location = mir_issues::Location {
            file: file.clone(),
            line,
            line_end,
            col_start,
            col_end: crate::diagnostics::clamp_col_end(line, line_end, col_start, col_end),
        };

        let check_class_name = |cls_fqcn: &str, all_issues: &mut Vec<Issue>| {
            self.check_and_record_docblock_class_at(cls_fqcn, &location, all_issues)
        };

        let type_class_names = |ty: &mir_types::Type| -> Vec<mir_types::Name> {
            ty.types
                .iter()
                .filter_map(|atomic| match atomic {
                    mir_types::Atomic::TNamedObject { fqcn, .. } => Some(*fqcn),
                    _ => None,
                })
                .collect()
        };

        let here = crate::db::Fqcn::from_str(self.db, fqcn);
        let Some(class) = crate::db::find_class_like(self.db, here) else {
            return;
        };

        for mixin_fqcn in class.mixins() {
            check_class_name(mixin_fqcn.as_ref(), all_issues);
        }

        for (_local, _original, from_fqcn) in class.pending_import_types() {
            check_class_name(from_fqcn.as_ref(), all_issues);
        }

        if let Some(props) = class.own_properties() {
            for prop in props.values().filter(|p| p.from_docblock) {
                let Some(ty) = prop.ty.as_deref() else {
                    continue;
                };
                for cls_fqcn in type_class_names(ty) {
                    check_class_name(cls_fqcn.as_ref(), all_issues);
                }
            }
        }

        for method in class.own_methods().values().filter(|m| m.is_virtual) {
            if let Some(ret) = method.return_type.as_deref() {
                for cls_fqcn in type_class_names(ret) {
                    check_class_name(cls_fqcn.as_ref(), all_issues);
                }
            }
            for param in method.params.iter() {
                let Some(ty) = param.ty.as_deref() else {
                    continue;
                };
                for cls_fqcn in type_class_names(ty) {
                    check_class_name(cls_fqcn.as_ref(), all_issues);
                }
            }
        }
    }

    /// Shared `UndefinedDocblockClass`/`cls:` usage for a single docblock-only
    /// class name: emit the issue if it doesn't resolve to a real class/
    /// interface/trait/enum, otherwise record a reference at `location`.
    fn check_and_record_docblock_class_at(
        &self,
        cls_fqcn: &str,
        location: &mir_issues::Location,
        all_issues: &mut Vec<Issue>,
    ) {
        if crate::diagnostics::is_pseudo_type(cls_fqcn) {
            return;
        }
        if !crate::db::class_exists(self.db, cls_fqcn) {
            all_issues.push(mir_issues::Issue::new(
                mir_issues::IssueKind::UndefinedDocblockClass {
                    name: cls_fqcn.to_string(),
                },
                location.clone(),
            ));
        } else {
            self.db.record_reference_location(crate::db::RefLoc {
                symbol_key: Arc::from(format!("cls:{cls_fqcn}")),
                file: location.file.clone(),
                line: location.line,
                col_start: location.col_start,
                col_end: location.col_end,
            });
        }
    }

    /// `UndefinedDocblockClass`/`cls:` usage for class names nested inside a
    /// generic type-argument list — `@extends Base<Arg>`, `@implements
    /// Iface<Arg>`, and a class's own `@template T of Bound` — none of which
    /// are walked by the plain extends/implements existence checks (those
    /// only look at the outer class/interface name itself, via
    /// `extends_type_args`/`implements_type_args`/`TemplateParam::bound`,
    /// which are already namespace/template-resolved at collection time —
    /// see `resolve_union`/`resolve_union_doc_with_templates` — so no
    /// qualification concern here, only the missing validation).
    fn check_class_generic_type_args(
        &self,
        decl: &php_ast::owned::ClassDecl,
        fqcn: &str,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
    ) {
        if self.mode != AnalysisMode::Full {
            return;
        }
        let Some(doc_comment) = &decl.doc_comment else {
            return;
        };
        let (line, col_start) =
            crate::diagnostics::offset_to_line_col(source, doc_comment.span.start, source_map);
        let (line_end, col_end) =
            crate::diagnostics::offset_to_line_col(source, doc_comment.span.end, source_map);
        let location = mir_issues::Location {
            file: file.clone(),
            line,
            line_end,
            col_start,
            col_end: crate::diagnostics::clamp_col_end(line, line_end, col_start, col_end),
        };

        let here = crate::db::Fqcn::from_str(self.db, fqcn);
        let Some(crate::db::ClassLike::Class(class)) = crate::db::find_class_like(self.db, here)
        else {
            return;
        };

        // `extends_type_args`/`implements_type_args` are the concrete type
        // arguments THIS class passes to its parent/interfaces — e.g. `class
        // TypedList<T> implements Collection<T>` forwards its own template
        // param `T` positionally. Collected via plain `resolve_union` (no
        // template awareness — see the field docs), a forwarded template
        // name stays a bare `TNamedObject` instead of becoming
        // `TTemplateParam`, so it must be filtered out here or every generic
        // class forwarding its own template param would misreport it as an
        // undefined class.
        let own_template_names: rustc_hash::FxHashSet<&str> = class
            .template_params
            .iter()
            .map(|tp| tp.name.as_ref())
            .collect();

        let mut names = Vec::new();
        for ty in class.extends_type_args.iter() {
            collect_named_object_fqcns(ty, &mut names);
        }
        for (_iface, args) in class.implements_type_args.iter() {
            for ty in args {
                collect_named_object_fqcns(ty, &mut names);
            }
        }
        for tp in class.template_params.iter() {
            if let Some(bound) = tp.bound.as_deref() {
                collect_named_object_fqcns(bound, &mut names);
            }
        }
        for cls_fqcn in names {
            if own_template_names.contains(cls_fqcn.as_ref()) {
                continue;
            }
            self.check_and_record_docblock_class_at(cls_fqcn.as_ref(), &location, all_issues);
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
        self.check_method_docblock_classes(method, fqcn, file, source, source_map, all_issues);

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
            sa.collect_symbols = self.collect_symbols;
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

        if method_name == "__construct" && self.mode == AnalysisMode::Full {
            for param in method.params.iter() {
                if param.visibility.is_some() && param.type_hint.is_none() {
                    let prop_name = param.name.as_deref().unwrap_or("").to_string();
                    let (line, col_start) = crate::diagnostics::offset_to_line_col(
                        source,
                        param.span.start,
                        source_map,
                    );
                    let (line_end, col_end) =
                        crate::diagnostics::offset_to_line_col(source, param.span.end, source_map);
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
                            col_end: crate::diagnostics::clamp_col_end(
                                line, line_end, col_start, col_end,
                            ),
                        },
                    ));
                }
            }
        }

        let (params, return_ty, template_params, declared_throws) =
            method_chain_signature(self.db, fqcn, method_name);

        self.check_and_record_throws_classes(
            &declared_throws,
            method_header_name_span(source, method),
            file,
            source,
            source_map,
            all_issues,
        );

        // A docblock @return that conflicts with the native hint must not
        // make the method's own valid `return` statements look invalid — the
        // native hint is runtime truth. This only affects body-statement
        // checking below; MismatchingDocblockReturnType (computed elsewhere)
        // still compares against the raw, unfiltered docblock value.
        let return_ty = super::return_ty_for_body_check(
            self.db,
            file.as_ref(),
            return_ty,
            method.return_type.as_ref(),
            Some(fqcn),
        );
        let declared_return = return_ty.clone();
        let is_ctor = cx.detect_ctor && method_name == "__construct";
        let templates: Option<&[mir_codebase::definitions::TemplateParam]> = if cx.with_templates {
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
        ctx.current_method_name = Some(Arc::from(method_name));

        // Set is_in_pure_fn if the method is annotated @pure,
        // is_in_immutable_method if it's annotated @psalm-mutation-free, and
        // is_in_external_mutation_free_method if annotated @psalm-external-mutation-free.
        if let Some((_, method_storage)) = crate::db::find_method_in_chain(
            self.db,
            crate::db::Fqcn::from_str(self.db, fqcn),
            &method_name.to_ascii_lowercase(),
        ) {
            ctx.is_in_pure_fn = method_storage.is_pure;
            if !is_ctor && method_storage.is_mutation_free {
                ctx.is_in_immutable_method = true;
            }
            if !is_ctor && method_storage.is_external_mutation_free {
                ctx.is_in_external_mutation_free_method = true;
            }
        }

        // Set is_in_immutable_method for non-constructor methods of @psalm-immutable classes.
        if !is_ctor {
            if let Some(crate::db::ClassLike::Class(cls)) =
                crate::db::find_class_like(self.db, crate::db::Fqcn::from_str(self.db, fqcn))
            {
                if cls.is_immutable {
                    ctx.is_in_immutable_method = true;
                }
            }
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
        sa.collect_symbols = self.collect_symbols;
        ctx.is_generator = body_has_yield(&body.stmts);
        sa.analyze_stmts(&body.stmts, &mut ctx);
        let inferred = merge_return_types(&sa.return_types);
        let inferred = if sa.yielded_types.is_empty() {
            inferred
        } else {
            build_generator_return_type(&sa.yielded_types, inferred)
        };
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
            decl,
            self.db,
            file,
            source,
            source_map,
            all_issues,
            self.mode == AnalysisMode::Full,
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
                    self.mode == AnalysisMode::Full,
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
                    self.mode == AnalysisMode::Full,
                );
            }
        }

        self.check_class_docblock_magic_members(decl, fqcn, file, source, source_map, all_issues);
        self.check_class_generic_type_args(decl, fqcn, file, source, source_map, all_issues);

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
                    self.mode == AnalysisMode::Full,
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
                    self.mode == AnalysisMode::Full,
                );
            }
        }

        self.check_class_docblock_magic_members(decl, fqcn, file, source, source_map, all_issues);
        self.check_class_generic_type_args(decl, fqcn, file, source, source_map, all_issues);

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

    /// Emit `InvalidTraitUse` issues if this class/enum violates any
    /// `@psalm-require-extends` / `@psalm-require-implements` constraint declared
    /// on the traits it uses, or (for an enum) consumes a trait that declares an
    /// instance property — enums may use traits but cannot carry extra state
    /// beyond their cases, so a trait instance property is a hard PHP fatal.
    pub(super) fn check_trait_constraints(
        &self,
        fqcn: &str,
        file: &Arc<str>,
        all_issues: &mut Vec<Issue>,
    ) {
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

            if self.mode == AnalysisMode::Full {
                let loc = make_loc();
                self.db.record_reference_location(crate::db::RefLoc {
                    symbol_key: Arc::from(format!("cls:{trait_fqcn}")),
                    file: loc.file.clone(),
                    line: loc.line,
                    col_start: loc.col_start,
                    col_end: loc.col_end,
                });
            }

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

            if class.is_enum() {
                if let Some(props) = trait_class.own_properties() {
                    for (prop_name, prop_def) in props.iter() {
                        if !prop_def.is_static {
                            all_issues.push(mir_issues::Issue::new(
                                mir_issues::IssueKind::InvalidTraitUse {
                                    trait_name: tr_short.to_string(),
                                    reason: format!(
                                        "Enum {fqcn} cannot use trait {tr_short}: it declares \
                                         a non-static property ${prop_name}, and enums cannot \
                                         carry state beyond their cases"
                                    ),
                                },
                                make_loc(),
                            ));
                        }
                    }
                }
            }

            // A `readonly class` requires EVERY property it carries — including
            // ones pulled in from a used trait — to be readonly. A trait property
            // declared without `readonly` (and not just an advisory `@readonly`
            // docblock tag) is a hard PHP fatal once consumed by a readonly class.
            if class.is_readonly() {
                if let Some(props) = trait_class.own_properties() {
                    for (prop_name, prop_def) in props.iter() {
                        if !prop_def.is_static
                            && !prop_def.from_docblock
                            && !prop_def.has_native_readonly
                        {
                            all_issues.push(mir_issues::Issue::new(
                                mir_issues::IssueKind::InvalidTraitUse {
                                    trait_name: tr_short.to_string(),
                                    reason: format!(
                                        "Readonly class {fqcn} cannot use trait {tr_short}: it \
                                         declares a non-readonly property ${prop_name}"
                                    ),
                                },
                                make_loc(),
                            ));
                        }
                    }
                }
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

        // `use T { T::missing as alias; }` (or an unqualified `as` naming no
        // method any used trait declares) — PHP fatals at class-declaration
        // time. Trait aliases carry no span of their own in storage, so the
        // diagnostic falls back to the class's own location, mirroring
        // `make_loc`'s fallback for a trait use with no recorded location.
        if let crate::db::ClassLike::Class(c) = &class {
            let fallback_loc = mir_issues::Location {
                file: file.clone(),
                line: 1,
                line_end: 1,
                col_start: 0,
                col_end: 0,
            };
            for (trait_name_opt, orig_lower, _vis_override, _alias_cased) in
                c.trait_aliases.values()
            {
                let candidates: &[Arc<str>] = match trait_name_opt {
                    Some(t) => std::slice::from_ref(t),
                    None => &trait_list,
                };
                let found = candidates.iter().any(|t| {
                    let here = crate::db::Fqcn::from_str(self.db, t.as_ref());
                    crate::db::find_class_like(self.db, here)
                        .is_some_and(|cl| cl.own_methods().contains_key(orig_lower.as_ref()))
                });
                if !found {
                    all_issues.push(mir_issues::Issue::new(
                        mir_issues::IssueKind::UndefinedTraitAliasMethod {
                            trait_name: trait_name_opt
                                .as_ref()
                                .map(|t| t.rsplit('\\').next().unwrap_or(t.as_ref()).to_string()),
                            method: orig_lower.to_string(),
                        },
                        fallback_loc.clone(),
                    ));
                }
            }
        }
    }
}
