//! Interprocedural return-type inference for opaque `callable` parameters.
//!
//! `array_map($cb, $arr)` / `array_reduce($arr, $cb)` normally resolve their
//! element/return type from `$cb`'s own static type
//! (`callable::callable_return_type`). When `$cb` is itself a bare,
//! unrefined `callable` parameter of the *enclosing* function, its own type
//! carries no signature to consult — the concrete closure lives at the
//! enclosing function's own call sites instead. This module closes that gap:
//! it finds every statically-resolvable concrete callable passed to a given
//! function at a given argument position, across the whole workspace, and
//! unions their return types.
//!
//! Deliberately out of scope for this pass (syntactic, not flow-sensitive):
//! method-call callees (`$obj->foo($cb)` / `Foo::bar($cb)`) — resolving them
//! would need the receiver's type, which requires the very flow analysis
//! this module is called from and must not re-enter. Only plain function
//! calls are indexed. Closures/arrow functions are only recognized when they
//! carry an explicit native return-type hint — inferring an unannotated
//! closure's return type would require running full body analysis on it in
//! isolation, a materially bigger feature reserved for a future pass.
//! Named/spread arguments are skipped (position tracking only covers plain
//! positional args) — a reasonable V1 simplification since the vast
//! majority of `array_map`-style callback params are positional.

use crate::ast_ext::ArgExt as _;
use crate::ast_ext::ByteStringExt as _;
use crate::ast_ext::{FunctionRef, FunctionRefExt as _};
use std::ops::ControlFlow;
use std::sync::Arc;

use mir_types::Type;
use php_ast::owned::visitor::OwnedVisitor;
use php_ast::owned::{CallableCreateKind, Expr, ExprKind};

use crate::db::{MirDatabase, SourceFile};
use crate::parser::type_from_hint_owned;

/// Identity of a call's callee. V1 only records plain function calls — see
/// module docs for why method calls are out of scope.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum CalleeKey {
    Function(Arc<str>),
}

/// One statically-resolvable concrete callable passed to `callee` at
/// `arg_position` in some call site.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CallArgReturn {
    pub callee: CalleeKey,
    pub arg_position: u16,
    pub return_type: Type,
}

/// Attempt to resolve `expr` (an argument expression at a call site) to a
/// concrete callable's return type, without any flow-sensitive analysis.
/// Returns `None` for anything not statically resolvable this way (a bare
/// variable, property access, a method-call result, …) — the call site
/// simply contributes no fact, not an error.
fn resolve_concrete_callback_return(db: &dyn MirDatabase, file: &str, expr: &Expr) -> Option<Type> {
    match &expr.kind {
        ExprKind::Closure(c) => {
            let hint = c.return_type.as_ref()?;
            let ty = type_from_hint_owned(hint, None);
            Some(crate::stmt::resolve_union_for_file_native(ty, db, file))
        }
        ExprKind::ArrowFunction(a) => {
            let hint = a.return_type.as_ref()?;
            let ty = type_from_hint_owned(hint, None);
            Some(crate::stmt::resolve_union_for_file_native(ty, db, file))
        }
        ExprKind::CallableCreate(cc) => {
            let CallableCreateKind::Function(target) = &cc.kind else {
                return None;
            };
            let (_, f) = target.kind.function_ref(db, file)?.find(db, file)?;
            f.return_type.as_deref().cloned()
        }
        ExprKind::String(s) if !s.is_empty() => {
            let (_, f) = FunctionRef::from_text(s.as_str()).find(db, file)?;
            f.return_type.as_deref().cloned()
        }
        _ => None,
    }
}

struct OpaqueCallScanner<'a> {
    db: &'a dyn MirDatabase,
    file: &'a str,
    out: Vec<CallArgReturn>,
}

impl OwnedVisitor for OpaqueCallScanner<'_> {
    type Break = ();

    fn visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        if let ExprKind::FunctionCall(call) = &expr.kind {
            if let Some(name) = call.name.kind.function_ref(self.db, self.file) {
                let callee = CalleeKey::Function(Arc::from(name.resolve(self.db, self.file)));
                let mut position = 0u16;
                for arg in call.args.iter() {
                    if arg.name.is_none() && !arg.unpack() {
                        if let Some(return_type) = arg
                            .value()
                            .and_then(|v| resolve_concrete_callback_return(self.db, self.file, v))
                        {
                            self.out.push(CallArgReturn {
                                callee: callee.clone(),
                                arg_position: position,
                                return_type,
                            });
                        }
                        position += 1;
                    }
                }
            }
        }
        crate::recursion::walk_expr(self, expr)
    }
}

/// Per-file syntactic scan of every plain function call, recording a fact
/// for each argument that is a statically-resolvable concrete callable.
/// Tracked so salsa invalidates it exactly when this file's text changes —
/// same shape as `collect_file_definitions`, independently re-parsing rather
/// than sharing a parse cache (matching the rest of this crate's per-query
/// parsing convention).
#[salsa::tracked]
pub(crate) fn file_callable_call_args(
    db: &dyn MirDatabase,
    file: SourceFile,
) -> Arc<[CallArgReturn]> {
    let path = file.path(db);
    let text = file.text(db);
    let parsed = php_rs_parser::parse(text);
    let mut scanner = OpaqueCallScanner {
        db,
        file: path.as_ref(),
        out: Vec::new(),
    };
    let _ = scanner.visit_program(&parsed.program);
    Arc::from(scanner.out)
}

/// Union the return types of every statically-resolvable concrete callable
/// passed to `callee` at `arg_position` across the whole workspace. Returns
/// `None` when no caller passes a resolvable callable there (no callers at
/// all, or every caller's argument is itself opaque) — callers treat this
/// exactly like "no callback return type known," falling back to the
/// generic stub return.
pub(crate) fn opaque_callback_return_type(
    db: &dyn MirDatabase,
    callee: &CalleeKey,
    arg_position: u16,
) -> Option<Type> {
    let mut acc: Option<Type> = None;
    for file in db.all_source_files() {
        for rec in file_callable_call_args(db, file).iter() {
            if &rec.callee == callee && rec.arg_position == arg_position {
                match &mut acc {
                    None => acc = Some(rec.return_type.clone()),
                    Some(t) => t.merge_with(&rec.return_type),
                }
            }
        }
    }
    acc
}
