use std::sync::Arc;

use mir_codebase::definitions::{
    wrap_return_type, wrap_template_bound, DeclaredParam, FunctionDef, TemplateParam,
};
use mir_types::Name;

use super::{expand_aliases_only, DefinitionCollector};
use crate::parser::type_from_hint_owned;

/// Returns `true` if `stmts` (does not recurse into nested function/closure
/// bodies) contain a call to `func_get_args()`, `func_get_arg()`, or
/// `func_num_args()`.
///
/// When a function uses these PHP intrinsics it can accept more positional
/// arguments than its declared parameter list suggests. We add a synthetic
/// trailing variadic parameter so the arity checker does not emit
/// `TooManyArguments` for such functions.
pub(crate) fn stmts_use_func_get_args(stmts: &[php_ast::owned::Stmt]) -> bool {
    use php_ast::owned::visitor::OwnedVisitor;
    use php_ast::owned::{Expr, ExprKind, Stmt, StmtKind};
    use std::ops::ControlFlow;

    struct Finder;

    impl OwnedVisitor for Finder {
        type Break = ();

        fn visit_stmt(&mut self, stmt: &Stmt) -> ControlFlow<()> {
            match &stmt.kind {
                StmtKind::Function(_)
                | StmtKind::Class(_)
                | StmtKind::Interface(_)
                | StmtKind::Trait(_)
                | StmtKind::Enum(_) => ControlFlow::Continue(()),
                _ => crate::recursion::walk_stmt(self, stmt),
            }
        }

        fn visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            match &expr.kind {
                ExprKind::FunctionCall(call)
                    if matches!(
                        crate::ast_ext::NameBareExt::name_bare(&call.name.kind),
                        Some("func_get_args" | "func_get_arg" | "func_num_args")
                    ) =>
                {
                    ControlFlow::Break(())
                }
                ExprKind::Closure(_) | ExprKind::ArrowFunction(_) | ExprKind::AnonymousClass(_) => {
                    ControlFlow::Continue(())
                }
                _ => crate::recursion::walk_expr(self, expr),
            }
        }
    }

    let mut finder = Finder;
    stmts.iter().any(|s| finder.visit_stmt(s).is_break())
}

impl DefinitionCollector<'_> {
    pub(super) fn collect_function(
        &mut self,
        decl: &php_ast::owned::FunctionDecl,
        stmt_span: php_ast::Span,
    ) {
        let short_name = decl.name.as_str().unwrap_or_default().to_string();
        let fqn = if let Some(ns) = &self.namespace {
            format!("{ns}\\{short_name}")
        } else {
            short_name.clone()
        };

        let doc = self.parse_callable_docblock(decl.doc_comment.as_ref());
        let doc_span = decl
            .doc_comment
            .as_ref()
            .map(|c| c.span.start)
            .unwrap_or(stmt_span.start);
        self.emit_docblock_issues(&doc, doc_span);

        if !self.version_allows(&doc) || !self.version_attr_available(&decl.attributes) {
            return;
        }

        let type_aliases = self.build_type_aliases_deferring_imports(&doc);

        // Build template names first so bound resolution below can recognise template-param
        // names and avoid FQN-qualifying them (e.g. `@template T of K` where K is another param).
        let template_names: rustc_hash::FxHashSet<String> = doc
            .templates
            .iter()
            .map(|(n, _, _, _)| n.to_string())
            .collect();

        // Extract template parameters; resolve bounds with template-awareness so template
        // names used as bounds are stored as TTemplateParam, not wrongly namespace-qualified.
        let template_params = doc
            .templates
            .iter()
            .map(|(name, bound, variance, default)| TemplateParam {
                name: name.as_str().into(),
                bound: wrap_template_bound(bound.clone().map(|b| {
                    let b = expand_aliases_only(b, &type_aliases);
                    Self::fill_self_static_parent(
                        self.resolve_union_doc_with_templates(
                            b,
                            &template_names,
                            fqn.as_str(),
                            &[],
                        ),
                        fqn.as_str(),
                    )
                })),
                default: wrap_template_bound(default.clone().map(|d| {
                    let d = expand_aliases_only(d, &type_aliases);
                    Self::fill_self_static_parent(
                        self.resolve_union_doc_with_templates(
                            d,
                            &template_names,
                            fqn.as_str(),
                            &[],
                        ),
                        fqn.as_str(),
                    )
                })),
                defining_entity: fqn.as_str().into(),
                variance: *variance,
            })
            .collect::<Vec<_>>();

        let mut params = Vec::new();
        let mut local_scalar = 0usize;
        let mut local_complex = 0usize;
        let mut local_defaults = 0usize;
        for p in decl.params.iter() {
            // phpstorm-stubs `#[PhpStormStubsElementAvailable]`: a param that
            // does not exist at the target version is omitted entirely (keeping
            // it would corrupt arity checks).
            if !self.version_attr_available(&p.attributes) {
                continue;
            }
            let param_name = p.name.as_str().unwrap_or_default();
            let has_native_type =
                p.type_hint.is_some() || self.version_attr_type_string(&p.attributes).is_some();
            let native_ty =
                self.resolve_union_opt(p.type_hint.as_ref().map(|h| type_from_hint_owned(h, None)));
            let doc_type_raw = doc
                .get_param_type_string(param_name)
                .map(|s| Arc::<str>::from(s.trim()));
            let doc_ty = doc.get_param_type(param_name).cloned().map(|u| {
                let expanded = expand_aliases_only(u, &type_aliases);
                let doc_ty = self.resolve_union_doc_with_templates(
                    expanded,
                    &template_names,
                    &fqn,
                    &template_params,
                );
                // When the native hint is a concrete scalar and the docblock has only
                // atoms from a different scalar family (e.g. `@param int` + `bool` hint),
                // the PHP type hint is the runtime truth — prefer it over the docblock.
                if native_ty
                    .as_ref()
                    .is_some_and(|n| super::native_hint_wins_over_docblock_scalar(n, &doc_ty))
                {
                    return native_ty.clone().unwrap();
                }
                // Partial conflict (e.g. `@param int|string` on a native `int`
                // hint): strip the atoms foreign to the hint's family instead
                // of storing the raw union, which would let body analysis
                // believe $x could hold a value the native hint rules out.
                let mut doc_ty = match native_ty.as_ref() {
                    Some(n) => super::preserve_native_nullability(
                        n,
                        super::resolve_docblock_scalar_conflict(n, doc_ty),
                    ),
                    None => doc_ty,
                };
                // Mark the type as docblock-sourced so signature checks (e.g.
                // param contravariance) can tell a `@param` refinement apart
                // from a native type hint.
                doc_ty = super::per_argument_variadic_doc_type(
                    p.variadic,
                    native_ty.as_ref(),
                    doc_type_raw.as_deref(),
                    doc_ty,
                );
                doc_ty.from_docblock = true;
                doc_ty
            });
            // phpstorm-stubs `#[LanguageLevelTypeAware]` type override wins unless the docblock refines it.
            let ty = match self
                .version_attr_type_string(&p.attributes)
                .map(|s| crate::parser::docblock::parse_type_string(&s))
            {
                Some(attr_ty) => Some(super::refine_version_attr_type(
                    attr_ty,
                    doc_ty,
                    &template_params,
                )),
                None => doc_ty.or(native_ty),
            };
            if let Some(ty_ref) = &ty {
                if super::is_simple_scalar(ty_ref) {
                    local_scalar += 1;
                } else {
                    local_complex += 1;
                }
            }
            let has_default = p.default.is_some();
            if has_default {
                local_defaults += 1;
            }

            let out_ty = doc.get_out_param_type(param_name).cloned().map(|u| {
                let expanded = expand_aliases_only(u, &type_aliases);
                let mut doc_ty = self.resolve_union_doc_with_templates(
                    expanded,
                    &template_names,
                    &fqn,
                    &template_params,
                );
                doc_ty.from_docblock = true;
                doc_ty
            });
            let closure_this = doc.get_closure_this_type(param_name).cloned().map(|u| {
                let mut resolved = self.resolve_union_doc_with_templates(
                    u,
                    &template_names,
                    &fqn,
                    &template_params,
                );
                resolved.from_docblock = true;
                resolved
            });
            let ty_is_docblock = ty.as_ref().is_some_and(|t| t.from_docblock);
            let (ty, out_ty) = super::stub_out_param_types(
                &self.file,
                p.by_ref,
                has_default,
                has_native_type,
                ty,
                out_ty,
            );
            params.push(DeclaredParam {
                name: Name::new(param_name),
                ty: mir_codebase::wrap_param_type(ty),
                out_ty: mir_codebase::wrap_param_type(out_ty),
                closure_this: mir_codebase::wrap_param_type(closure_this),
                doc_type_raw: doc_type_raw.filter(|_| ty_is_docblock),
                doc_type_file: ty_is_docblock.then(|| self.file.clone()),
                has_default,
                default_text: p
                    .default
                    .as_ref()
                    .and_then(|d| crate::parser::span_text(self.source, d.span))
                    .map(Arc::from),
                is_variadic: p.variadic,
                is_byref: p.by_ref,
                is_optional: has_default || p.variadic,
            });
        }
        if local_scalar > 0 {
            super::SCALAR_PARAM_COUNT.fetch_add(local_scalar, std::sync::atomic::Ordering::Relaxed);
        }
        if local_complex > 0 {
            super::COMPLEX_PARAM_COUNT
                .fetch_add(local_complex, std::sync::atomic::Ordering::Relaxed);
        }
        if local_defaults > 0 {
            super::PARAM_WITH_DEFAULT
                .fetch_add(local_defaults, std::sync::atomic::Ordering::Relaxed);
        }

        // If the function body calls func_get_args() / func_get_arg() /
        // func_num_args(), it can accept more positional args than declared.
        // Add a synthetic untyped variadic param so TooManyArguments is not
        // emitted for such functions.
        let last_is_variadic = params.last().is_some_and(|p| p.is_variadic);
        if !last_is_variadic && stmts_use_func_get_args(&decl.body.stmts) {
            params.push(DeclaredParam {
                closure_this: None,
                name: Name::new("..."),
                ty: None,
                out_ty: None,
                doc_type_raw: None,
                doc_type_file: None,
                has_default: false,
                default_text: None,
                is_variadic: true,
                is_byref: false,
                is_optional: true,
            });
        }

        // phpstorm-stubs `#[LanguageLevelTypeAware]` return type wins over the
        // declared/docblock return type, routed through the same resolution.
        let return_type = if let Some(s) = self.version_attr_type_string(&decl.attributes) {
            let mut ty = crate::parser::docblock::parse_type_string(&s);
            ty.from_docblock = true;
            let expanded = expand_aliases_only(ty, &type_aliases);
            Some(self.resolve_union_doc_with_templates(
                expanded,
                &template_names,
                &fqn,
                &template_params,
            ))
        } else {
            match (doc.return_type.clone(), decl.return_type.as_ref()) {
                (Some(mut ty), _) => {
                    ty.from_docblock = true;
                    let expanded = expand_aliases_only(ty, &type_aliases);
                    Some(self.resolve_union_doc_with_templates(
                        expanded,
                        &template_names,
                        &fqn,
                        &template_params,
                    ))
                }
                (None, Some(h)) => self.resolve_union_opt(Some(type_from_hint_owned(h, None))),
                (None, None) => None,
            }
        };

        let throws = self.resolve_throws(&doc.throws, Some(&type_aliases));

        let docstring = if doc.description.trim().is_empty() {
            None
        } else {
            Some(Arc::from(doc.description.as_str()))
        };

        let assertions = self.build_assertions(
            &doc,
            Some(&type_aliases),
            &template_names,
            &template_params,
            fqn.as_str(),
        );

        let storage = FunctionDef {
            fqn: fqn.clone().into(),
            short_name: short_name.into(),
            params: Arc::from(params.into_boxed_slice()),
            return_type: wrap_return_type(return_type),
            inferred_return_type: None,
            template_params,
            assertions,
            throws,
            deprecated: doc.deprecated.as_deref().map(Arc::from).or_else(|| {
                // Only detect #[Deprecated] without arguments (no-arg form used in
                // user code). Stubs use #[Deprecated(since: '...', ...)] with args
                // which would otherwise flood callers with spurious DeprecatedCall.
                if decl.attributes.iter().any(|a| {
                    a.args.is_empty()
                        && a.name
                            .parts
                            .last()
                            .map(|p| p.as_ref().eq_ignore_ascii_case("Deprecated"))
                            .unwrap_or(false)
                }) {
                    Some(Arc::from(""))
                } else {
                    None
                }
            }),
            is_pure: doc.is_pure || self.pure_attr(&decl.attributes),
            is_mutation_free: doc.is_mutation_free,
            is_external_mutation_free: doc.is_external_mutation_free,
            no_named_arguments: doc.no_named_arguments,
            location: Some(self.location(stmt_span.start, stmt_span.end)),
            docstring,
            tag_descriptions: self.tag_descriptions(&doc),
            taint_sink_params: doc
                .taint_sinks
                .iter()
                .map(|(param, kind)| (Arc::from(param.as_str()), Arc::from(kind.as_str())))
                .collect(),
            is_taint_source: doc.is_taint_source,
            taint: doc.taint_annotations(),
            type_aliases: Self::stored_type_aliases(&type_aliases),
        };

        self.slice.functions.push(std::sync::Arc::new(storage));

        // Scan the function body for `@var`-annotated global declarations.
        self.scan_stmts_for_global_vars(&decl.body.stmts);
        self.scan_stmts_for_defines(&decl.body.stmts);
    }

    pub(super) fn collect_global_stmt(&mut self, stmt: &php_ast::owned::Stmt) {
        // Top-level `global $x` — unusual in PHP but valid.
        self.try_collect_global_var_annotation(stmt);
    }
}
