use super::helpers::{
    apply_doc_param_types, ast_params_to_fn_params_resolved, resolve_named_objects_in_union_native,
};
use super::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use crate::stmt::{mir_check_matches, return_type_is_invalid, widen_for_check};
use crate::symbol::ReferenceKind;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Name, Type};
use php_ast::owned::{ArrowFunctionExpr, ClosureExpr, ExprKind, Param};
use php_ast::Span;
use std::sync::Arc;

/// A by-ref capture (`use (&$var)`) is the same variable the closure body
/// runs against on every invocation, including any invocation prior to this
/// one that mutated it — so seeding it with the exact literal/narrowed type
/// snapshotted at the closure-literal site falsely claims that value can
/// never differ. Widens scalar literals to their base type and collapses
/// `true`/`false` to `bool`, mirroring the `mixed` fallback already used for
/// the undefined-capture case, but only as much as is safe for an
/// already-typed variable. A bare `null` has no base type to widen to, so it
/// becomes `mixed` (`$seen = null;` filled in by a sibling closure).
fn widen_byref_capture(ty: Type) -> Type {
    if matches!(ty.types.as_slice(), [Atomic::TNull]) {
        return Type::mixed();
    }
    let widened = widen_for_check(ty);
    let mut out = Type::empty();
    for atomic in widened.types {
        let atomic = match atomic {
            Atomic::TTrue | Atomic::TFalse => Atomic::TBool,
            other => other,
        };
        out.add_type(atomic);
    }
    out
}

/// Parent type after a closure writes a by-ref capture. An empty array is
/// dropped once the closure has written an array. If several array types
/// remain, the closure may or may not have run, so shapes are opened and keys
/// either shape adds stay readable.
fn merge_byref_write(pre: Type, written: &Type) -> Type {
    let writes_array = written
        .types
        .iter()
        .any(|a| matches!(a, Atomic::TArray { .. } | Atomic::TKeyedArray { .. }));
    let mut base = Type::empty();
    for atomic in pre.types {
        let is_empty_array =
            matches!(&atomic, Atomic::TKeyedArray { properties, .. } if properties.is_empty());
        if !(writes_array && is_empty_array) {
            base.add_type(atomic);
        }
    }
    let merged = Type::merge(&base, written);
    let arrays = merged
        .types
        .iter()
        .filter(|a| {
            matches!(
                a,
                Atomic::TArray { .. }
                    | Atomic::TList { .. }
                    | Atomic::TNonEmptyArray { .. }
                    | Atomic::TNonEmptyList { .. }
                    | Atomic::TKeyedArray { .. }
            )
        })
        .count();
    if arrays < 2 {
        return merged;
    }
    let mut out = Type::empty();
    for atomic in merged.types {
        out.add_type(match atomic {
            Atomic::TKeyedArray {
                properties,
                is_list,
                ..
            } => Atomic::TKeyedArray {
                properties,
                is_open: true,
                is_list,
            },
            other => other,
        });
    }
    out
}

fn param_name_span(source: &str, p: &Param) -> Span {
    let Some(raw) = p.name.as_deref() else {
        return p.span;
    };
    let bare = raw.trim_start_matches('$');
    let range_start = p.span.start as usize;
    let range_end = (p.span.end as usize).min(source.len());
    let slice = &source[range_start..range_end];
    let needle = format!("${bare}");
    if let Some(rel) = slice.find(needle.as_str()) {
        let start = p.span.start + rel as u32;
        Span {
            start,
            end: start + needle.len() as u32,
        }
    } else {
        p.span
    }
}

/// Carry a `$this->prop` narrowing proven before a closure/arrow function
/// literal into the closure's own scope, but only for `readonly` properties.
/// An ordinary mutable property could still change between the guard and
/// whenever the closure actually runs, so resetting it is correct; a
/// `readonly` property can never change after construction, so the guard's
/// proof stays valid no matter when the closure is invoked.
fn propagate_readonly_prop_refinements(
    db: &dyn crate::db::MirDatabase,
    ctx: &FlowState,
    inner_ctx: &mut FlowState,
) {
    let Some(self_fqcn) = ctx.self_fqcn.clone() else {
        return;
    };
    let this_sym = mir_types::Name::from("this");
    let here = crate::db::Fqcn::from_str(db, self_fqcn.as_ref());
    for ((obj_var, prop), ty) in ctx.prop_refined.iter() {
        if *obj_var != this_sym {
            continue;
        }
        if let Some((_, p_def)) = crate::db::find_property_in_chain(db, here, prop.as_str()) {
            if p_def.is_readonly {
                inner_ctx.set_prop_refined("this", prop.as_str(), (**ty).clone());
            }
        }
    }
}

/// Pick the return type recorded on a closure/arrow function. Normally the
/// declared return (`: T` native hint or `@return` docblock) wins, because
/// a declared contract is what call sites may rely on. But when the body —
/// fully visible at this expression — provably returns a narrower *string*
/// than the declared string family, record the body's refined type instead:
/// every value the closure can actually return is a body value, so the
/// refinement is sound for call sites (e.g.
/// `fn (string $n): string => 'prefix' . $n` provably returns
/// `non-empty-string`, letting `array_map` yield `list<non-empty-string>`
/// rather than `list<string>` — M19, B8 lineage).
///
/// The refinement is limited to the string family (B8/M19, the
/// `non-empty-string` lineage) and to a bare native `array`. A declared
/// signature is never demoted to literal precision (`"hello"`), and
/// `mixed`/`never`/object/list-literal body types never replace the declared
/// contract. A keyed record body (`['id' => $i]`) refines a bare `array` with
/// its literal values widened. When the body is not a subtype
/// of the declared type (e.g. it returns `null` where the declaration does
/// not allow it, or a different family), keep the declared one: the
/// `InvalidReturnType`/`MixedReturnStatement` emitted for the mismatch
/// already covers it, and call sites must not see a value domain the
/// declared contract does not promise. Always-diverging bodies (`never`)
/// are likewise left alone: recording `never` changes call-site typing in
/// a way a refinement of the declared contract does not.
fn refined_closure_return(
    db: &dyn crate::db::MirDatabase,
    return_ty_hint: Option<Type>,
    inferred_return: Type,
) -> Type {
    let inferred_return = match &return_ty_hint {
        Some(declared)
            if is_bare_array(declared) && inferred_return.types.iter().any(is_record_shape) =>
        {
            widen_record_literals(&inferred_return)
        }
        _ => inferred_return,
    };
    let inferred_return = if return_ty_hint.is_some() {
        widen_scalar_literals(inferred_return)
    } else {
        inferred_return
    };
    let refines = return_ty_hint
        .as_ref()
        .filter(|declared| {
            let same_family = (is_string_family(declared) && is_string_family(&inferred_return))
                || (is_int_family(declared) && is_int_family(&inferred_return))
                || (is_bare_array(declared)
                    && inferred_return
                        .types
                        .iter()
                        .all(|a| is_generic_array(a) || is_record_shape(a)));
            same_family && crate::subtype::is_subtype(db, &inferred_return, declared)
        })
        .is_some();
    if refines {
        inferred_return
    } else {
        return_ty_hint.unwrap_or(inferred_return)
    }
}

/// Non-list keyed shape: a string-keyed record rather than a positional literal.
fn is_record_shape(a: &Atomic) -> bool {
    matches!(a, Atomic::TKeyedArray { is_list: false, .. })
}

/// Widen scalar literals inside record shapes, recursing into nested ones.
fn widen_record_literals(ty: &Type) -> Type {
    let mut out = Type::empty();
    for a in &ty.types {
        match a {
            Atomic::TKeyedArray {
                properties,
                is_open,
                is_list: false,
            } => {
                let properties = properties
                    .iter()
                    .map(|(k, p)| {
                        let mut p = p.clone();
                        p.ty = widen_record_literals(&crate::stmt::widen_for_check(p.ty));
                        (k.clone(), p)
                    })
                    .collect();
                out.add_type(Atomic::TKeyedArray {
                    properties: Box::new(properties),
                    is_open: *is_open,
                    is_list: false,
                });
            }
            other => out.add_type(other.clone()),
        }
    }
    out
}

/// String family: every atom is a string variant (a `null` in the union is
/// tolerated; the final subtype check decides whether it is allowed by the
/// declared type). `mixed`, `never`, scalars, objects, and arrays fail, so
/// this predicate alone keeps the refinement inside the B8/M19 lineage.
fn is_string_family(ty: &Type) -> bool {
    !ty.types.is_empty()
        && ty
            .types
            .iter()
            .all(|a| a.is_string() || matches!(a, Atomic::TNull))
}

/// Literal ints/strings widen to one sign / non-emptiness class, so a literal never leaks into the signature.
fn widen_scalar_literals(ty: Type) -> Type {
    let int_bounds = ty
        .types
        .iter()
        .filter_map(|a| match a {
            Atomic::TLiteralInt(n) => Some(*n),
            _ => None,
        })
        .fold(None, |acc: Option<(i64, i64)>, n| {
            Some(acc.map_or((n, n), |(lo, hi)| (lo.min(n), hi.max(n))))
        });
    let has_empty_string = ty
        .types
        .iter()
        .any(|a| matches!(a, Atomic::TLiteralString(s) if s.is_empty()));
    let mut out = Type::empty();
    out.from_docblock = ty.from_docblock;
    for a in ty.types {
        out.add_type(match (a, int_bounds) {
            (Atomic::TLiteralInt(_), Some((lo, _))) if lo > 0 => Atomic::TPositiveInt,
            (Atomic::TLiteralInt(_), Some((lo, _))) if lo >= 0 => Atomic::TNonNegativeInt,
            (Atomic::TLiteralInt(_), Some((_, hi))) if hi < 0 => Atomic::TNegativeInt,
            (Atomic::TLiteralInt(_), _) => Atomic::TInt,
            (Atomic::TLiteralString(_), _) if has_empty_string => Atomic::TString,
            (Atomic::TLiteralString(_), _) => Atomic::TNonEmptyString,
            (other, _) => other,
        });
    }
    out
}

/// Int family: every atom is an int variant (`null` tolerated, as in the string family).
fn is_int_family(ty: &Type) -> bool {
    !ty.types.is_empty()
        && ty
            .types
            .iter()
            .all(|a| a.is_int() || matches!(a, Atomic::TNull))
}

/// Native `array`: key and value are unconstrained.
fn is_bare_array(ty: &Type) -> bool {
    matches!(
        ty.types.as_slice(),
        [Atomic::TArray { key, value }]
            if (key.is_mixed() || key.is_array_key()) && value.is_mixed()
    )
}

/// Array atoms other than keyed shapes, whose literal entries must not leak into the signature.
fn is_generic_array(a: &Atomic) -> bool {
    a.is_array() && !matches!(a, Atomic::TKeyedArray { .. })
}

impl<'a> ExpressionAnalyzer<'a> {
    /// Local type aliases (`@psalm-type`/`@phpstan-type`) declared in the
    /// enclosing class-like's (or, for a closure declared inside a free
    /// function, that function's own) docblock never expanded inside a
    /// nested closure/arrow-function's OWN `@param`/`@return` docblock —
    /// every usage site showed the literal unresolved alias name instead of
    /// its expansion. Mirrors `stmt/mod.rs`'s `extract_var_annotation_from`,
    /// which already does this for a bare `@var` annotation; not cached the
    /// way that helper is, since a closure body is analyzed once per
    /// enclosing function, not once per statement.
    fn expand_local_type_aliases_in_doc(
        &self,
        doc: &mut crate::parser::ParsedDocblock,
        ctx: &FlowState,
    ) {
        let aliases = if let Some(fqcn) = ctx.self_fqcn.as_deref() {
            crate::db::find_class_like(self.db, crate::db::Fqcn::from_str(self.db, fqcn))
                .map(|cl| cl.type_aliases().clone())
        } else if let Some(fqn) = ctx.current_function_fqn.as_deref() {
            crate::db::find_function(self.db, crate::db::Fqcn::from_str(self.db, fqn))
                .map(|f| f.type_aliases.clone())
        } else {
            None
        };
        let Some(aliases) = aliases else { return };
        if aliases.is_empty() {
            return;
        }
        for (_, ty) in doc.params.iter_mut() {
            *ty = crate::collector::expand_aliases_only(ty.clone(), &aliases);
        }
        if let Some(rt) = doc.return_type.take() {
            doc.return_type = Some(crate::collector::expand_aliases_only(rt, &aliases));
        }
    }

    /// Template params visible to a closure/arrow-function literal's OWN
    /// leading `@param`/`@return` docblock: the enclosing class's own (for a
    /// closure declared inside a generic class's method) plus the enclosing
    /// method's own (a per-method `@template`), or a free function's own when
    /// there's no enclosing class. Mirrors `stmt/mod.rs`'s
    /// `extract_var_annotation_from`, which gathers the same context for a
    /// bare `@var` annotation — a name matching one of these must resolve to
    /// a `TTemplateParam` via `resolve_union_for_file_with_templates`, not an
    /// ordinary (and namespace-mis-qualified) class reference, the same way
    /// the closure body's own template-typed values already do.
    fn own_doc_template_context(
        &self,
        ctx: &FlowState,
    ) -> (
        rustc_hash::FxHashSet<String>,
        Vec<mir_codebase::definitions::TemplateParam>,
        String,
    ) {
        let mut template_names: rustc_hash::FxHashSet<String> = rustc_hash::FxHashSet::default();
        let mut template_params: Vec<mir_codebase::definitions::TemplateParam> = Vec::new();
        let defining_entity = ctx
            .self_fqcn
            .as_deref()
            .or(ctx.current_function_fqn.as_deref())
            .unwrap_or_default()
            .to_string();
        if let Some(fqcn) = ctx.self_fqcn.as_deref() {
            if let Some(class_like) =
                crate::db::find_class_like(self.db, crate::db::Fqcn::from_str(self.db, fqcn))
            {
                template_names.extend(
                    class_like
                        .template_params()
                        .iter()
                        .map(|tp| tp.name.to_string()),
                );
                template_params.extend(class_like.template_params().iter().cloned());
                if let Some(method_name) = ctx.current_method_name.as_deref() {
                    let method_name = crate::util::php_ident_lowercase(method_name);
                    if let Some(method) = class_like.own_methods().get(method_name.as_str()) {
                        template_names
                            .extend(method.template_params.iter().map(|tp| tp.name.to_string()));
                        template_params.extend(method.template_params.iter().cloned());
                    }
                }
            }
        } else if let Some(fqn) = ctx.current_function_fqn.as_deref() {
            if let Some(function) =
                crate::db::find_function(self.db, crate::db::Fqcn::from_str(self.db, fqn))
            {
                template_names.extend(
                    function
                        .template_params
                        .iter()
                        .map(|tp| tp.name.to_string()),
                );
                template_params.extend(function.template_params.iter().cloned());
            }
        }
        (template_names, template_params, defining_entity)
    }

    /// Narrows natively-typed params to the types the callee actually passes in
    /// (`callable(Err::NotFound)` for a closure declared `Err $e`). Docblock
    /// `@param` types win.
    fn narrow_params_to_callback_hints(&mut self, params: &mut [mir_codebase::DeclaredParam]) {
        let Some(hints) = self.callback_param_hints.take() else {
            return;
        };
        for (param, hint) in params.iter_mut().zip(hints) {
            let (Some(declared), Some(hint)) = (&param.ty, hint) else {
                continue;
            };
            if param.is_byref
                || param.is_variadic
                || declared.from_docblock
                || hint.is_never()
                || hint == **declared
                || !crate::subtype::is_subtype(self.db, &hint, declared)
            {
                continue;
            }
            param.ty = mir_codebase::wrap_param_type(Some(hint));
        }
    }

    pub(super) fn analyze_closure(
        &mut self,
        c: &ClosureExpr,
        expr_span: php_ast::Span,
        ctx: &mut FlowState,
    ) -> Type {
        for param in c.params.iter() {
            if let Some(hint) = &param.type_hint {
                self.check_type_hint(hint);
            }
        }
        if let Some(hint) = &c.return_type {
            self.check_type_hint(hint);
        }

        let mut leading_doc = crate::parser::find_preceding_docblock(self.source, expr_span.start)
            .map(|doc| crate::parser::DocblockParser::parse(&doc));
        if let Some(doc) = &mut leading_doc {
            self.expand_local_type_aliases_in_doc(doc, ctx);
        }

        let mut params = ast_params_to_fn_params_resolved(
            &c.params,
            ctx.self_fqcn.as_deref(),
            self.db,
            &self.file,
        );
        let (template_names, template_params, defining_entity) = self.own_doc_template_context(ctx);
        if let Some(doc) = &leading_doc {
            apply_doc_param_types(
                &mut params,
                &c.params,
                &doc.params,
                self.db,
                &self.file,
                &template_names,
                &template_params,
                &defining_entity,
            );
        }
        self.narrow_params_to_callback_hints(&mut params);
        let return_ty_hint = c
            .return_type
            .as_ref()
            .map(|h| crate::parser::type_from_hint_owned(h, ctx.self_fqcn.as_deref()))
            .map(|u| resolve_named_objects_in_union_native(u, self.db, &self.file))
            .or_else(|| {
                // Fall back to `@return` docblock preceding the `function` keyword.
                leading_doc
                    .as_ref()
                    .and_then(|doc| doc.return_type.clone())
                    .map(|ty| {
                        crate::stmt::resolve_union_for_file_with_templates(
                            ty,
                            self.db,
                            &self.file,
                            &template_names,
                            &template_params,
                            &defining_entity,
                        )
                    })
            });

        if return_ty_hint.is_none() && self.mode == crate::expr::AnalysisMode::Full {
            self.emit(
                mir_issues::IssueKind::MissingClosureReturnType,
                mir_issues::Severity::Info,
                expr_span,
            );
        }

        let mut closure_ctx = crate::flow_state::FlowState::for_function(
            &params,
            return_ty_hint.clone(),
            Arc::from([]),
            ctx.self_fqcn.clone(),
            ctx.parent_fqcn.clone(),
            ctx.static_fqcn.clone(),
            ctx.strict_types,
            c.is_static,
        );
        // A non-static closure declared outside any class body doesn't get `$this`
        // injected by `for_function` (no `self_fqcn`), but it's still valid PHP for
        // it to reference `$this` if the closure is later rebound to an object via
        // `Closure::bind()`/`bindTo()`/`call()` — a common macro/PHPUnit idiom.
        // Model that by seeding `$this` as an object of unknown type rather than
        // leaving it undefined, which would otherwise misfire `InvalidScope`.
        if ctx.self_fqcn.is_none() && !c.is_static {
            let this_sym = Name::from("this");
            Arc::make_mut(&mut closure_ctx.vars).insert(
                this_sym,
                mir_codebase::definitions::wrap_var_type(Type::single(Atomic::TObject)),
            );
            Arc::make_mut(&mut closure_ctx.assigned_vars).insert(this_sym);
        }
        // Closures see the enclosing function/method's template params (e.g. a
        // captured `@template T`-typed variable assigned to a typed property
        // inside the closure body) — without this, `type_refs_any_template`
        // checks against an empty set and treats the value as a concrete type,
        // producing spurious InvalidPropertyAssignment/instanceof narrowing bugs.
        closure_ctx.template_param_names = Arc::clone(&ctx.template_param_names);
        // A closure invoked from inside a @pure/@psalm-immutable/
        // @psalm-external-mutation-free body can still smuggle out an
        // observable side effect, so it must inherit that purity context
        // rather than starting fresh — an immediately-invoked closure that
        // mutates a captured object would otherwise go completely unchecked.
        closure_ctx.is_in_pure_fn = ctx.is_in_pure_fn;
        closure_ctx.is_in_immutable_method = ctx.is_in_immutable_method;
        closure_ctx.is_in_external_mutation_free_method = ctx.is_in_external_mutation_free_method;
        propagate_readonly_prop_refinements(self.db, ctx, &mut closure_ctx);
        for p in c.params.iter() {
            if let Some(raw) = p.name.as_deref() {
                let trimmed = raw.trim_start_matches('$');
                let ty = closure_ctx.get_var(trimmed);
                self.record_symbol(
                    param_name_span(self.source, p),
                    ReferenceKind::Variable(Arc::from(trimmed)),
                    ty,
                );
            }
        }
        let mut byref_captures: Vec<(&str, Type)> = Vec::new();
        for use_var in c.use_vars.iter() {
            let name = use_var.name.trim_start_matches('$');
            // A by-ref capture (`use (&$f)`) binds by reference and auto-creates
            // the variable in the parent scope if it does not yet exist, so it is
            // never "undefined" — this is what makes a self-referential closure
            // `$f = function () use (&$f) {...}` valid. Define it in both scopes
            // and skip the undefined check.
            if use_var.by_ref {
                if !ctx.var_is_defined(name) {
                    // Only the self-referential idiom
                    // (`$f = function () use (&$f) {...}`, where this capture
                    // shares a name with the variable the closure literal is
                    // itself being assigned to) can safely be typed as a
                    // callable of unknown arity. Any other undefined by-ref
                    // capture is the equally common by-ref out-param idiom
                    // (`set_error_handler(function () use (&$error) {
                    // $error = ...; })`, no enclosing assignment at all) —
                    // seeding THAT as non-nullable callable falsely claims
                    // the value eventually assigned in the body can never be
                    // null/falsy, misfiring RedundantCondition/InvalidThrow
                    // at every later use. `mixed` makes no such claim.
                    let is_self_ref = ctx.self_ref_closure_hint == Some(Name::from(name));
                    ctx.set_var(
                        name,
                        if is_self_ref {
                            Type::single(mir_types::Atomic::TCallable {
                                params: None,
                                return_type: None,
                            })
                        } else {
                            Type::mixed()
                        },
                    );
                }
            } else if !ctx.var_is_defined(name) {
                if ctx.var_possibly_defined(name) {
                    self.emit(
                        mir_issues::IssueKind::PossiblyUndefinedVariable {
                            name: name.to_string(),
                        },
                        mir_issues::Severity::Warning,
                        use_var.span,
                    );
                } else {
                    self.emit(
                        mir_issues::IssueKind::UndefinedVariable {
                            name: name.to_string(),
                        },
                        mir_issues::Severity::Error,
                        use_var.span,
                    );
                }
            }
            let captured_ty = if use_var.by_ref {
                widen_byref_capture(ctx.get_var(name))
            } else {
                ctx.get_var(name)
            };
            self.record_symbol(
                crate::symbol::variable_token_span(self.source, use_var.span, name),
                ReferenceKind::Variable(Arc::from(name)),
                captured_ty.clone(),
            );
            closure_ctx.set_var(name, captured_ty.clone());
            if use_var.by_ref {
                byref_captures.push((name, captured_ty));
            }
            if ctx.is_tainted(name) {
                closure_ctx.taint_var(name);
            }
            // Mark the captured variable as read in the parent context, and
            // consume its pending write so it isn't reported as a dead write.
            ctx.read_vars.insert(mir_types::Name::from(name));
            ctx.mark_consumed(name);
        }

        // A capture (by value OR by reference) of a variable that is itself a
        // parameter of the enclosing function is still externally owned by
        // the caller, so calling a mutating method on it inside the closure
        // body is an externally observable side effect — exactly like
        // calling one on a real parameter. Extend `param_names` so the
        // existing pure/immutable/external-mutation-free method-call checks
        // (which key off that set) also catch such captures. A capture of a
        // locally-created object stays out of this set, matching the "local
        // objects are exempt" rule the same checks already apply to real
        // params. A by-ref capture (`use (&$c)`) counts too: it is the SAME
        // external variable, not a copy.
        if closure_ctx.is_in_pure_fn
            || closure_ctx.is_in_immutable_method
            || closure_ctx.is_in_external_mutation_free_method
        {
            let mut extended_param_names = (*closure_ctx.param_names).clone();
            // A by-ref capture of the enclosing function's OWN by-ref
            // parameter (`use (&$x)` where `&$x` is itself a by-ref
            // parameter) is the SAME reference, not a copy — a write
            // through it inside the closure body is exactly as
            // externally observable as writing `$x` directly in the
            // enclosing scope, but `check_var_write_purity`/
            // `assign_to_target`'s `Variable` arm (both keyed on
            // `byref_param_names`) could never see it, since that set was
            // never propagated into a closure's own `FlowState` at all.
            let mut extended_byref_param_names = (*closure_ctx.byref_param_names).clone();
            for use_var in c.use_vars.iter() {
                let name = use_var.name.trim_start_matches('$');
                if ctx.param_names.contains(&Name::from(name)) {
                    extended_param_names.insert(Name::from(name));
                }
                if use_var.by_ref && ctx.byref_param_names.contains(&Name::from(name)) {
                    extended_byref_param_names.insert(Name::from(name));
                }
            }
            closure_ctx.param_names = Arc::new(extended_param_names);
            closure_ctx.byref_param_names = Arc::new(extended_byref_param_names);
        }

        let mut sa = crate::stmt::StatementsAnalyzer::new(
            self.db,
            self.file.clone(),
            self.source,
            self.source_map,
            self.issues,
            self.symbols.as_deref_mut(),
            self.navigation_facts,
            self.resolved_navigation_facts,
            self.php_version,
            self.mode,
        );

        sa.collect_symbols = self.collect_symbols;
        sa.capture_symbol_types = self.capture_symbol_types;
        sa.codebase_symbols_only = self.codebase_symbols_only;
        sa.record_reference_locations = self.record_reference_locations;
        sa.collect_navigation_facts = self.collect_navigation_facts;
        sa.collect_resolved_navigation_facts = self.collect_resolved_navigation_facts;
        sa.analyze_stmts(&c.body.stmts, &mut closure_ctx);
        let inferred_return =
            crate::body_analysis::merge_return_types(&sa.return_types, closure_ctx.diverges);
        // A closure containing `yield` always returns a Generator, regardless
        // of what (if anything) it `return`s — same inference as a top-level
        // function/method (see `build_generator_return_type`), which this
        // closure-local `sa` otherwise silently dropped by only reading
        // `return_types`.
        let inferred_return = if sa.yielded_types.is_empty() {
            inferred_return
        } else {
            crate::body_analysis::build_generator_return_type(&sa.yielded_types, inferred_return)
        };

        // The closure may run zero or more times, so a by-ref write widens the
        // parent's type rather than replacing it. A bare `null` becomes `mixed`,
        // matching the capture side (`widen_byref_capture`).
        for (name, captured_ty) in byref_captures {
            let written_ty = closure_ctx.get_var(name);
            let pre_ty = ctx.get_var(name);
            if matches!(pre_ty.types.as_slice(), [Atomic::TNull]) {
                ctx.set_var(name, Type::mixed());
            } else if written_ty != captured_ty {
                let merged = merge_byref_write(pre_ty, &written_ty);
                ctx.set_var(name, merged);
            }
        }

        // If the closure reads an outer-scope variable without capturing it via `use`,
        // mark that variable as read in the outer context to suppress false UnusedParam.
        for name in &closure_ctx.read_vars {
            if ctx.var_is_defined(name) || ctx.var_possibly_defined(name) {
                ctx.read_vars.insert(*name);
                ctx.mark_consumed(name.as_str());
            }
        }

        let return_ty = refined_closure_return(self.db, return_ty_hint, inferred_return);
        let closure_params: Box<[mir_types::atomic::FnParam]> = params
            .iter()
            .map(|p| mir_types::atomic::FnParam {
                name: Name::from(p.name.as_ref()),
                ty: p
                    .ty
                    .as_ref()
                    .map(|arc| mir_types::SimpleType::from_union((**arc).clone())),
                out_ty: None,
                default: if p.has_default {
                    Some(mir_types::SimpleType::from_union(Type::mixed()))
                } else {
                    None
                },
                is_variadic: p.is_variadic,
                is_byref: p.is_byref,
                is_optional: p.is_optional,
            })
            .collect();

        Type::single(Atomic::TClosure {
            data: Box::new(mir_types::atomic::ClosureData {
                params: closure_params,
                return_type: return_ty,
                this_type: ctx.self_fqcn.clone().map(|f| {
                    Type::single(Atomic::TNamedObject {
                        fqcn: Name::from(f.as_ref()),
                        type_params: mir_types::union::empty_type_params(),
                    })
                }),
            }),
        })
    }

    pub(super) fn analyze_arrow_function(
        &mut self,
        af: &ArrowFunctionExpr,
        expr_span: php_ast::Span,
        ctx: &mut FlowState,
    ) -> Type {
        for param in af.params.iter() {
            if let Some(hint) = &param.type_hint {
                self.check_type_hint(hint);
            }
        }
        if let Some(hint) = &af.return_type {
            self.check_type_hint(hint);
        }

        let mut leading_doc = crate::parser::find_preceding_docblock(self.source, expr_span.start)
            .map(|doc| crate::parser::DocblockParser::parse(&doc));
        if let Some(doc) = &mut leading_doc {
            self.expand_local_type_aliases_in_doc(doc, ctx);
        }

        let mut params = ast_params_to_fn_params_resolved(
            &af.params,
            ctx.self_fqcn.as_deref(),
            self.db,
            &self.file,
        );
        let (template_names, template_params, defining_entity) = self.own_doc_template_context(ctx);
        if let Some(doc) = &leading_doc {
            apply_doc_param_types(
                &mut params,
                &af.params,
                &doc.params,
                self.db,
                &self.file,
                &template_names,
                &template_params,
                &defining_entity,
            );
        }
        self.narrow_params_to_callback_hints(&mut params);
        let return_ty_hint = af
            .return_type
            .as_ref()
            .map(|h| crate::parser::type_from_hint_owned(h, ctx.self_fqcn.as_deref()))
            .map(|u| resolve_named_objects_in_union_native(u, self.db, &self.file))
            .or_else(|| {
                // Fall back to `@return` docblock preceding the `fn` keyword — mirrors
                // the same fallback in `analyze_closure` for `function(...) {...}`.
                leading_doc
                    .as_ref()
                    .and_then(|doc| doc.return_type.clone())
                    .map(|ty| {
                        crate::stmt::resolve_union_for_file_with_templates(
                            ty,
                            self.db,
                            &self.file,
                            &template_names,
                            &template_params,
                            &defining_entity,
                        )
                    })
            });

        let mut arrow_ctx = crate::flow_state::FlowState::for_function(
            &params,
            return_ty_hint.clone(),
            Arc::from([]),
            ctx.self_fqcn.clone(),
            ctx.parent_fqcn.clone(),
            ctx.static_fqcn.clone(),
            ctx.strict_types,
            af.is_static,
        );
        // See analyze_closure: propagate the enclosing scope's template params
        // so captured template-typed variables aren't misjudged as concrete.
        arrow_ctx.template_param_names = Arc::clone(&ctx.template_param_names);
        // See analyze_closure: an arrow function invoked from inside a
        // @pure/@psalm-immutable/@psalm-external-mutation-free body can still
        // smuggle out a side effect through an implicitly-captured variable —
        // `fn() => impure_fn()` or a tainted value flowing into a sink must be
        // checked the same way the equivalent `function(){...}` closure is.
        arrow_ctx.is_in_pure_fn = ctx.is_in_pure_fn;
        arrow_ctx.is_in_immutable_method = ctx.is_in_immutable_method;
        arrow_ctx.is_in_external_mutation_free_method = ctx.is_in_external_mutation_free_method;
        propagate_readonly_prop_refinements(self.db, ctx, &mut arrow_ctx);
        // Arrow functions auto-capture every outer variable by value (no
        // explicit `use()` list), so taint on any of them must carry over too.
        arrow_ctx.tainted_vars = ctx.tainted_vars.clone();
        let this_sym = mir_types::Name::from("this");
        for (name, ty) in ctx.vars.iter() {
            // Static arrow functions don't capture $this from the outer scope.
            if af.is_static && *name == this_sym {
                continue;
            }
            if !arrow_ctx.vars.contains_key(name) {
                std::sync::Arc::make_mut(&mut arrow_ctx.vars).insert(*name, ty.clone());
                std::sync::Arc::make_mut(&mut arrow_ctx.assigned_vars).insert(*name);
            }
        }
        // See analyze_closure: an arrow function outside any class also produces
        // a rebindable Closure, so `$this` may be validly late-bound even though
        // there's no enclosing `self_fqcn` to capture it from here.
        if !af.is_static && !arrow_ctx.vars.contains_key(&this_sym) {
            std::sync::Arc::make_mut(&mut arrow_ctx.vars).insert(
                this_sym,
                mir_codebase::definitions::wrap_var_type(Type::single(Atomic::TObject)),
            );
            std::sync::Arc::make_mut(&mut arrow_ctx.assigned_vars).insert(this_sym);
        }
        // See analyze_closure: a captured (by-value) outer parameter is still
        // externally owned by the caller, so mutating it via method call
        // inside the arrow body is an observable side effect just like a real
        // parameter — extend param_names so the existing pure/immutable/
        // external-mutation-free checks (which key off that set) catch it.
        // Every outer var is auto-captured, so union the whole set rather
        // than filtering by an explicit use() list.
        if arrow_ctx.is_in_pure_fn
            || arrow_ctx.is_in_immutable_method
            || arrow_ctx.is_in_external_mutation_free_method
        {
            let mut extended_param_names = (*arrow_ctx.param_names).clone();
            extended_param_names.extend(ctx.param_names.iter().copied());
            arrow_ctx.param_names = Arc::new(extended_param_names);
        }

        for p in af.params.iter() {
            if let Some(raw) = p.name.as_deref() {
                let trimmed = raw.trim_start_matches('$');
                // Use arrow_ctx.get_var to get the resolved type (params take priority
                // over outer-scope vars of the same name since they were inserted first).
                let ty = arrow_ctx.get_var(trimmed);
                self.record_symbol(
                    param_name_span(self.source, p),
                    ReferenceKind::Variable(Arc::from(trimmed)),
                    ty,
                );
            }
        }

        // Check @mir-check directives in the arrow function body.
        // If the body is parenthesized, look for docblocks before the inner expression.
        let check_target = match &af.body.kind {
            ExprKind::Parenthesized(inner) => inner.as_ref(),
            _ => &af.body,
        };
        if let Some(doc) =
            crate::parser::find_preceding_docblock(self.source, check_target.span.start)
        {
            let checks = crate::parser::DocblockParser::parse(&doc).mir_checks;
            for (expr_text, expected_str) in checks {
                let expected = crate::parser::docblock::parse_type_string(&expected_str);
                let actual_raw = self.eval_check_expr(&expr_text, &arrow_ctx);
                if !mir_check_matches(&expected, &actual_raw) {
                    self.emit(
                        IssueKind::TypeCheckMismatch {
                            var: expr_text,
                            expected: expected.to_string(),
                            actual: widen_for_check(actual_raw).to_string(),
                        },
                        Severity::Error,
                        check_target.span,
                    );
                }
            }
        }

        let inferred_return = self.analyze(&af.body, &mut arrow_ctx);
        // Arrow functions capture the whole outer scope by value: any variable
        // the body reads is a read (and consumed write) in the outer context.
        for name in &arrow_ctx.read_vars {
            ctx.read_vars.insert(*name);
            ctx.mark_consumed(name.as_str());
        }

        // The `=> expr` body is exactly one implicit `return expr;` — check it
        // against the declared return type the same way analyze_return_stmt does
        // for a regular closure/function body.
        if let Some(declared) = &return_ty_hint {
            let has_invalid = !declared.contains(|t| matches!(t, Atomic::TConditional { .. }))
                && ((declared.is_void()
                    && !inferred_return.is_void()
                    && !inferred_return.is_mixed())
                    || return_type_is_invalid(
                        &inferred_return,
                        declared,
                        ctx.strict_types,
                        self.db,
                        &self.file,
                    ));
            let is_mixed_return = !has_invalid
                && !declared.is_void()
                && !declared.is_mixed()
                && inferred_return.is_mixed()
                && !declared.contains(|t| matches!(t, Atomic::TConditional { .. }));
            if is_mixed_return {
                let kind = IssueKind::MixedReturnStatement {
                    declared: format!("{declared}"),
                };
                let severity = kind.default_severity();
                self.emit(kind, severity, check_target.span);
            } else if has_invalid {
                let kind = IssueKind::InvalidReturnType {
                    expected: format!("{declared}"),
                    actual: format!("{inferred_return}"),
                };
                let severity = kind.default_severity();
                self.emit(kind, severity, check_target.span);
            } else if !declared.is_void()
                && !declared.is_mixed()
                && !declared.contains(|t| matches!(t, Atomic::TNull))
                && !declared.contains(|t| matches!(t, Atomic::TConditional { .. }))
                && !declared.contains(|t| matches!(t, Atomic::TTemplateParam { .. }))
                && inferred_return.contains(|t| matches!(t, Atomic::TNull))
                && !inferred_return.remove_null().is_empty()
                && !return_type_is_invalid(
                    &inferred_return.remove_null(),
                    declared,
                    ctx.strict_types,
                    self.db,
                    &self.file,
                )
            {
                let kind = IssueKind::NullableReturnStatement {
                    expected: format!("{declared}"),
                    actual: format!("{inferred_return}"),
                };
                let severity = kind.default_severity();
                self.emit(kind, severity, check_target.span);
            }
        }

        let return_ty = refined_closure_return(self.db, return_ty_hint, inferred_return);
        let closure_params: Box<[mir_types::atomic::FnParam]> = params
            .iter()
            .map(|p| mir_types::atomic::FnParam {
                name: Name::from(p.name.as_ref()),
                ty: p
                    .ty
                    .as_ref()
                    .map(|arc| mir_types::SimpleType::from_union((**arc).clone())),
                out_ty: None,
                default: if p.has_default {
                    Some(mir_types::SimpleType::from_union(Type::mixed()))
                } else {
                    None
                },
                is_variadic: p.is_variadic,
                is_byref: p.is_byref,
                is_optional: p.is_optional,
            })
            .collect();

        Type::single(Atomic::TClosure {
            data: Box::new(mir_types::atomic::ClosureData {
                params: closure_params,
                return_type: return_ty,
                this_type: if af.is_static {
                    None
                } else {
                    ctx.self_fqcn.clone().map(|f| {
                        Type::single(Atomic::TNamedObject {
                            fqcn: Name::from(f.as_ref()),
                            type_params: mir_types::union::empty_type_params(),
                        })
                    })
                },
            }),
        })
    }
}
