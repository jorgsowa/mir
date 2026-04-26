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
            ExprKind::Identifier(name) => ea.codebase.resolve_class_name(&ea.file, name.as_ref()),
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

        if let Some(method) = ea.codebase.get_method(&fqcn, method_name) {
            let method_span = call.method.span;
            ea.codebase.mark_method_referenced_at(
                &fqcn,
                method_name,
                ea.file.clone(),
                method_span.start,
                method_span.end,
            );
            if let Some(msg) = method.deprecated.clone() {
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
                    params: &method.params,
                    arg_types: &arg_types,
                    arg_spans: &arg_spans,
                    arg_names: &arg_names,
                    arg_can_be_byref: &arg_can_be_byref,
                    call_span: span,
                    has_spread: call.args.iter().any(|a| a.unpack),
                },
            );
            let ret_raw = method
                .effective_return_type()
                .cloned()
                .unwrap_or_else(Union::mixed);
            let fqcn_arc: Arc<str> = Arc::from(fqcn.as_str());
            let ret = substitute_static_in_return(ret_raw, &fqcn_arc);
            ea.record_symbol(
                method_span,
                SymbolKind::StaticCall {
                    class: fqcn_arc,
                    method: Arc::from(method_name),
                },
                ret.clone(),
            );
            ret
        } else if ea.codebase.type_exists(&fqcn) && !ea.codebase.has_unknown_ancestor(&fqcn) {
            let is_interface = ea.codebase.interfaces.contains_key(fqcn.as_str());
            let is_abstract = ea.codebase.is_abstract_class(&fqcn);
            if is_interface
                || is_abstract
                || ea.codebase.get_method(&fqcn, "__callStatic").is_some()
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
        } else if !ea.codebase.type_exists(&fqcn)
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
