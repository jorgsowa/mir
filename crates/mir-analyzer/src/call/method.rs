use std::sync::Arc;

use php_ast::owned::{ExprKind, MethodCallExpr};
use php_ast::Span;

use crate::narrowing::extract_expr_guard_key;
use crate::taint::is_expr_tainted;
use mir_codebase::storage::{FnParam, TemplateParam, Visibility};
use mir_issues::{IssueKind, Severity};
use mir_types::{Name, Type};

use crate::expr::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use crate::generic::{
    build_class_bindings, check_template_bounds_with_inheritance, infer_template_bindings,
};
use crate::symbol::ReferenceKind;

use super::args::{
    check_args, check_method_visibility, expr_can_be_passed_by_reference_owned,
    spread_element_type, substitute_static_in_return, CheckArgsParams,
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
    pub(crate) params: Vec<FnParam>,
    pub(crate) template_params: Vec<TemplateParam>,
    pub(crate) return_ty_raw: Type,
    pub(crate) throws: Arc<[Arc<str>]>,
    pub(crate) no_named_arguments: bool,
    pub(crate) taint_sink_params: Vec<(Arc<str>, Arc<str>)>,
    pub(crate) if_this_is: Option<Arc<Type>>,
}

/// Resolve a method via the Salsa db, walking the class ancestor chain.
pub(crate) fn resolve_method_from_db(
    ea: &ExpressionAnalyzer<'_>,
    fqcn: &Arc<str>,
    method_name_lower: &str,
) -> Option<ResolvedMethod> {
    let db = ea.db;

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

        let params: Vec<FnParam> = if let Some(ref p) = parent {
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
                                return FnParam {
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

        return Some(ResolvedMethod {
            owner_fqcn,
            name,
            visibility: storage.visibility,
            deprecated: storage.deprecated.clone(),
            is_internal: storage.is_internal,
            is_static: storage.is_static,
            is_abstract: storage.is_abstract,
            params,
            template_params,
            return_ty_raw,
            throws,
            no_named_arguments: storage.no_named_arguments,
            taint_sink_params: storage.taint_sink_params.clone(),
            if_this_is: storage.if_this_is.clone(),
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
                // Analyze arguments so variables used in them are marked as consumed.
                for arg in &call.args {
                    ea.analyze(&arg.value, ctx);
                }
                return Type::mixed();
            }
        };

        // Flag explicit __construct() calls
        if method_name.eq_ignore_ascii_case("__construct") {
            // Detect the class from the object type
            for atomic in &obj_ty.types {
                if let mir_types::Atomic::TNamedObject { fqcn, .. } = atomic {
                    ea.emit(
                        IssueKind::DirectConstructorCall {
                            class: fqcn.to_string(),
                        },
                        Severity::Error,
                        span,
                    );
                    break;
                }
            }
        }

        // Pre-mark by-reference parameter variables as defined before evaluating
        // the arguments, so passing an undefined variable to an out-parameter
        // (e.g. `$this->fill($out, …)` where the method declares `&$out`) does not
        // produce a false UndefinedVariable. Mirrors the free-function path.
        if let Some(fqcn) = obj_ty
            .remove_null()
            .types
            .iter()
            .find_map(|a| a.named_object_fqcn())
        {
            let fqcn_resolved = crate::db::resolve_name(ea.db, &ea.file, fqcn);
            let fqcn_arc: Arc<str> = Arc::from(fqcn_resolved.as_str());
            if let Some(resolved) = resolve_method_from_db(
                ea,
                &fqcn_arc,
                &crate::util::php_ident_lowercase(method_name),
            ) {
                super::premark_byref_arg_vars(&resolved.params, &call.args, ctx);
            }
        }

        // Always analyze arguments — even when the receiver is null/mixed and we
        // return early — so that variable reads inside args are tracked and side
        // effects (taint, etc.) are recorded.
        let mut arg_types = super::ARG_TYPES_BUF
            .with(|b| b.borrow_mut().take())
            .unwrap_or_default();
        arg_types.clear();
        for arg in call.args.iter() {
            let ty = ea.analyze(&arg.value, ctx);
            super::consume_arg_assignment(&arg.value, ctx);
            arg_types.push(if arg.unpack {
                spread_element_type(&ty)
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
            return Type::mixed();
        }

        // Purity check: calling a method on a parameter in a @pure function.
        if ctx.is_in_pure_fn {
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

        let receiver = obj_ty.remove_null();
        let mut result = Type::empty();
        // Declaring class of the resolved method, threaded out of
        // `resolve_method_return` so the symbol-recording loop below does not
        // have to walk the ancestor chain a second time. Only the
        // `TNamedObject` branch feeds it — the recording loop matches
        // top-level `TNamedObject` atomics only.
        let mut declaring = None;

        for atomic in &receiver.types {
            match atomic {
                mir_types::Atomic::TNamedObject {
                    fqcn,
                    type_params: receiver_type_params,
                } => {
                    let fqcn_resolved = crate::db::resolve_name(ea.db, &ea.file, fqcn);
                    let fqcn = &std::sync::Arc::from(fqcn_resolved.as_str());
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
                        &mut declaring,
                    ));
                    // Fallback for unresolvable calls (__call, unknown methods):
                    // key the symbol on the receiver type itself.
                    if declaring.is_none() {
                        declaring = Some(fqcn.clone());
                    }
                }
                mir_types::Atomic::TSelf { fqcn }
                | mir_types::Atomic::TStaticObject { fqcn }
                | mir_types::Atomic::TParent { fqcn } => {
                    let fqcn_resolved = crate::db::resolve_name(ea.db, &ea.file, fqcn);
                    let fqcn = &std::sync::Arc::from(fqcn_resolved.as_str());
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
                        &mut None,
                    ));
                }
                mir_types::Atomic::TIntersection { parts } => {
                    let mut intersection_result = Type::empty();
                    let mut found_method = false;
                    for part in parts.iter() {
                        for inner_atomic in &part.types {
                            if let mir_types::Atomic::TNamedObject {
                                fqcn,
                                type_params: receiver_type_params,
                            } = inner_atomic
                            {
                                let fqcn_resolved = crate::db::resolve_name(ea.db, &ea.file, fqcn);
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
                                        &mut None,
                                    ));
                                }
                            }
                        }
                    }
                    if found_method {
                        result.merge_with(&intersection_result);
                    } else {
                        result.add_type(mir_types::Atomic::TMixed);
                    }
                }
                mir_types::Atomic::TObject | mir_types::Atomic::TTemplateParam { .. } => {
                    result.add_type(mir_types::Atomic::TMixed);
                }
                mir_types::Atomic::TClosure {
                    params,
                    return_type,
                    ..
                } => {
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
                                    Some(Box::new(non_null))
                                }
                            };
                            let mut bound = Type::single(mir_types::Atomic::TClosure {
                                params: params.clone(),
                                return_type: return_type.clone(),
                                this_type,
                            });
                            bound.add_type(mir_types::Atomic::TNull);
                            result.merge_with(&bound);
                        }
                        "call" => {
                            // call($newThis, ...$args): mixed
                            // Immediately invokes the closure, returns its return_type (not nullable)
                            result.merge_with(return_type);
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
                                &mut None,
                            ));
                        }
                    }
                }
                _ => {
                    result.add_type(mir_types::Atomic::TMixed);
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
#[allow(clippy::too_many_arguments)]
fn resolve_method_return<'a>(
    ea: &mut ExpressionAnalyzer<'a>,
    ctx: &FlowState,
    call: &MethodCallExpr,
    span: Span,
    method_name: &str,
    fqcn: &Arc<str>,
    receiver_type_params: &[Type],
    arg_types: &[Type],
    arg_spans: &[Span],
    declaring_class: &mut Option<Arc<str>>,
) -> Type {
    let method_name_lower = crate::util::php_ident_lowercase(method_name);
    let resolved = resolve_method_from_db(ea, fqcn, &method_name_lower);

    if let Some(resolved) = resolved {
        if declaring_class.is_none() {
            *declaring_class = Some(resolved.owner_fqcn.clone());
        }
        ea.record_ref(
            Arc::from(format!(
                "{}::{}",
                &resolved.owner_fqcn,
                crate::util::php_ident_lowercase(&resolved.name)
            )),
            call.method.span,
        );
        // `@if-this-is X<Y>`: the method may only be called when the receiver
        // satisfies the constraint. We can only judge this when the receiver's
        // type arguments are known — an unparameterized receiver (e.g.
        // `new Foo()` with no `@var`) carries nothing to contradict.
        if let Some(constraint) = resolved.if_this_is.clone() {
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
                let receiver = Type::single(mir_types::Atomic::TNamedObject {
                    fqcn: Name::new(fqcn.as_ref()),
                    type_params: receiver_type_params.to_vec().into(),
                });
                if !crate::subtype::is_subtype(ea.db, &receiver, &constraint) {
                    ea.emit(
                        IssueKind::IfThisIsMismatch {
                            class: fqcn.to_string(),
                            method: method_name.to_string(),
                            expected: format!("{constraint}"),
                            actual: format!("{receiver}"),
                        },
                        Severity::Info,
                        span,
                    );
                }
            }
        }
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

        let arg_names: Vec<Option<String>> = call
            .args
            .iter()
            .map(|a| a.name.as_ref().map(crate::parser::name_to_string_owned))
            .collect();
        let arg_can_be_byref: Vec<bool> = call
            .args
            .iter()
            .map(|a| expr_can_be_passed_by_reference_owned(&a.value))
            .collect();
        // Build class-level template bindings before arg-checking so we can substitute
        // template params (e.g. T → int from Box<int>) into param types.
        let class_tps = crate::db::class_template_params(ea.db, fqcn)
            .map(|tps| tps.to_vec())
            .unwrap_or_default();
        let mut bindings = build_class_bindings(&class_tps, receiver_type_params);
        for (k, v) in crate::db::inherited_template_bindings(ea.db, fqcn) {
            bindings.entry(k).or_insert(v);
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
        let substituted_params: Vec<FnParam>;
        let effective_params: &[FnParam] = if param_bindings.is_empty() {
            &resolved.params
        } else {
            substituted_params = resolved
                .params
                .iter()
                .map(|p| FnParam {
                    ty: mir_codebase::wrap_param_type(
                        p.ty.as_ref()
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
                arg_types,
                arg_spans,
                arg_names: &arg_names,
                arg_can_be_byref: &arg_can_be_byref,
                call_span: span,
                has_spread: call.args.iter().any(|a| a.unpack),
                template_params: &resolved.template_params,
                no_named_arguments: resolved.no_named_arguments,
            },
        );

        // Taint sink check: emit TaintedLlmPrompt when a tainted value reaches a
        // @taint-sink annotated parameter.
        if !resolved.taint_sink_params.is_empty() {
            'sink: for (param_name, sink_kind) in &resolved.taint_sink_params {
                // Find positional index of this param in the method's param list.
                let param_idx = resolved
                    .params
                    .iter()
                    .position(|p| p.name.as_ref() == param_name.as_ref());
                let arg = if let Some(idx) = param_idx {
                    call.args.get(idx)
                } else {
                    None
                };
                // Also check named args.
                let named_arg = call.args.iter().find(|a| {
                    a.name
                        .as_ref()
                        .map(|n| crate::parser::name_to_string_owned(n) == param_name.as_ref())
                        .unwrap_or(false)
                });
                let arg = arg.or(named_arg);
                if let Some(arg) = arg {
                    if is_expr_tainted(&arg.value, ctx) {
                        let issue = match sink_kind.as_ref() {
                            "llm_prompt" => IssueKind::TaintedLlmPrompt,
                            _ => continue 'sink,
                        };
                        ea.emit(issue, Severity::Error, span);
                    }
                }
            }
        }

        let ret_raw = substitute_static_in_return(resolved.return_ty_raw, fqcn);

        if !resolved.template_params.is_empty() {
            let method_bindings =
                infer_template_bindings(&resolved.template_params, &resolved.params, arg_types);
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
            for (name, inferred, bound) in
                check_template_bounds_with_inheritance(ea.db, &bindings, &resolved.template_params)
            {
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

        // Check inter-procedural throws: if callee declares @throws, check if caller covers them.
        // Unchecked exceptions (RuntimeException / LogicException descendants) are skipped by
        // PHP convention — see [`is_unchecked_exception`].
        for callee_throw in resolved.throws.iter() {
            if crate::db::is_unchecked_exception(ea.db, callee_throw.as_ref()) {
                continue;
            }
            if !ctx.fn_declared_throws.iter().any(|declared| {
                declared.as_ref() == callee_throw.as_ref()
                    || crate::db::extends_or_implements(
                        ea.db,
                        callee_throw.as_ref(),
                        declared.as_ref(),
                    )
            }) {
                ea.emit(
                    IssueKind::MissingThrowsDocblock {
                        class: callee_throw.to_string(),
                    },
                    Severity::Info,
                    span,
                );
            }
        }

        let return_ty = if !bindings.is_empty() {
            ret_raw.substitute_templates(&bindings)
        } else {
            ret_raw
        };
        return_ty.resolve_conditional_returns(|param_name| {
            resolved
                .params
                .iter()
                .position(|p| p.name.as_ref() == param_name)
                .and_then(|idx| arg_types.get(idx))
                .cloned()
        })
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
        let guarded_by_method_exists = extract_expr_guard_key(&call.object)
            .map(|key| {
                ctx.method_exists_guards.contains(&(
                    key,
                    Arc::from(crate::util::php_ident_lowercase(method_name).as_str()),
                ))
            })
            .unwrap_or(false);
        if is_interface || is_abstract || is_trait || has_call_magic || guarded_by_method_exists {
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
