use std::sync::Arc;

use php_ast::owned::{ExprKind, MethodCallExpr};
use php_ast::Span;

use crate::narrowing::{extract_any_prop_access, extract_expr_guard_key};
use crate::taint::{classify_method_sink, is_expr_tainted, taint_sink_issue, SinkKind};
use mir_codebase::definitions::{
    Assertion, AssertionKind, DeclaredParam, TemplateParam, Visibility,
};
use mir_issues::{IssueKind, Severity};
use mir_types::{Name, Type};

use crate::expr::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use crate::generic::{
    build_class_bindings, check_template_bounds_with_inheritance, infer_template_bindings,
};
use crate::symbol::ReferenceKind;

use super::args::{
    check_args, check_method_visibility, distinct_spans_for_expansion, expand_sole_spread_arg,
    expr_can_be_passed_by_reference_owned, spread_element_type, substitute_static_in_return,
    CheckArgsParams,
};
use super::CallAnalyzer;

fn extract_namespace(fqcn: &str) -> Option<&str> {
    if let Some(pos) = fqcn.rfind('\\') {
        Some(&fqcn[..pos])
    } else {
        None
    }
}

/// First namespace segment ("root namespace"), or `None` for the global namespace.
/// `@internal` (no argument) scopes a symbol to its root namespace, so any
/// sub-namespace under the same root may use it (Psalm semantics).
fn namespace_root(ns: Option<&str>) -> Option<&str> {
    ns.map(|n| n.trim_start_matches('\\'))
        .and_then(|n| n.split('\\').next())
        .filter(|seg| !seg.is_empty())
}

pub(crate) struct ResolvedMethod {
    pub(crate) owner_fqcn: Arc<str>,
    pub(crate) name: Arc<str>,
    pub(crate) visibility: Visibility,
    pub(crate) deprecated: Option<Arc<str>>,
    pub(crate) is_internal: bool,
    pub(crate) is_static: bool,
    pub(crate) is_abstract: bool,
    pub(crate) is_pure: bool,
    pub(crate) is_mutation_free: bool,
    pub(crate) is_external_mutation_free: bool,
    pub(crate) params: Vec<DeclaredParam>,
    pub(crate) template_params: Vec<TemplateParam>,
    pub(crate) return_ty_raw: Type,
    pub(crate) throws: Arc<[Arc<str>]>,
    pub(crate) no_named_arguments: bool,
    pub(crate) taint_sink_params: Vec<(Arc<str>, Arc<str>)>,
    pub(crate) if_this_is: Option<Arc<Type>>,
    pub(crate) self_out: Option<Arc<Type>>,
    pub(crate) assertions: Vec<Assertion>,
}

/// Resolve a method via the Salsa db, walking the class ancestor chain.
pub(crate) fn resolve_method_from_db(
    db: &dyn crate::db::MirDatabase,
    fqcn: &Arc<str>,
    method_name_lower: &str,
) -> Option<ResolvedMethod> {
    if let Some((owner_fqcn, storage)) = crate::db::find_method_respecting_precedence(
        db,
        crate::db::Fqcn::from_str(db, fqcn.as_ref()),
        method_name_lower,
    ) {
        let name = storage.name.clone();
        let name_lower = if name.bytes().any(|b| b.is_ascii_uppercase()) {
            Arc::<str>::from(crate::util::php_ident_lowercase(&name).as_str())
        } else {
            name.clone()
        };
        let inferred = crate::db::inferred_method_return_type_demand(db, &owner_fqcn, &name_lower);

        // Resolve @inheritDoc: when the method has no docblock-annotated return type
        // or unannotated params, inherit them from the nearest ancestor that has them.
        // A native-hint `mixed` (from_docblock=false) counts as "no docblock type" so
        // that `/** @inheritdoc */ public function f(): mixed {}` still inherits.
        let parent = crate::db::find_inheritdoc_parent(
            db,
            crate::db::Fqcn::from_str(db, fqcn.as_ref()),
            crate::db::Fqcn::from_str(db, owner_fqcn.as_ref()),
            method_name_lower,
            &storage,
        );

        let own_has_docblock_return = storage
            .return_type
            .as_deref()
            .map(|t| t.from_docblock)
            .unwrap_or(false);

        let return_ty_raw = if own_has_docblock_return {
            storage.return_type.clone()
        } else {
            parent
                .as_ref()
                .and_then(|p| p.return_type.clone())
                .or_else(|| storage.return_type.clone())
        }
        .or(inferred)
        .map(|t| (*t).clone())
        .unwrap_or_else(Type::mixed);

        let params: Vec<DeclaredParam> = if let Some(ref p) = parent {
            storage
                .params
                .iter()
                .enumerate()
                .map(|(i, own)| {
                    // Inherit parent param type only when the own type is absent
                    // or is a non-docblock `mixed` hint. A concrete native type
                    // hint (e.g. `A $class`) overrides the parent's narrower
                    // docblock refinement to avoid false positives.
                    let own_ty_is_docblock =
                        own.ty.as_deref().map(|t| t.from_docblock).unwrap_or(false);
                    let own_is_mixed_or_absent =
                        own.ty.as_deref().map(|t| t.is_mixed()).unwrap_or(true);
                    if !own_ty_is_docblock && own_is_mixed_or_absent {
                        if let Some(parent_param) = p.params.get(i) {
                            if parent_param.ty.is_some() {
                                return DeclaredParam {
                                    ty: parent_param.ty.clone(),
                                    ..own.clone()
                                };
                            }
                        }
                    }
                    own.clone()
                })
                .collect()
        } else {
            storage.params.to_vec()
        };

        let template_params = if storage.template_params.is_empty() {
            parent
                .as_ref()
                .map(|p| p.template_params.clone())
                .unwrap_or_default()
        } else {
            storage.template_params.clone()
        };

        let throws: Arc<[Arc<str>]> = if storage.throws.is_empty() {
            parent
                .as_ref()
                .map(|p| Arc::from(p.throws.as_slice()))
                .unwrap_or_else(|| Arc::from([]))
        } else {
            storage.throws.clone().into()
        };

        // `@inheritDoc` with no own `@if-this-is`/`@psalm-self-out` inherits the
        // ancestor's, same as return type/params/throws above — an override
        // that only exists to narrow visibility or add a body shouldn't have
        // to redeclare these to keep them.
        let if_this_is = storage
            .if_this_is
            .clone()
            .or_else(|| parent.as_ref().and_then(|p| p.if_this_is.clone()));
        let self_out = storage
            .self_out
            .clone()
            .or_else(|| parent.as_ref().and_then(|p| p.self_out.clone()));
        let assertions = if storage.assertions.is_empty() {
            parent
                .as_ref()
                .map(|p| p.assertions.clone())
                .unwrap_or_default()
        } else {
            storage.assertions.clone()
        };

        return Some(ResolvedMethod {
            owner_fqcn,
            name,
            visibility: storage.visibility,
            deprecated: storage.deprecated.clone(),
            is_internal: storage.is_internal,
            is_static: storage.is_static,
            is_abstract: storage.is_abstract,
            is_pure: storage.is_pure,
            is_mutation_free: storage.is_mutation_free,
            is_external_mutation_free: storage.is_external_mutation_free,
            params,
            template_params,
            return_ty_raw,
            throws,
            no_named_arguments: storage.no_named_arguments,
            taint_sink_params: storage.taint_sink_params.clone(),
            if_this_is,
            self_out,
            assertions,
        });
    }

    None
}

impl CallAnalyzer {
    pub fn analyze_method_call<'a>(
        ea: &mut ExpressionAnalyzer<'a>,
        call: &MethodCallExpr,
        ctx: &mut FlowState,
        span: Span,
        nullsafe: bool,
    ) -> Type {
        let obj_ty = ea.analyze(&call.object, ctx);

        let method_name = match &call.method.kind {
            ExprKind::Identifier(name) => name.as_ref(),
            _ => {
                ea.analyze(&call.method, ctx);
                ea.record_dynamic_member_access(&obj_ty, call.method.span);
                // Analyze arguments so variables used in them are marked as consumed.
                for arg in &call.args {
                    let Some(value) = &arg.value else { continue };
                    ea.analyze(value, ctx);
                }
                return Type::mixed();
            }
        };

        // Flag explicit __construct() calls.
        // Exception: $this->__construct() inside __wakeup/__clone/__unserialize/unserialize is a
        // documented PHP re-initialization pattern (e.g. after unserialization or cloning).
        // `unserialize()` (no leading double underscore) is the legacy Serializable interface
        // method, not a magic method, but the manual documents the same re-init idiom for it.
        if method_name.eq_ignore_ascii_case("__construct") {
            let receiver_is_this = matches!(
                &call.object.kind,
                ExprKind::Variable(n) if n.trim_start_matches('$') == "this"
            );
            let in_lifecycle_method = ctx.current_method_name.as_deref().is_some_and(|m| {
                m.eq_ignore_ascii_case("__wakeup")
                    || m.eq_ignore_ascii_case("__clone")
                    || m.eq_ignore_ascii_case("__unserialize")
                    || m.eq_ignore_ascii_case("unserialize")
            });
            for atomic in &obj_ty.types {
                if let mir_types::Atomic::TNamedObject { fqcn, .. } = atomic {
                    let exempt = receiver_is_this
                        && ctx.self_fqcn.as_deref() == Some(fqcn.as_ref())
                        && in_lifecycle_method;
                    if !exempt {
                        ea.emit(
                            IssueKind::DirectConstructorCall {
                                class: fqcn.to_string(),
                            },
                            Severity::Error,
                            span,
                        );
                    }
                    break;
                }
            }
        }

        // Pre-mark by-reference parameter variables as defined before evaluating
        // the arguments, so passing an undefined variable to an out-parameter
        // (e.g. `$this->fill($out, …)` where the method declares `&$out`) does not
        // produce a false UndefinedVariable. Mirrors the free-function path.
        // `named_object_fqcn()` is `None` for `TIntersection` — collect that
        // candidate's member fqcns too so an intersection receiver isn't
        // silently skipped (mirrors the member-lookup loop above).
        let method_name_lower = crate::util::php_ident_lowercase(method_name);
        let premark_resolved = obj_ty.remove_null().types.iter().find_map(|a| match a {
            mir_types::Atomic::TIntersection { parts } => parts.iter().find_map(|part| {
                part.types.iter().find_map(|inner| {
                    let fqcn = inner.named_object_fqcn()?;
                    let resolved = crate::db::resolve_receiver_fqcn(ea.db, &ea.file, fqcn);
                    resolve_method_from_db(ea.db, &Arc::from(resolved.as_str()), &method_name_lower)
                })
            }),
            _ => {
                let fqcn = a.named_object_fqcn()?;
                let resolved = crate::db::resolve_receiver_fqcn(ea.db, &ea.file, fqcn);
                resolve_method_from_db(ea.db, &Arc::from(resolved.as_str()), &method_name_lower)
            }
        });
        if let Some(resolved) = premark_resolved {
            super::premark_byref_arg_vars(&resolved.params, &call.args, ctx);
        }

        // Always analyze arguments — even when the receiver is null/mixed and we
        // return early — so that variable reads inside args are tracked and side
        // effects (taint, etc.) are recorded.
        let mut arg_types = super::ARG_TYPES_BUF
            .with(|b| b.borrow_mut().take())
            .unwrap_or_default();
        arg_types.clear();
        // A sole spread arg over a literal, sequentially-keyed shape (e.g.
        // `$obj->make(...['x', 42])`) can be expanded into one binding per
        // element — captured here (before `spread_element_type` collapses it
        // into a single unioned entry) and expanded inside
        // `resolve_method_return`, mirroring `static_call.rs`/`function.rs`.
        let mut sole_spread_ty: Option<Type> = None;
        for arg in call.args.iter() {
            // `None` is a PHP 8.6 partial-application placeholder (`?`/`...`)
            // — not yet modeled; keep positional slots aligned with `mixed`.
            let Some(value) = &arg.value else {
                arg_types.push(Type::mixed());
                continue;
            };
            let ty = ea.analyze(value, ctx);
            super::consume_arg_assignment(value, ctx);
            if arg.unpack && call.args.len() == 1 {
                sole_spread_ty = Some(ty.clone());
            }
            arg_types.push(if arg.unpack {
                spread_element_type(ea.db, &ty)
            } else {
                ty
            });
        }

        let arg_spans: Vec<Span> = call.args.iter().map(|a| a.span).collect();

        // `mixed` already subsumes `null`, so a `mixed | null` receiver is just `mixed`.
        // Such unions arise un-normalized from type inference (e.g. a @template TValue
        // accessor declared @return TValue|null used unbound: TValue → mixed). Skip the
        // nullability diagnostic and let the MixedMethodCall path below own it.
        if obj_ty.contains(|t| matches!(t, mir_types::Atomic::TNull)) && !obj_ty.is_mixed() {
            if nullsafe {
                // ?-> is fine, just returns null on null receiver
            } else if obj_ty.is_single() {
                ea.emit(
                    IssueKind::NullMethodCall {
                        method: method_name.to_string(),
                    },
                    Severity::Error,
                    span,
                );
                return Type::mixed();
            } else {
                ea.emit(
                    IssueKind::PossiblyNullMethodCall {
                        method: method_name.to_string(),
                    },
                    Severity::Info,
                    span,
                );
            }
        }

        // Purity check: calling a method on a parameter in a @pure function
        // OR a @psalm-external-mutation-free method, for a receiver whose
        // type is unknowable (untyped/mixed parameter) — the callee can't
        // be resolved, so warn blanket rather than silently allow it.
        // That's exactly the common case for loosely-typed legacy code,
        // where this check has the most to catch. A resolvable receiver's
        // callee purity is checked precisely below instead, once the
        // method is actually resolved (see the mirrored check in
        // `resolve_method_return`) — narrowing this blanket check to the
        // unresolvable case avoids flagging a call to a provably
        // pure/mutation-free method. Previously only gated on
        // `is_in_pure_fn`, unlike the resolved-callee checks below which
        // also cover `is_in_external_mutation_free_method` for the same
        // param-receiver shape.
        if (ctx.is_in_pure_fn || ctx.is_in_external_mutation_free_method) && obj_ty.is_mixed() {
            if let ExprKind::Variable(recv_name) = &call.object.kind {
                let recv_stripped = recv_name.trim_start_matches('$');
                if ctx
                    .param_names
                    .contains(&mir_types::Name::from(recv_stripped))
                {
                    ea.emit(
                        IssueKind::ImpureMethodCall {
                            method: method_name.to_string(),
                        },
                        Severity::Warning,
                        span,
                    );
                }
            }
        }

        if obj_ty.is_mixed() {
            // Don't report MixedMethodCall on template parameters, since they can be any type
            let is_only_template_params = obj_ty
                .types
                .iter()
                .all(|t| matches!(t, mir_types::Atomic::TTemplateParam { .. }));
            if !is_only_template_params {
                ea.emit(
                    IssueKind::MixedMethodCall {
                        method: method_name.to_string(),
                    },
                    Severity::Info,
                    span,
                );
            }
            // The receiver type is unknowable, so the callee can't be keyed to
            // a class — record a name-only fallback so find-references on any
            // `X::name` can still surface this call as a possible reference.
            ea.record_ref(
                Arc::from(format!(
                    "methname:{}",
                    crate::util::php_ident_lowercase(method_name)
                )),
                call.method.span,
            );
            return Type::mixed();
        }

        let receiver = obj_ty.remove_null();
        // A test-double-style union (e.g. `Real|Mock` from a mocking library's
        // `prophesize()`-style helper) is only ever a `Real` instance in the
        // type system, never at runtime — the real value always satisfies the
        // magic-`__call` sibling. So a sibling atom's catch-all `__call`
        // suppresses UndefinedMethod on atoms that lack the method themselves.
        let union_has_call_magic = receiver.types.iter().any(|atomic| {
            let fqcn = match atomic {
                mir_types::Atomic::TNamedObject { fqcn, .. }
                | mir_types::Atomic::TSelf { fqcn }
                | mir_types::Atomic::TStaticObject { fqcn }
                | mir_types::Atomic::TParent { fqcn } => Some(fqcn),
                _ => None,
            };
            fqcn.is_some_and(|fqcn| {
                let resolved = crate::db::resolve_receiver_fqcn(ea.db, &ea.file, fqcn);
                let resolved: Arc<str> = Arc::from(resolved.as_str());
                crate::db::class_exists(ea.db, &resolved)
                    && crate::db::has_method_in_chain(ea.db, &resolved, "__call")
            })
        });
        let mut result = Type::empty();
        // Declaring class of the resolved method, threaded out of
        // `resolve_method_return` so the symbol-recording loop below does not
        // have to walk the ancestor chain a second time. Only the
        // `TNamedObject` branch feeds it — the recording loop matches
        // top-level `TNamedObject` atomics only.
        let mut declaring = None;
        // `@psalm-self-out` per-atomic accumulator: a union receiver (e.g.
        // `A|B`) must keep every branch that doesn't declare self-out
        // unchanged, and union in the substituted type for every branch that
        // does — not just overwrite the receiver with whichever atomic
        // happened to be processed last.
        let mut self_out_union = Type::empty();
        let mut self_out_used = false;

        for atomic in &receiver.types {
            match atomic {
                mir_types::Atomic::TNamedObject {
                    fqcn,
                    type_params: receiver_type_params,
                } => {
                    let fqcn_resolved = crate::db::resolve_receiver_fqcn(ea.db, &ea.file, fqcn);
                    let fqcn = &std::sync::Arc::from(fqcn_resolved.as_str());
                    let mut this_self_out = None;
                    result.merge_with(&resolve_method_return(
                        ea,
                        ctx,
                        call,
                        span,
                        method_name,
                        fqcn,
                        &receiver_type_params[..],
                        &arg_types,
                        &arg_spans,
                        sole_spread_ty.clone(),
                        &mut declaring,
                        &mut this_self_out,
                        None,
                        union_has_call_magic,
                    ));
                    match this_self_out {
                        Some(ty) => {
                            self_out_used = true;
                            self_out_union.merge_with(&ty);
                        }
                        None => self_out_union.merge_with(&Type::single(atomic.clone())),
                    }
                    // Fallback for unresolvable calls (__call, unknown methods):
                    // key the symbol on the receiver type itself.
                    if declaring.is_none() {
                        declaring = Some(fqcn.clone());
                    }
                }
                mir_types::Atomic::TSelf { fqcn }
                | mir_types::Atomic::TStaticObject { fqcn }
                | mir_types::Atomic::TParent { fqcn } => {
                    let fqcn_resolved = crate::db::resolve_receiver_fqcn(ea.db, &ea.file, fqcn);
                    let fqcn = &std::sync::Arc::from(fqcn_resolved.as_str());
                    let mut this_self_out = None;
                    result.merge_with(&resolve_method_return(
                        ea,
                        ctx,
                        call,
                        span,
                        method_name,
                        fqcn,
                        &[],
                        &arg_types,
                        &arg_spans,
                        sole_spread_ty.clone(),
                        &mut None,
                        &mut this_self_out,
                        None,
                        union_has_call_magic,
                    ));
                    match this_self_out {
                        Some(ty) => {
                            self_out_used = true;
                            self_out_union.merge_with(&ty);
                        }
                        None => self_out_union.merge_with(&Type::single(atomic.clone())),
                    }
                }
                mir_types::Atomic::TIntersection { parts } => {
                    let mut intersection_result = Type::empty();
                    let mut found_method = false;
                    let mut this_self_out = None;
                    let full_receiver_ty = Type::single(atomic.clone());
                    for part in parts.iter() {
                        for inner_atomic in &part.types {
                            if let mir_types::Atomic::TNamedObject {
                                fqcn,
                                type_params: receiver_type_params,
                            } = inner_atomic
                            {
                                let fqcn_resolved =
                                    crate::db::resolve_receiver_fqcn(ea.db, &ea.file, fqcn);
                                let resolved_arc = Arc::from(fqcn_resolved.as_str());
                                if crate::db::has_method_in_chain(ea.db, &resolved_arc, method_name)
                                {
                                    found_method = true;
                                    intersection_result.merge_with(&resolve_method_return(
                                        ea,
                                        ctx,
                                        call,
                                        span,
                                        method_name,
                                        &resolved_arc,
                                        &receiver_type_params[..],
                                        &arg_types,
                                        &arg_spans,
                                        sole_spread_ty.clone(),
                                        &mut None,
                                        &mut this_self_out,
                                        Some(&full_receiver_ty),
                                        union_has_call_magic,
                                    ));
                                }
                            }
                        }
                    }
                    if found_method {
                        result.merge_with(&intersection_result);
                    } else {
                        ea.emit(
                            IssueKind::UndefinedMethod {
                                class: atomic.to_string(),
                                method: method_name.to_string(),
                            },
                            Severity::Error,
                            span,
                        );
                        result.add_type(mir_types::Atomic::TMixed);
                    }
                    // Same atomic-level granularity as the TNamedObject arm
                    // above: if self-out fired for the declaring part, the
                    // WHOLE intersection atomic is replaced by the self-out
                    // type, not merged part-by-part.
                    match this_self_out {
                        Some(ty) => {
                            self_out_used = true;
                            self_out_union.merge_with(&ty);
                        }
                        None => self_out_union.merge_with(&Type::single(atomic.clone())),
                    }
                }
                mir_types::Atomic::TObject | mir_types::Atomic::TTemplateParam { .. } => {
                    result.add_type(mir_types::Atomic::TMixed);
                    self_out_union.merge_with(&Type::single(atomic.clone()));
                }
                mir_types::Atomic::TClosure { data } => {
                    let method_name_lower = crate::util::php_ident_lowercase(method_name);
                    match method_name_lower.as_str() {
                        "bindto" => {
                            // bindTo($newThis, $newScope = 'static'): ?Closure
                            // Preserve the closure's params and return_type, update this_type
                            let new_this = arg_types.first().cloned().unwrap_or_else(Type::null);
                            let this_type = {
                                let non_null = new_this.remove_null();
                                if non_null.is_empty() {
                                    None
                                } else {
                                    Some(non_null)
                                }
                            };
                            let mut bound = Type::single(mir_types::Atomic::TClosure {
                                data: Box::new(mir_types::atomic::ClosureData {
                                    params: data.params.clone(),
                                    return_type: data.return_type.clone(),
                                    this_type,
                                }),
                            });
                            bound.add_type(mir_types::Atomic::TNull);
                            result.merge_with(&bound);
                        }
                        "call" => {
                            // call($newThis, ...$args): mixed
                            // Immediately invokes the closure, returns its return_type (not nullable)
                            result.merge_with(&data.return_type);
                        }
                        _ => {
                            // Other methods (e.g. __invoke) dispatch through the Closure stub
                            let closure_fqcn: Arc<str> = Arc::from("Closure");
                            result.merge_with(&resolve_method_return(
                                ea,
                                ctx,
                                call,
                                span,
                                method_name,
                                &closure_fqcn,
                                &[],
                                &arg_types,
                                &arg_spans,
                                sole_spread_ty.clone(),
                                &mut None,
                                &mut None,
                                None,
                                union_has_call_magic,
                            ));
                        }
                    }
                    self_out_union.merge_with(&Type::single(atomic.clone()));
                }
                _ => {
                    result.add_type(mir_types::Atomic::TMixed);
                    self_out_union.merge_with(&Type::single(atomic.clone()));
                }
            }
        }

        super::ARG_TYPES_BUF.with(|b| {
            let mut g = b.borrow_mut();
            if g.as_ref().map_or(0, |v| v.capacity()) < arg_types.capacity() {
                *g = Some(arg_types);
            }
        });

        if nullsafe && obj_ty.is_nullable() {
            result.add_type(mir_types::Atomic::TNull);
        }

        // Write the merged self-out union once, after every atomic has had a
        // chance to contribute — not per-atomic — so a partially-self-out'd
        // union receiver (e.g. `A|B` where only A declares self-out) keeps
        // the untouched branch instead of collapsing to the last one visited.
        if self_out_used {
            if nullsafe && obj_ty.is_nullable() {
                // `$x?->touch()` never ran the call at all when $x was null,
                // so $x is still possibly null afterward.
                self_out_union.add_type(mir_types::Atomic::TNull);
            }
            if let ExprKind::Variable(recv_name) = &call.object.kind {
                ctx.set_var(recv_name.trim_start_matches('$'), self_out_union);
            } else if let Some((obj_var, prop)) = extract_any_prop_access(&call.object) {
                // `extract_any_prop_access` also matches a nullsafe (`?->`)
                // receiver chain (`$h?->factory->prepare()`), which the
                // plain-`->`-only `extract_prop_access` used to miss here.
                ctx.set_prop_refined(&obj_var, &prop, self_out_union);
            } else if let Some((obj_key, prop)) =
                crate::narrowing::extract_chained_prop_access(&call.object)
            {
                // `$this->a->b->touch()` — a chained (2+ hop) property
                // receiver. `extract_any_prop_access` only matches a
                // bare-variable object, so this fell through entirely
                // before; see `extract_chained_prop_access`'s own doc
                // comment for the synthetic-key encoding.
                ctx.set_prop_refined(&obj_key, &prop, self_out_union);
            } else if let Some((static_fqcn, prop)) =
                crate::narrowing::extract_static_prop_access(&call.object, ctx, ea.db, &ea.file)
            {
                ctx.set_prop_refined(&static_fqcn, &prop, self_out_union);
            }
        }

        let final_ty = if result.is_empty() {
            Type::mixed()
        } else {
            result
        };

        for atomic in &obj_ty.types {
            if let mir_types::Atomic::TNamedObject { .. } = atomic {
                // The declaring class (via the inheritance chain) was threaded
                // out of `resolve_method_return` above so that symbol_at →
                // to_symbol() → references_to uses the same key as record_ref,
                // which also keys by owner_fqcn — without walking the chain a
                // second time.
                let Some(declaring_class) = declaring.take() else {
                    // Receiver names a class we couldn't resolve the method on
                    // (unknown class, undeclared method) — keep a name-only
                    // fallback so find-references can surface the call.
                    ea.record_ref(
                        Arc::from(format!(
                            "methname:{}",
                            crate::util::php_ident_lowercase(method_name)
                        )),
                        call.method.span,
                    );
                    break;
                };
                ea.record_symbol_with_expr_span(
                    call.method.span,
                    span,
                    ReferenceKind::MethodCall {
                        class: declaring_class,
                        method: Arc::from(method_name),
                    },
                    final_ty.clone(),
                );
                break;
            }
        }
        final_ty
    }
}

/// Resolves method return type for a known receiver FQCN, shared between the
/// `TNamedObject` and `TSelf`/`TStaticObject`/`TParent` branches.
///
/// `declaring_class` is set (first resolution wins) to the FQCN of the class
/// that declares the method — reused by the caller for symbol recording so
/// the ancestor chain is only walked once.
///
/// `self_out_out`, if the resolved method declares `@psalm-self-out`, is set
/// to the substituted self-out type for this one atomic — the caller is
/// responsible for merging every atomic's contribution into the receiver's
/// final retyped union (a single atomic's resolution must not unilaterally
/// overwrite a union receiver's other branches).
///
/// `sibling_has_call_magic` suppresses UndefinedMethod, and (via
/// `arity_unknown`) TooFewArguments/TooManyArguments, when a DIFFERENT atom
/// in the same union receiver has a catch-all `__call` — a real|test-double
/// union is never actually the "real" atom at runtime.
#[allow(clippy::too_many_arguments)]
fn resolve_method_return<'a>(
    ea: &mut ExpressionAnalyzer<'a>,
    ctx: &mut FlowState,
    call: &MethodCallExpr,
    span: Span,
    method_name: &str,
    fqcn: &Arc<str>,
    receiver_type_params: &[Type],
    arg_types: &[Type],
    arg_spans: &[Span],
    sole_spread_ty: Option<Type>,
    declaring_class: &mut Option<Arc<str>>,
    self_out_out: &mut Option<Type>,
    full_receiver: Option<&Type>,
    sibling_has_call_magic: bool,
) -> Type {
    let method_name_lower = crate::util::php_ident_lowercase(method_name);
    let resolved = resolve_method_from_db(ea.db, fqcn, &method_name_lower);

    if let Some(resolved) = resolved {
        if declaring_class.is_none() {
            *declaring_class = Some(resolved.owner_fqcn.clone());
        }
        ea.record_ref(
            Arc::from(format!(
                "meth:{}::{}",
                resolved.owner_fqcn,
                crate::util::php_ident_lowercase(&resolved.name)
            )),
            call.method.span,
        );
        if let Some(msg) = resolved.deprecated.clone() {
            ea.emit(
                IssueKind::DeprecatedMethod {
                    class: fqcn.to_string(),
                    method: method_name.to_string(),
                    message: Some(msg).filter(|m| !m.is_empty()),
                },
                Severity::Info,
                span,
            );
        }
        if method_name != resolved.name.as_ref()
            && method_name.eq_ignore_ascii_case(resolved.name.as_ref())
        {
            ea.emit(
                IssueKind::WrongCaseMethod {
                    class: fqcn.to_string(),
                    used: method_name.to_string(),
                    canonical: resolved.name.to_string(),
                },
                Severity::Info,
                call.method.span,
            );
        }
        // Immutability check: calling a non-mutation-free instance method on $this
        // inside a @psalm-immutable class or @psalm-mutation-free method is forbidden
        // because it may indirectly mutate object state.
        if ctx.is_in_immutable_method
            && !resolved.is_mutation_free
            && !resolved.is_pure
            && !resolved.is_static
            && crate::expr::root_receiver_var(&call.object)
                .is_some_and(|n| n.trim_start_matches('$') == "this")
        {
            ea.emit(
                IssueKind::ImpureMethodCall {
                    method: method_name.to_string(),
                },
                Severity::Warning,
                span,
            );
        }
        // External-mutation-free check: calling a method that can mutate its receiver
        // (`$this` inside the callee) on a parameter is forbidden in a
        // @psalm-external-mutation-free method — it would indirectly mutate the
        // argument. Only @pure and @mutation-free callees are safe because they
        // guarantee not to mutate their own `$this`.
        if ctx.is_in_external_mutation_free_method
            && !resolved.is_pure
            && !resolved.is_mutation_free
            && !resolved.is_static
        {
            if let Some(recv_name) = crate::expr::root_receiver_var(&call.object) {
                let recv_stripped = recv_name.trim_start_matches('$');
                if recv_stripped != "this"
                    && ctx
                        .param_names
                        .contains(&mir_types::Name::from(recv_stripped))
                {
                    ea.emit(
                        IssueKind::ImpureMethodCall {
                            method: method_name.to_string(),
                        },
                        Severity::Warning,
                        span,
                    );
                }
            }
        }
        // Purity check: calling a non-pure/non-mutation-free method on a
        // parameter inside a @pure function may have side effects or
        // indirectly mutate the argument. Mirrors the blanket,
        // resolution-agnostic check above (for unresolvable/mixed
        // receivers) but only fires here once the callee's own purity is
        // known, so a call to a provably pure/mutation-free method on a
        // resolvable receiver isn't flagged.
        if ctx.is_in_pure_fn
            && !resolved.is_pure
            && !resolved.is_mutation_free
            && !resolved.is_static
        {
            if let Some(recv_name) = crate::expr::root_receiver_var(&call.object) {
                let recv_stripped = recv_name.trim_start_matches('$');
                if recv_stripped != "this"
                    && ctx
                        .param_names
                        .contains(&mir_types::Name::from(recv_stripped))
                {
                    ea.emit(
                        IssueKind::ImpureMethodCall {
                            method: method_name.to_string(),
                        },
                        Severity::Warning,
                        span,
                    );
                }
            }
        }
        // Same "argument passed into a not-provably-safe callee" check as
        // `new X(...)` (expr/objects.rs), free-function calls
        // (call/function.rs), and static calls (call/static_call.rs) — a
        // plain, by-value argument reachable from `$this`/a parameter can
        // still be stored/mutated by a callee that isn't provably pure, even
        // though the RECEIVER checks above only cover the call's own object.
        if (ctx.is_in_immutable_method || ctx.is_in_external_mutation_free_method)
            && !resolved.is_pure
            && !resolved.is_mutation_free
        {
            for arg in call.args.iter() {
                let Some(value) = &arg.value else { continue };
                let Some(recv_name) = crate::expr::root_receiver_var(value) else {
                    continue;
                };
                let recv_stripped = recv_name.trim_start_matches('$');
                let reachable = (ctx.is_in_immutable_method && recv_stripped == "this")
                    || (ctx.is_in_external_mutation_free_method
                        && recv_stripped != "this"
                        && ctx.param_names.contains(&Name::from(recv_stripped)));
                if !reachable {
                    continue;
                }
                let arg_is_object = crate::expr::assignment::resolve_chained_receiver_type(
                    value, ctx, ea.db, &ea.file,
                )
                .is_some_and(|ty| {
                    ty.types.iter().any(|a| {
                        matches!(
                            a,
                            mir_types::Atomic::TNamedObject { .. }
                                | mir_types::Atomic::TSelf { .. }
                                | mir_types::Atomic::TStaticObject { .. }
                                | mir_types::Atomic::TParent { .. }
                        )
                    })
                });
                if arg_is_object {
                    ea.emit(
                        IssueKind::ImpureMethodCall {
                            method: method_name.to_string(),
                        },
                        Severity::Warning,
                        span,
                    );
                    break;
                }
            }
        }
        if resolved.is_internal {
            let calling_ns = ea.db.file_namespace(&ea.file);
            let calling_root = namespace_root(calling_ns.as_deref());
            let owner_root = namespace_root(extract_namespace(&resolved.owner_fqcn));
            // Calling an @internal method on $this (self-call or inherited) is allowed —
            // trait methods become part of the using class, and child classes may call
            // parent/trait @internal methods that are part of their own API.
            let is_self_call = ctx
                .self_fqcn
                .as_deref()
                .map(|s| s.eq_ignore_ascii_case(fqcn.as_ref()))
                .unwrap_or(false);
            if calling_root != owner_root && !is_self_call {
                ea.emit(
                    IssueKind::InternalMethod {
                        class: fqcn.to_string(),
                        method: method_name.to_string(),
                    },
                    Severity::Warning,
                    span,
                );
            }
        }
        check_method_visibility(
            ea,
            resolved.visibility,
            &resolved.owner_fqcn,
            &resolved.name,
            ctx,
            span,
        );

        let mut arg_names: Vec<Option<String>> = call
            .args
            .iter()
            .map(|a| a.name.as_ref().map(crate::parser::name_to_string_owned))
            .collect();
        let mut arg_can_be_byref: Vec<bool> = call
            .args
            .iter()
            .map(|a| {
                a.value
                    .as_ref()
                    .is_some_and(expr_can_be_passed_by_reference_owned)
            })
            .collect();
        // `effective_arg_types`/`effective_arg_spans` are kept distinct from
        // the `arg_types`/`arg_spans` parameters (rather than shadowing them)
        // because those are still used below in their original, unexpanded
        // form for plugin hooks and conditional-return resolution, which key
        // off the original per-argument positions.
        let mut effective_arg_types = arg_types.to_vec();
        let mut effective_arg_spans = arg_spans.to_vec();
        let mut has_spread = call.args.iter().any(|a| a.unpack);
        // A sole spread arg over a literal, sequentially-keyed shape can be
        // expanded into one binding per element so each parameter (and
        // template-binding inference below) is checked individually instead
        // of only the first (see expand_sole_spread_arg) — mirrors
        // static_call.rs/function.rs, which already do this for their own
        // call forms. `arity_unknown` stays true even after expansion — PHP
        // allows extra/spread positional args, so a concretely-known count
        // still shouldn't trigger TooFew/TooManyArguments.
        //
        // A sibling atom's catch-all `__call` also forces `arity_unknown`:
        // the real runtime value may actually be that sibling (test-double
        // idiom), which never enforces this atom's own arity at all — so a
        // mismatch here can't be trusted as a genuine error, same reasoning
        // `union_has_call_magic` already applies to UndefinedMethod above.
        let mut arity_unknown = has_spread || sibling_has_call_magic;
        if let Some(expanded) = sole_spread_ty.and_then(|t| expand_sole_spread_arg(&t)) {
            effective_arg_spans =
                distinct_spans_for_expansion(effective_arg_spans[0], expanded.len());
            arg_names = expanded
                .iter()
                .map(|(name, _)| name.as_ref().map(|n| n.to_string()))
                .collect();
            arg_can_be_byref = vec![false; expanded.len()];
            effective_arg_types = expanded.into_iter().map(|(_, ty)| ty).collect();
            has_spread = false;
            arity_unknown = true;
        }
        // The resolved method has no concrete body of its own (an `abstract`
        // method, or any interface method — interfaces never carry a body) —
        // whatever object this receiver actually holds at runtime is some
        // OTHER concrete (sub)class providing the real implementation, and
        // PHP's override-compatibility rules let that override freely ADD
        // extra optional params beyond this signature. Only TooMany is
        // suppressed: an override can never REQUIRE more params than an
        // abstract/interface method declares, so the required-param count is
        // still a reliable floor.
        let too_many_arity_unknown = resolved.is_abstract
            || crate::db::class_kind(ea.db, &resolved.owner_fqcn).is_some_and(|k| k.is_interface);
        // Build class-level template bindings before arg-checking so we can substitute
        // template params (e.g. T → int from Box<int>) into param types. A plain
        // subclass that doesn't redeclare `@template` (`class IntBox extends
        // Box {}`) still carries `receiver_type_params` positioned against
        // Box's own template list (see `infer_new_type_params`), so binding
        // must walk up to that same ancestor instead of finding zero
        // templates on `fqcn` itself and discarding every bound type param.
        let class_tps = crate::db::class_template_params(ea.db, fqcn)
            .map(|tps| tps.to_vec())
            .unwrap_or_default();
        let mut bindings = build_class_bindings(&class_tps, receiver_type_params);
        let inherited_bindings = crate::db::inherited_template_bindings(ea.db, fqcn, &bindings);
        if resolved.owner_fqcn.as_ref() == fqcn.as_ref() {
            // The called method is declared directly on the receiver's own
            // class — a bare template name in its signature is the
            // receiver's OWN template, so it must win over a same-named but
            // unrelated ancestor template (only fill in names `bindings`
            // doesn't already have).
            for (k, v) in inherited_bindings {
                bindings.entry(k).or_insert(v);
            }
        } else {
            // The method is inherited from `resolved.owner_fqcn` — a bare
            // template name in ITS signature is scoped to that owner's own
            // declaration, which the ancestor-chain walk resolves; it must
            // win over a same-named receiver-own template.
            bindings.extend(inherited_bindings);
        }

        // A class-level `@template T of Bound` was previously only ever checked
        // at `new Box(...)` construction sites — a receiver typed `Box<NotAnimal>`
        // via a docblock/param annotation instead (no constructor call in sight)
        // sailed through every method call unchecked, regardless of whether the
        // called method itself declares any template params of its own.
        for (name, inferred, bound) in check_template_bounds_with_inheritance(
            ea.db,
            &bindings,
            &class_tps,
            &Default::default(),
            Some(fqcn.as_ref()),
        ) {
            ea.emit(
                IssueKind::InvalidTemplateParam {
                    name: name.to_string(),
                    expected_bound: format!("{bound}"),
                    actual: format!("{inferred}"),
                },
                Severity::Error,
                span,
            );
        }

        // Substitute class bindings into param types so argument checking resolves T → int etc.
        // A method-level `@template T` SHADOWS a same-named class template: its
        // occurrences in param types must stay unbound here so `check_args` can
        // infer them from the arguments instead (e.g. ReflectionClass<Foo> with
        // `getAttributes(class-string<T>|null $name)` redeclaring T).
        let mut param_bindings = bindings.clone();
        for tp in resolved.template_params.iter() {
            param_bindings.remove(&Name::from(tp.name.as_ref()));
        }
        let substituted_params: Vec<DeclaredParam>;
        let effective_params: &[DeclaredParam] = if param_bindings.is_empty() {
            &resolved.params
        } else {
            substituted_params = resolved
                .params
                .iter()
                .map(|p| DeclaredParam {
                    ty: mir_codebase::wrap_param_type(
                        p.ty.as_ref()
                            .map(|t| t.substitute_templates(&param_bindings)),
                    ),
                    out_ty: mir_codebase::wrap_param_type(
                        p.out_ty
                            .as_ref()
                            .map(|t| t.substitute_templates(&param_bindings)),
                    ),
                    ..p.clone()
                })
                .collect();
            &substituted_params
        };

        check_args(
            ea,
            CheckArgsParams {
                fn_name: method_name,
                params: effective_params,
                arg_types: &effective_arg_types,
                arg_spans: &effective_arg_spans,
                arg_names: &arg_names,
                arg_can_be_byref: &arg_can_be_byref,
                call_span: span,
                has_spread,
                arity_unknown,
                too_many_arity_unknown,
                template_params: &resolved.template_params,
                no_named_arguments: resolved.no_named_arguments,
            },
        );

        // A call we can't prove pure/mutation-free may reassign the
        // receiver's own properties from inside the callee (its `$this` IS
        // this receiver), staling any narrowing recorded before the call —
        // e.g. `$this->user = $u; $this->reset(); $this->user->getId();`
        // must not still see `$this->user` as non-null after `reset()`.
        if !resolved.is_static && !resolved.is_pure && !resolved.is_mutation_free {
            if let ExprKind::Variable(recv_name) = &call.object.kind {
                ctx.invalidate_prop_refined_receiver(recv_name);
            }
        }
        // Similarly, an object passed as an argument to a call that isn't
        // proven pure/external-mutation-free may have its own properties
        // reassigned inside the callee, regardless of which object receives
        // the call itself (e.g. `$this->save($logger)` mutating `$logger`).
        if !resolved.is_pure && !resolved.is_external_mutation_free {
            for arg in call.args.iter() {
                if let Some(ExprKind::Variable(name)) = arg.value.as_ref().map(|v| &v.kind) {
                    ctx.invalidate_prop_refined_receiver(name);
                }
            }
        }

        // Taint sink check: emit the matching Tainted* issue when a tainted
        // value reaches a @taint-sink annotated parameter.
        if !resolved.taint_sink_params.is_empty() {
            for (param_name, sink_kind) in &resolved.taint_sink_params {
                // Find positional index of this param in the method's param list.
                let param_idx = resolved
                    .params
                    .iter()
                    .position(|p| p.name.as_ref() == param_name.as_ref());
                // A variadic sink parameter (`...$args`) swallows every
                // trailing positional argument from its index onward, not
                // just the first — check them all.
                let is_variadic = param_idx
                    .and_then(|idx| resolved.params.get(idx))
                    .is_some_and(|p| p.is_variadic);
                let args: Vec<&php_ast::owned::Arg> = if is_variadic {
                    let idx = param_idx.unwrap();
                    call.args
                        .iter()
                        .filter(|a| a.name.is_none())
                        .skip(idx)
                        .collect()
                } else {
                    let positional = param_idx.and_then(|idx| call.args.get(idx));
                    // Also check named args.
                    let named_arg = call.args.iter().find(|a| {
                        a.name
                            .as_ref()
                            .map(|n| crate::parser::name_to_string_owned(n) == param_name.as_ref())
                            .unwrap_or(false)
                    });
                    positional.or(named_arg).into_iter().collect()
                };
                for arg in args {
                    let Some(value) = &arg.value else { continue };
                    if is_expr_tainted(value, ctx, ea.db, &ea.file) {
                        ea.emit(taint_sink_issue(sink_kind), Severity::Error, span);
                    }
                }
            }
        }

        // OOP database sink check (M19 parity with call/function.rs's
        // procedural classify_sink): $pdo->query($sql) etc.
        if let Some(sink_kind) = classify_method_sink(ea.db, fqcn.as_ref(), method_name) {
            for arg in call.args.iter() {
                let Some(value) = &arg.value else { continue };
                if is_expr_tainted(value, ctx, ea.db, &ea.file) {
                    let issue_kind = match sink_kind {
                        SinkKind::Sql => IssueKind::TaintedSql,
                        _ => unreachable!("classify_method_sink only returns SinkKind::Sql"),
                    };
                    ea.emit(issue_kind, Severity::Error, span);
                    break;
                }
            }
        }

        let ret_raw =
            substitute_static_in_return(resolved.return_ty_raw, fqcn, receiver_type_params);

        if !resolved.template_params.is_empty() {
            let (method_bindings, unchecked) = infer_template_bindings(
                ea.db,
                &resolved.template_params,
                effective_params,
                &effective_arg_types,
                &arg_names,
            );
            // Only warn about template shadowing when the declaring class lives
            // in the file under analysis — a shadow inside a stub or vendor
            // class is the library's concern, not this call site's.
            let declared_here = crate::db::class_like_decl_file(
                ea.db,
                crate::db::Fqcn::from_str(ea.db, resolved.owner_fqcn.as_ref()),
            )
            .is_some_and(|f| f.as_ref() == ea.file.as_ref());
            if declared_here {
                for key in method_bindings.keys() {
                    if bindings.contains_key(key) {
                        ea.emit(
                            IssueKind::ShadowedTemplateParam {
                                name: key.to_string(),
                            },
                            Severity::Info,
                            span,
                        );
                    }
                }
            }
            bindings.extend(method_bindings);
            for (name, inferred, bound) in check_template_bounds_with_inheritance(
                ea.db,
                &bindings,
                &resolved.template_params,
                &unchecked,
                Some(fqcn.as_ref()),
            ) {
                ea.emit(
                    IssueKind::InvalidTemplateParam {
                        name: name.to_string(),
                        expected_bound: format!("{bound}"),
                        actual: format!("{inferred}"),
                    },
                    Severity::Error,
                    span,
                );
            }
        }

        // `@if-this-is X<Y>`: the method may only be called when the receiver
        // satisfies the constraint. We can only judge this when the receiver's
        // type arguments are known — an unparameterized receiver (e.g.
        // `new Foo()` with no `@var`) carries nothing to contradict. Checked
        // here (after `bindings` includes this call's own inferred method-
        // level template bindings, not right at method resolution) so a
        // constraint referencing the METHOD's own `@template` (`@if-this-is
        // Box<U>` where `U` is this call's own template, inferred from an
        // argument) is substituted before the comparison — otherwise `U`
        // stays a bare, never-excluded template atom and the check can never
        // fail for its own template params (it already worked correctly for
        // a constraint referencing only the CLASS's template, since that's
        // already resolved into `receiver_type_params` by this point).
        if let Some(original_constraint) = resolved.if_this_is.clone() {
            let constraint = original_constraint.substitute_templates(&bindings);
            let constraint_has_params = constraint.types.iter().any(|a| {
                matches!(a, mir_types::Atomic::TNamedObject { type_params, .. } if !type_params.is_empty())
            });
            // Receiver type args that are still unresolved template vars (e.g.
            // a call on `$this` inside the generic class body) carry no concrete
            // instantiation to judge against — skip rather than risk a false
            // mismatch.
            let receiver_has_unresolved_template = receiver_type_params.iter().any(|t| {
                t.types
                    .iter()
                    .any(|a| matches!(a, mir_types::Atomic::TTemplateParam { .. }))
            });
            if !receiver_has_unresolved_template
                && (!receiver_type_params.is_empty() || !constraint_has_params)
            {
                // For an intersection-typed receiver, `full_receiver` carries
                // every part (`Foo&HasBar`), not just the one part that
                // declares the called method — checking only the bare
                // declaring atom against a constraint naming a SIBLING part
                // (`@if-this-is HasBar` on a method declared in `Foo`) would
                // false-positive since `Foo` alone doesn't implement
                // `HasBar`, even though the actual receiver does via the
                // intersection.
                let receiver = full_receiver.cloned().unwrap_or_else(|| {
                    Type::single(mir_types::Atomic::TNamedObject {
                        fqcn: Name::new(fqcn.as_ref()),
                        type_params: receiver_type_params.to_vec().into(),
                    })
                });
                if !crate::subtype::is_subtype(ea.db, &receiver, &constraint) {
                    ea.emit(
                        IssueKind::IfThisIsMismatch {
                            class: fqcn.to_string(),
                            method: method_name.to_string(),
                            // The message displays the ORIGINAL, unsubstituted
                            // constraint (`Box<U>`, not `Box<mixed>`) — a
                            // method template with no way to be inferred
                            // (never appears in a real @param) substitutes to
                            // `mixed`, which renders as if the generic were
                            // entirely absent (`Box`) instead of keeping the
                            // more informative raw template name. Only the
                            // actual subtype CHECK needs the substituted form.
                            expected: format!("{original_constraint}"),
                            actual: format!("{receiver}"),
                        },
                        Severity::Info,
                        span,
                    );
                }
            }
        }

        // Check inter-procedural throws: if callee declares @throws, check if caller covers them.
        // Unchecked exceptions (RuntimeException / LogicException descendants) are skipped by
        // PHP convention — see [`is_unchecked_exception`].
        let guarded_by_reflection_throws_guard = extract_expr_guard_key(&call.object, ctx, ea.db, &ea.file)
            .map(|key| {
                ctx.reflection_throws_guards.contains(&(
                    key,
                    Arc::from(crate::util::php_ident_lowercase(method_name).as_str()),
                ))
            })
            .unwrap_or(false);
        for callee_throw in resolved.throws.iter() {
            if crate::db::is_unchecked_exception(ea.db, callee_throw.as_ref()) {
                continue;
            }
            let is_covered = |declared: &Arc<str>| {
                declared.as_ref() == callee_throw.as_ref()
                    || crate::db::extends_or_implements(
                        ea.db,
                        callee_throw.as_ref(),
                        declared.as_ref(),
                    )
            };
            if !ctx.fn_declared_throws.iter().any(is_covered)
                && !ctx.locally_caught_throws.iter().any(is_covered)
                && !guarded_by_reflection_throws_guard
            {
                ea.emit(
                    IssueKind::MissingThrowsDocblock {
                        class: callee_throw.to_string(),
                    },
                    Severity::Info,
                    span,
                );
            }
        }

        // Write @param-out types back to caller variables for by-ref params.
        // Substitute the full `bindings` map — the receiver's own bound class
        // template (e.g. Box<int>'s T -> int) plus this call's own inferred
        // method-level template bindings, merged in above — the same combined
        // approach static_call.rs's out_bindings already uses. `effective_params`
        // deliberately strips the method's own template names (so check_args can
        // still infer them from the arguments), so it can't be reused here: its
        // out_ty would leak the bare method template atom to the caller.
        // A by-ref argument that's a property (`array_push($this->items, $n)`
        // called through a helper method) mutates it exactly as much as an
        // explicit `$this->items = …` write would, regardless of whether the
        // param also declares `@param-out` — checked in its own pass since
        // the loop below only ever runs for `@param-out`-declared params.
        for (i, param) in resolved.params.iter().enumerate() {
            if !param.is_byref {
                continue;
            }
            if param.is_variadic {
                for arg in call.args.iter().skip(i) {
                    let Some(value) = &arg.value else { continue };
                    ea.check_byref_arg_purity(value, ctx, value.span);
                }
            } else if let Some(value) =
                super::resolve_named_arg_type_index(&resolved.params, &call.args, i)
                    .and_then(|idx| call.args.get(idx))
                    .and_then(|arg| arg.value.as_ref())
            {
                ea.check_byref_arg_purity(value, ctx, value.span);
            }
        }

        // A by-ref output parameter's written-back value derives from the
        // call's own arguments (e.g. `preg_match`'s `$matches` derives from
        // the tainted subject) — conservatively taint (or clear) it the same
        // "sticky" way `.=`/`??=` already treat their own result, rather
        // than leaving its taint bit untouched from whatever it happened to
        // hold before the call.
        let any_arg_tainted = call.args.iter().any(|arg| {
            arg.value
                .as_ref()
                .is_some_and(|v| is_expr_tainted(v, ctx, ea.db, &ea.file))
        });
        for (i, param) in resolved.params.iter().enumerate() {
            let Some(out_ty) = param.out_ty.as_ref() else {
                continue;
            };
            // `@param-out self`/`@param-out static` must resolve to the receiver's
            // concrete class, the same way `@return static` already does.
            let out_ty =
                substitute_static_in_return((**out_ty).clone(), fqcn, receiver_type_params);
            let out_ty = if bindings.is_empty() {
                out_ty
            } else {
                out_ty.substitute_templates(&bindings)
            };
            if param.is_variadic {
                for arg in call.args.iter().skip(i) {
                    if let Some(php_ast::owned::ExprKind::Variable(name)) =
                        arg.value.as_ref().map(|v| &v.kind)
                    {
                        let var_name = name.as_ref().trim_start_matches('$');
                        ctx.set_var(var_name, out_ty.clone());
                        if any_arg_tainted {
                            ctx.taint_var(var_name);
                        } else {
                            ctx.clear_var_taint(var_name);
                        }
                    }
                }
            } else if let Some(value) =
                super::resolve_named_arg_type_index(&resolved.params, &call.args, i)
                    .and_then(|idx| call.args.get(idx))
                    .and_then(|arg| arg.value.as_ref())
            {
                if let php_ast::owned::ExprKind::Variable(name) = &value.kind {
                    let var_name = name.as_ref().trim_start_matches('$');
                    ctx.set_var(var_name, out_ty);
                    if any_arg_tainted {
                        ctx.taint_var(var_name);
                    } else {
                        ctx.clear_var_taint(var_name);
                    }
                }
            }
        }

        // `@psalm-self-out Type` — report how this call narrows/changes the
        // receiver (including `$this`) for this one atomic. The caller merges
        // every atomic's contribution and writes the receiver variable once
        // (including `$this`), the same way a by-ref `@param-out` retypes its
        // argument above.
        if let Some(self_out_raw) = resolved.self_out.clone() {
            let self_out_ty =
                substitute_static_in_return((*self_out_raw).clone(), fqcn, receiver_type_params);
            let self_out_ty = if !bindings.is_empty() {
                // Widen literal argument types (e.g. a bare `"hello"` binding
                // `U`) before substituting — carrying a literal into the
                // receiver's type params is over-narrow and risks false
                // positives downstream, same as `widen_type_param` for `new`.
                let mut widened_bindings = bindings.clone();
                for v in widened_bindings.values_mut() {
                    *v = crate::stmt::widen_for_check(v.clone());
                }
                self_out_ty.substitute_templates(&widened_bindings)
            } else {
                self_out_ty
            };
            *self_out_out = Some(self_out_ty);
        }

        // Bare-statement `@psalm-assert` — the method-call counterpart of
        // `call/function.rs`'s unconditional-assert block. Reuses the exact
        // per-assertion body the conditional if-true/if-false dispatch uses
        // (`apply_one_assertion`), which this hand-duplicated loop never
        // did: it never read `assertion.param_key`, never handled a
        // variadic param, and never resolved a named argument. `bindings`
        // is the fully merged class + method template scope computed
        // above, so a class-level `T` or a method-level `T` in the
        // assertion's type both substitute correctly.
        for assertion in resolved
            .assertions
            .iter()
            .filter(|a| a.kind == AssertionKind::Assert)
        {
            crate::narrowing::apply_one_assertion(
                assertion,
                &resolved.params,
                &call.args,
                Some(&call.object),
                ctx,
                Some(&bindings),
                ea.db,
                &ea.file,
            );
        }

        let return_ty = if !bindings.is_empty() {
            ret_raw.substitute_templates(&bindings)
        } else {
            ret_raw
        };
        let mut return_ty =
            crate::call::resolve_conditional_return(return_ty, ea.db, |param_name| {
                // `@return ($this is X ? A : B)`: `$this` is never a declared
                // parameter, so the lookup below always misses it — resolve it
                // to the receiver's own concrete type instead, mirroring the
                // `@if-this-is` receiver construction just above.
                if param_name == "this" {
                    return Some(Type::single(mir_types::Atomic::TNamedObject {
                        fqcn: Name::new(fqcn.as_ref()),
                        type_params: receiver_type_params.to_vec().into(),
                    }));
                }
                resolved
                    .params
                    .iter()
                    .position(|p| p.name.as_ref() == param_name)
                    .and_then(|idx| {
                        crate::call::resolve_named_arg_type_index(&resolved.params, &call.args, idx)
                    })
                    .and_then(|idx| arg_types.get(idx))
                    .cloned()
            });
        ea.apply_method_call_plugins(
            fqcn.as_ref(),
            resolved.owner_fqcn.as_ref(),
            method_name,
            &call.args,
            arg_types,
            span,
            &mut return_ty,
        );
        return_ty
    } else if crate::db::class_exists(ea.db, fqcn) && !crate::db::has_unknown_ancestor(ea.db, fqcn)
    {
        let (is_interface, is_abstract, is_trait) = crate::db::class_kind(ea.db, fqcn)
            .map(|k| (k.is_interface, k.is_abstract, k.is_trait))
            .unwrap_or((false, false, false));
        // Check for __call in the full inheritance chain (not just direct methods)
        let has_call_magic = crate::db::has_method_in_chain(ea.db, fqcn, "__call");
        // A trait body's $this is the future consuming class — the method may
        // be provided by the consumer, so an unresolved call is not undefined.
        // Also suppress when caller guarded with `method_exists($obj, 'method')`.
        let guarded_by_method_exists = extract_expr_guard_key(&call.object, ctx, ea.db, &ea.file)
            .map(|key| {
                ctx.method_exists_guards.contains(&(
                    key,
                    Arc::from(crate::util::php_ident_lowercase(method_name).as_str()),
                ))
            })
            .unwrap_or(false);
        if is_trait {
            // The call may be satisfied by whichever class ends up consuming
            // this trait — record a per-trait marker so DeadCodeAnalyzer can
            // credit any composing class's own private method of this name
            // as used, instead of only ever seeing the trait's own (failed)
            // resolution attempt.
            ea.record_ref(
                Arc::from(format!("traituse:{fqcn}::{method_name_lower}")),
                call.method.span,
            );
        }
        if has_call_magic {
            // `__call`'s own declared return type is more precise than a blanket
            // `mixed` — e.g. a fluent test-double stub typed `@return static`.
            resolve_method_from_db(ea.db, fqcn, "__call")
                .map(|call_magic| {
                    substitute_static_in_return(
                        call_magic.return_ty_raw,
                        fqcn,
                        receiver_type_params,
                    )
                })
                .unwrap_or_else(Type::mixed)
        } else if is_interface
            || is_abstract
            || is_trait
            || guarded_by_method_exists
            || sibling_has_call_magic
        {
            Type::mixed()
        } else {
            ea.emit(
                IssueKind::UndefinedMethod {
                    class: fqcn.to_string(),
                    method: method_name.to_string(),
                },
                Severity::Error,
                span,
            );
            Type::mixed()
        }
    } else {
        Type::mixed()
    }
}
