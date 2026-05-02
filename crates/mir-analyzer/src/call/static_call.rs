use std::sync::Arc;

use php_ast::ast::{ExprKind, StaticDynMethodCallExpr, StaticMethodCallExpr};
use php_ast::Span;

use mir_issues::{IssueKind, Severity};
use mir_types::Union;

use crate::context::Context;
use crate::expr::ExpressionAnalyzer;
use crate::symbol::SymbolKind;

use super::args::{
    check_args, expr_can_be_passed_by_reference, spread_element_type, substitute_static_in_return,
    CheckArgsParams,
};
use super::method::resolve_method_from_db;
use super::CallAnalyzer;

impl CallAnalyzer {
    pub fn analyze_static_method_call<'a, 'arena, 'src>(
        ea: &mut ExpressionAnalyzer<'a>,
        call: &StaticMethodCallExpr<'arena, 'src>,
        ctx: &mut Context,
        span: Span,
    ) -> Union {
        let method_name = match &call.method.kind {
            ExprKind::Identifier(name) => name.as_str(),
            _ => return Union::mixed(),
        };

        let fqcn = match &call.class.kind {
            ExprKind::Identifier(name) => {
                crate::db::resolve_name_via_db(ea.db, &ea.file, name.as_ref())
            }
            _ => return Union::mixed(),
        };

        let fqcn = resolve_static_class(&fqcn, ctx);

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

        let fqcn_arc: Arc<str> = Arc::from(fqcn.as_str());
        let method_name_lower = method_name.to_lowercase();

        let resolved = resolve_method_from_db(ea, &fqcn_arc, &method_name_lower);

        if let Some(resolved) = resolved {
            if !ea.inference_only {
                let (line, col_start, col_end) = ea.span_to_ref_loc(call.method.span);
                ea.db.record_reference_location(crate::db::RefLoc {
                    symbol_key: Arc::from(format!("{}::{}", &fqcn, method_name.to_lowercase())),
                    file: ea.file.clone(),
                    line,
                    col_start,
                    col_end,
                });
            }
            if let Some(msg) = resolved.deprecated.clone() {
                ea.emit(
                    IssueKind::DeprecatedMethodCall {
                        class: fqcn.clone(),
                        method: method_name.to_string(),
                        message: Some(msg).filter(|m| !m.is_empty()),
                    },
                    Severity::Info,
                    span,
                );
            }
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
            check_args(
                ea,
                CheckArgsParams {
                    fn_name: method_name,
                    params: &resolved.params,
                    arg_types: &arg_types,
                    arg_spans: &arg_spans,
                    arg_names: &arg_names,
                    arg_can_be_byref: &arg_can_be_byref,
                    call_span: span,
                    has_spread: call.args.iter().any(|a| a.unpack),
                },
            );
            let ret_raw = resolved.return_ty_raw;
            let ret = substitute_static_in_return(ret_raw, &fqcn_arc);
            ea.record_symbol(
                call.method.span,
                SymbolKind::StaticCall {
                    class: fqcn_arc,
                    method: Arc::from(method_name),
                },
                ret.clone(),
            );
            ret
        } else if crate::db::type_exists_via_db(ea.db, &fqcn)
            && !crate::db::has_unknown_ancestor_via_db(ea.db, &fqcn)
        {
            let (is_interface, is_abstract) = crate::db::class_kind_via_db(ea.db, &fqcn)
                .map(|k| (k.is_interface, k.is_abstract))
                .unwrap_or((false, false));
            if is_interface
                || is_abstract
                || crate::db::method_exists_via_db(ea.db, &fqcn, "__callStatic")
            {
                Union::mixed()
            } else {
                ea.emit(
                    IssueKind::UndefinedMethod {
                        class: fqcn,
                        method: method_name.to_string(),
                    },
                    Severity::Error,
                    span,
                );
                Union::mixed()
            }
        } else if !crate::db::type_exists_via_db(ea.db, &fqcn)
            && !matches!(fqcn.as_str(), "self" | "static" | "parent")
        {
            ea.emit(
                IssueKind::UndefinedClass { name: fqcn },
                Severity::Error,
                call.class.span,
            );
            Union::mixed()
        } else {
            Union::mixed()
        }
    }

    pub fn analyze_static_dyn_method_call<'a, 'arena, 'src>(
        ea: &mut ExpressionAnalyzer<'a>,
        call: &StaticDynMethodCallExpr<'arena, 'src>,
        ctx: &mut Context,
    ) -> Union {
        for arg in call.args.iter() {
            ea.analyze(&arg.value, ctx);
        }
        Union::mixed()
    }
}

fn resolve_static_class(name: &str, ctx: &Context) -> String {
    match name.to_lowercase().as_str() {
        "self" => ctx.self_fqcn.as_deref().unwrap_or("self").to_string(),
        "parent" => ctx.parent_fqcn.as_deref().unwrap_or("parent").to_string(),
        "static" => ctx
            .static_fqcn
            .as_deref()
            .unwrap_or(ctx.self_fqcn.as_deref().unwrap_or("static"))
            .to_string(),
        _ => name.to_string(),
    }
}
