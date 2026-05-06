use std::sync::Arc;

use php_ast::ast::{ExprKind, MethodCallExpr};
use php_ast::Span;

use mir_codebase::storage::{FnParam, TemplateParam, Visibility};
use mir_issues::{IssueKind, Severity};
use mir_types::Union;

use crate::context::Context;
use crate::db::inferred_method_return_type;
use crate::expr::ExpressionAnalyzer;
use crate::generic::{build_class_bindings, check_template_bounds, infer_template_bindings};
use crate::symbol::SymbolKind;

use super::args::{
    check_args, check_method_visibility, expr_can_be_passed_by_reference, spread_element_type,
    substitute_static_in_return, CheckArgsParams,
};
use super::CallAnalyzer;

pub(super) struct ResolvedMethod {
    pub(super) owner_fqcn: Arc<str>,
    pub(super) name: Arc<str>,
    pub(super) visibility: Visibility,
    pub(super) deprecated: Option<Arc<str>>,
    pub(super) params: Vec<FnParam>,
    pub(super) template_params: Vec<TemplateParam>,
    pub(super) return_ty_raw: Union,
}

/// Resolve a method via the Salsa db, walking the class ancestor chain.
pub(super) fn resolve_method_from_db(
    ea: &ExpressionAnalyzer<'_>,
    fqcn: &Arc<str>,
    method_name_lower: &str,
) -> Option<ResolvedMethod> {
    let db = ea.db;

    // Walk own → mixins → traits → ancestors via the canonical chain helper.
    let node = crate::db::lookup_method_in_chain(db, fqcn, method_name_lower)?;
    let owner_fqcn = node.fqcn(db);
    let name = node.name(db);

    // Query the lazily-computed inferred return type via Salsa.
    // For explicit return types, the tracked query short-circuits and returns them.
    // For inferred types, it either parses the source (if available) or falls back
    // to the double-pass buffer (for synthetic/stub nodes).
    let inferred = inferred_method_return_type(db, node);
    let return_ty_raw = node
        .return_type(db)
        .map(|t| (*t).clone())
        .or_else(|| Some((*inferred).clone()))
        .unwrap_or_else(Union::mixed);

    Some(ResolvedMethod {
        owner_fqcn,
        name,
        visibility: node.visibility(db),
        deprecated: node.deprecated(db),
        params: node.params(db).to_vec(),
        template_params: node.template_params(db).to_vec(),
        return_ty_raw,
    })
}

impl CallAnalyzer {
    pub fn analyze_method_call<'a, 'arena, 'src>(
        ea: &mut ExpressionAnalyzer<'a>,
        call: &MethodCallExpr<'arena, 'src>,
        ctx: &mut Context,
        span: Span,
        nullsafe: bool,
    ) -> Union {
        let obj_ty = ea.analyze(call.object, ctx);

        let method_name = match &call.method.kind {
            ExprKind::Identifier(name) => name.as_str(),
            _ => return Union::mixed(),
        };

        // Always analyze arguments — even when the receiver is null/mixed and we
        // return early — so that variable reads inside args are tracked and side
        // effects (taint, etc.) are recorded.
        let arg_types: Vec<Union> = call
            .args
            .iter()
            .map(|arg| {
                let ty = ea.analyze(&arg.value, ctx);
                if arg.unpack {
                    spread_element_type(&ty)
                } else {
                    ty
                }
            })
            .collect();

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
                return Union::mixed();
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
            ea.emit(
                IssueKind::MixedMethodCall {
                    method: method_name.to_string(),
                },
                Severity::Info,
                span,
            );
            return Union::mixed();
        }

        let receiver = obj_ty.remove_null();
        let mut result = Union::empty();

        for atomic in &receiver.types {
            match atomic {
                mir_types::Atomic::TNamedObject {
                    fqcn,
                    type_params: receiver_type_params,
                } => {
                    let fqcn_resolved = crate::db::resolve_name_via_db(ea.db, &ea.file, fqcn);
                    let fqcn = &std::sync::Arc::from(fqcn_resolved.as_str());
                    result = Union::merge(
                        &result,
                        &resolve_method_return(
                            ea,
                            ctx,
                            call,
                            span,
                            method_name,
                            fqcn,
                            receiver_type_params.as_slice(),
                            &arg_types,
                            &arg_spans,
                        ),
                    );
                }
                mir_types::Atomic::TSelf { fqcn }
                | mir_types::Atomic::TStaticObject { fqcn }
                | mir_types::Atomic::TParent { fqcn } => {
                    let fqcn_resolved = crate::db::resolve_name_via_db(ea.db, &ea.file, fqcn);
                    let fqcn = &std::sync::Arc::from(fqcn_resolved.as_str());
                    result = Union::merge(
                        &result,
                        &resolve_method_return(
                            ea,
                            ctx,
                            call,
                            span,
                            method_name,
                            fqcn,
                            &[],
                            &arg_types,
                            &arg_spans,
                        ),
                    );
                }
                mir_types::Atomic::TIntersection { parts } => {
                    let mut intersection_result = Union::empty();
                    let mut found_method = false;
                    for part in parts {
                        for inner_atomic in &part.types {
                            if let mir_types::Atomic::TNamedObject {
                                fqcn,
                                type_params: receiver_type_params,
                            } = inner_atomic
                            {
                                let fqcn_resolved =
                                    crate::db::resolve_name_via_db(ea.db, &ea.file, fqcn);
                                let resolved_arc = Arc::from(fqcn_resolved.as_str());
                                if crate::db::method_exists_via_db(
                                    ea.db,
                                    &resolved_arc,
                                    method_name,
                                ) {
                                    found_method = true;
                                    intersection_result = Union::merge(
                                        &intersection_result,
                                        &resolve_method_return(
                                            ea,
                                            ctx,
                                            call,
                                            span,
                                            method_name,
                                            &resolved_arc,
                                            receiver_type_params.as_slice(),
                                            &arg_types,
                                            &arg_spans,
                                        ),
                                    );
                                }
                            }
                        }
                    }
                    if found_method {
                        result = Union::merge(&result, &intersection_result);
                    } else {
                        result = Union::merge(&result, &Union::mixed());
                    }
                }
                mir_types::Atomic::TObject | mir_types::Atomic::TTemplateParam { .. } => {
                    result = Union::merge(&result, &Union::mixed());
                }
                _ => {
                    result = Union::merge(&result, &Union::mixed());
                }
            }
        }

        if nullsafe && obj_ty.is_nullable() {
            result.add_type(mir_types::Atomic::TNull);
        }

        let final_ty = if result.is_empty() {
            Union::mixed()
        } else {
            result
        };

        for atomic in &obj_ty.types {
            if let mir_types::Atomic::TNamedObject { fqcn, .. } = atomic {
                ea.record_symbol(
                    call.method.span,
                    SymbolKind::MethodCall {
                        class: fqcn.clone(),
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
fn resolve_method_return<'a, 'arena, 'src>(
    ea: &mut ExpressionAnalyzer<'a>,
    ctx: &Context,
    call: &MethodCallExpr<'arena, 'src>,
    span: Span,
    method_name: &str,
    fqcn: &Arc<str>,
    receiver_type_params: &[Union],
    arg_types: &[Union],
    arg_spans: &[Span],
) -> Union {
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
            .map(|a| a.name.as_ref().map(|n| n.to_string_repr().into_owned()))
            .collect();
        let arg_can_be_byref: Vec<bool> = call
            .args
            .iter()
            .map(|a| expr_can_be_passed_by_reference(&a.value))
            .collect();
        // Build class-level template bindings before arg-checking so we can substitute
        // template params (e.g. T → int from Box<int>) into param types.
        let class_tps = crate::db::class_template_params_via_db(ea.db, fqcn)
            .map(|tps| tps.to_vec())
            .unwrap_or_default();
        let mut bindings = build_class_bindings(&class_tps, receiver_type_params);
        for (k, v) in crate::db::inherited_template_bindings_via_db(ea.db, fqcn) {
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
                check_template_bounds(&bindings, &resolved.template_params)
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

        if !bindings.is_empty() {
            ret_raw.substitute_templates(&bindings)
        } else {
            ret_raw
        }
    } else if crate::db::type_exists_via_db(ea.db, fqcn)
        && !crate::db::has_unknown_ancestor_via_db(ea.db, fqcn)
    {
        let (is_interface, is_abstract) = crate::db::class_kind_via_db(ea.db, fqcn)
            .map(|k| (k.is_interface, k.is_abstract))
            .unwrap_or((false, false));
        if is_interface || is_abstract || crate::db::method_exists_via_db(ea.db, fqcn, "__call") {
            Union::mixed()
        } else {
            ea.emit(
                IssueKind::UndefinedMethod {
                    class: fqcn.to_string(),
                    method: method_name.to_string(),
                },
                Severity::Error,
                span,
            );
            Union::mixed()
        }
    } else {
        Union::mixed()
    }
}
