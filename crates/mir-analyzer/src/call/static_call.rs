use std::sync::Arc;

use php_ast::owned::{ExprKind, StaticDynMethodCallExpr, StaticMethodCallExpr};
use php_ast::Span;

use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Union};

use crate::context::Context;
use crate::expr::ExpressionAnalyzer;
use crate::symbol::SymbolKind;

use super::args::{
    check_args, expr_can_be_passed_by_reference_owned, spread_element_type,
    substitute_static_in_return, CheckArgsParams,
};
use super::method::resolve_method_from_db;
use super::CallAnalyzer;

fn extract_namespace(fqcn: &str) -> Option<&str> {
    if let Some(pos) = fqcn.rfind('\\') {
        Some(&fqcn[..pos])
    } else {
        None
    }
}

fn is_valid_class_name_type(ty: &Union) -> bool {
    // Class names must be strings or class-string types
    ty.contains(|t| {
        matches!(
            t,
            Atomic::TString | Atomic::TClassString(_) | Atomic::TLiteralString(_)
        )
    })
}

impl CallAnalyzer {
    pub fn analyze_static_method_call<'a>(
        ea: &mut ExpressionAnalyzer<'a>,
        call: &StaticMethodCallExpr,
        ctx: &mut Context,
        span: Span,
    ) -> Union {
        let method_name = match &call.method.kind {
            ExprKind::Identifier(name) => name.as_ref(),
            _ => return Union::mixed(),
        };

        let fqcn = match &call.class.kind {
            ExprKind::Identifier(name) => {
                crate::db::resolve_name_via_db(ea.db, &ea.file, name.as_ref())
            }
            _ => {
                let ty = ea.analyze(&call.class, ctx);
                // Check if the expression could evaluate to a valid class name
                if !is_valid_class_name_type(&ty) {
                    ea.emit(
                        IssueKind::UndefinedClass {
                            name: "<dynamic>".to_string(),
                        },
                        Severity::Error,
                        call.class.span,
                    );
                }
                return Union::mixed();
            }
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

        // Check if trying to call static method on an interface (not allowed)
        if crate::db::type_exists_via_db(ea.db, &fqcn) {
            let here = crate::db::Fqcn::from_str(ea.db, fqcn_arc.as_ref());
            let is_interface = crate::db::find_class_like(ea.db, here)
                .map(|c| c.is_interface())
                .unwrap_or(false);
            if is_interface {
                ea.emit(
                    IssueKind::UndefinedClass { name: fqcn.clone() },
                    Severity::Error,
                    call.class.span,
                );
                return Union::mixed();
            }
        }

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
            if resolved.is_internal {
                let calling_namespace = ea.db.file_namespace(&ea.file).map(|ns| ns.to_string());
                let method_namespace =
                    extract_namespace(&resolved.owner_fqcn).map(|s| s.to_string());
                if calling_namespace != method_namespace {
                    ea.emit(
                        IssueKind::InternalMethod {
                            class: fqcn.clone(),
                            method: method_name.to_string(),
                        },
                        Severity::Warning,
                        span,
                    );
                }
            }
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
            let is_abstract = crate::db::class_kind_via_db(ea.db, &fqcn)
                .map(|k| k.is_abstract)
                .unwrap_or(false);
            // Check for __callStatic in the full inheritance chain (not just direct methods)
            let has_callstatic_magic = crate::db::has_method_in_chain(ea.db, &fqcn, "__callstatic");
            if is_abstract || has_callstatic_magic {
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

    pub fn analyze_static_dyn_method_call<'a>(
        ea: &mut ExpressionAnalyzer<'a>,
        call: &StaticDynMethodCallExpr,
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
