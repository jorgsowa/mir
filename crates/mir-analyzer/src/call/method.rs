use std::sync::Arc;

use php_ast::owned::{ExprKind, MethodCallExpr};
use php_ast::Span;

use mir_codebase::storage::{FnParam, TemplateParam, Visibility};
use mir_issues::{IssueKind, Severity};
use mir_types::Type;

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

pub(super) struct ResolvedMethod {
    pub(super) owner_fqcn: Arc<str>,
    pub(super) name: Arc<str>,
    pub(super) visibility: Visibility,
    pub(super) deprecated: Option<Arc<str>>,
    pub(super) is_internal: bool,
    pub(super) params: Vec<FnParam>,
    pub(super) template_params: Vec<TemplateParam>,
    pub(super) return_ty_raw: Type,
    pub(super) throws: Arc<[Arc<str>]>,
}

/// Resolve a method via the Salsa db, walking the class ancestor chain.
pub(super) fn resolve_method_from_db(
    ea: &ExpressionAnalyzer<'_>,
    fqcn: &Arc<str>,
    method_name_lower: &str,
) -> Option<ResolvedMethod> {
    let db = ea.db;

    if let Some((owner_fqcn, storage)) = crate::db::find_method_in_chain(
        db,
        crate::db::Fqcn::from_str(db, fqcn.as_ref()),
        method_name_lower,
    ) {
        let name = storage.name.clone();
        let name_lower = if name.chars().all(|c| !c.is_uppercase()) {
            name.clone()
        } else {
            Arc::<str>::from(name.to_ascii_lowercase().as_str())
        };
        let inferred = crate::db::inferred_method_return_type_demand(db, &owner_fqcn, &name_lower);
        let return_ty_raw = storage
            .return_type
            .clone()
            .or(inferred)
            .map(|t| (*t).clone())
            .unwrap_or_else(Type::mixed);

        return Some(ResolvedMethod {
            owner_fqcn,
            name,
            visibility: storage.visibility,
            deprecated: storage.deprecated.clone(),
            is_internal: storage.is_internal,
            params: storage.params.to_vec(),
            template_params: storage.template_params.clone(),
            return_ty_raw,
            throws: storage.throws.clone().into(),
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
            _ => return Type::mixed(),
        };

        // Always analyze arguments — even when the receiver is null/mixed and we
        // return early — so that variable reads inside args are tracked and side
        // effects (taint, etc.) are recorded.
        let mut arg_types = super::ARG_TYPES_BUF
            .with(|b| b.borrow_mut().take())
            .unwrap_or_default();
        arg_types.clear();
        for arg in call.args.iter() {
            let ty = ea.analyze(&arg.value, ctx);
            arg_types.push(if arg.unpack {
                spread_element_type(&ty)
            } else {
                ty
            });
        }

        let arg_spans: Vec<Span> = call.args.iter().map(|a| a.span).collect();

        if obj_ty.contains(|t| matches!(t, mir_types::Atomic::TNull)) {
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

        let receiver = obj_ty.remove_null();
        let mut result = Type::empty();

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
                    ));
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
            if let mir_types::Atomic::TNamedObject { fqcn, .. } = atomic {
                ea.record_symbol(
                    call.method.span,
                    ReferenceKind::MethodCall {
                        class: Arc::from(fqcn.as_ref()),
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
) -> Type {
    let method_name_lower = method_name.to_lowercase();
    let resolved = resolve_method_from_db(ea, fqcn, &method_name_lower);

    if let Some(resolved) = resolved {
        if !ea.inference_only {
            let (line, col_start, col_end) = ea.span_to_ref_loc(call.method.span);
            ea.db.record_reference_location(crate::db::RefLoc {
                symbol_key: Arc::from(format!(
                    "{}::{}",
                    &resolved.owner_fqcn,
                    resolved.name.to_lowercase()
                )),
                file: ea.file.clone(),
                line,
                col_start,
                col_end,
            });
        }
        if let Some(msg) = resolved.deprecated.clone() {
            ea.emit(
                IssueKind::DeprecatedMethodCall {
                    class: fqcn.to_string(),
                    method: method_name.to_string(),
                    message: Some(msg).filter(|m| !m.is_empty()),
                },
                Severity::Info,
                span,
            );
        }
        if resolved.is_internal {
            let calling_namespace = ea.db.file_namespace(&ea.file).map(|ns| ns.to_string());
            let method_namespace = extract_namespace(&resolved.owner_fqcn).map(|s| s.to_string());
            if calling_namespace != method_namespace {
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
        let substituted_params: Vec<FnParam>;
        let effective_params: &[FnParam] = if bindings.is_empty() {
            &resolved.params
        } else {
            substituted_params = resolved
                .params
                .iter()
                .map(|p| FnParam {
                    ty: mir_codebase::wrap_param_type(
                        p.ty.as_ref().map(|t| t.substitute_templates(&bindings)),
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
            },
        );

        let ret_raw = substitute_static_in_return(resolved.return_ty_raw, fqcn);

        if !resolved.template_params.is_empty() {
            let method_bindings =
                infer_template_bindings(&resolved.template_params, &resolved.params, arg_types);
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

        if !bindings.is_empty() {
            ret_raw.substitute_templates(&bindings)
        } else {
            ret_raw
        }
    } else if crate::db::class_exists(ea.db, fqcn) && !crate::db::has_unknown_ancestor(ea.db, fqcn)
    {
        let (is_interface, is_abstract) = crate::db::class_kind(ea.db, fqcn)
            .map(|k| (k.is_interface, k.is_abstract))
            .unwrap_or((false, false));
        // Check for __call in the full inheritance chain (not just direct methods)
        let has_call_magic = crate::db::has_method_in_chain(ea.db, fqcn, "__call");
        if is_interface || is_abstract || has_call_magic {
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
