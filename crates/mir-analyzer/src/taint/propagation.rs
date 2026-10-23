use super::*;
use crate::ast_ext::ArgExt as _;
use crate::ast_ext::ByteStringExt as _;
use crate::ast_ext::FunctionRefExt as _;
use crate::ast_ext::NameBareExt as _;
use crate::ast_ext::NameRefExt as _;
use crate::ast_ext::NameResolveExt as _;

/// Union of the kinds named by taint annotations (`html`, `sql`, `input`, ...).
pub(super) fn named_kinds(names: &[std::sync::Arc<str>]) -> Taint {
    names.iter().fold(Taint::NONE, |acc, name| {
        acc | match name.as_ref() {
            "input" | "text" => Taint::ALL,
            "has_quotes" => Taint::HTML | Taint::SQL,
            other => taint_sink_kind(other),
        }
    })
}

/// A return type whose values can't carry a payload into a sink.
pub(super) fn return_drops_taint(ty: &mir_types::Type) -> bool {
    use mir_types::Atomic;
    ty.is_void()
        || ty.is_never()
        || is_injection_safe_type(ty)
        || ty.types.iter().all(|a| match a {
            Atomic::TNamedObject { fqcn, .. } => !is_iterator_class(fqcn.as_ref()),
            _ => matches!(
                a,
                Atomic::TStaticObject { .. }
                    | Atomic::TSelf { .. }
                    | Atomic::TParent { .. }
                    | Atomic::TClosure { .. }
                    | Atomic::TObject
            ),
        })
}

/// Builtin classes whose instances hand out the arrays and iterators they are built from.
const ITERATOR_WRAPPERS: &[&str] = &[
    "ArrayObject",
    "ArrayIterator",
    "RecursiveArrayIterator",
    "IteratorIterator",
    "LimitIterator",
    "InfiniteIterator",
    "NoRewindIterator",
    "CachingIterator",
    "CallbackFilterIterator",
    "RecursiveCallbackFilterIterator",
    "RecursiveIteratorIterator",
    "RegexIterator",
    "ParentIterator",
];

/// A class whose instances hand out the values they were built from.
fn is_iterator_class(fqcn: &str) -> bool {
    let name = fqcn.trim_start_matches('\\');
    ["Generator", "Iterator", "IteratorAggregate", "Traversable"]
        .iter()
        .chain(ITERATOR_WRAPPERS)
        .any(|class| name.eq_ignore_ascii_case(class))
}

/// Taint of `new` on a builtin iterator wrapper: the data it is built from.
fn iterator_wrapper_taint(
    new: &php_ast::owned::NewExpr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    let Some(name) = new.class.kind.name_ref() else {
        return Taint::NONE;
    };
    if new.args.is_empty() {
        return Taint::NONE;
    }
    let class = match crate::db::resolve_name(db, file, name.as_ref()).as_str() {
        "self" | "static" => ctx.self_fqcn.clone(),
        "parent" => ctx.parent_fqcn.clone(),
        resolved => Some(std::sync::Arc::from(resolved)),
    };
    let built_in = class.is_some_and(|class| {
        crate::db::find_method_in_chain(db, crate::db::Fqcn::from_str(db, &class), "__construct")
            .is_some_and(|(owner, _)| {
                ITERATOR_WRAPPERS
                    .iter()
                    .any(|wrapper| owner.trim_start_matches('\\').eq_ignore_ascii_case(wrapper))
            })
    });
    if built_in {
        args_taint(&new.args, ctx, db, file)
    } else {
        Taint::NONE
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CalleeBody {
    /// Abstract, interface or stub: only annotations describe it.
    None,
    Dependency,
    Summarized,
}

impl CalleeBody {
    pub(super) fn of(location: Option<&mir_types::Location>, from: &str) -> CalleeBody {
        match location {
            Some(loc) if crate::stubs::is_stub_path(loc.file.as_ref()) => CalleeBody::None,
            Some(loc) if is_dependency_of(loc.file.as_ref(), from) => CalleeBody::Dependency,
            Some(_) => CalleeBody::Summarized,
            None => CalleeBody::None,
        }
    }
}

/// Taint the object `receiver` evaluates to holds in property `prop`.
pub(super) fn receiver_prop_taint(
    receiver: Option<&Expr>,
    prop: mir_types::Name,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    match receiver.map(|r| &unwrap_parens(r).kind) {
        Some(ExprKind::Variable(var)) => ctx.prop_taint(var.trim_start_matches('$'), prop.as_str()),
        Some(ExprKind::New(new)) => constructed_prop_taint(new, prop.as_str(), ctx, db, file),
        _ => Taint::NONE,
    }
}

/// Taint an inline `new` leaves in `prop` of the object it creates.
fn constructed_prop_taint(
    new: &php_ast::owned::NewExpr,
    prop: &str,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    constructed_prop_taints(new, ctx, db, file)
        .into_iter()
        .filter(|(name, _)| name.as_str() == prop)
        .fold(Taint::NONE, |acc, (_, taint)| acc | taint)
}

/// Whether the object `receiver` evaluates to holds source data worth a callee
/// summary; while summarizing, `$this` does too, as its property labels stand
/// for data its callees may sink or return.
pub(super) fn receiver_demands_summary(
    receiver: Option<&Expr>,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> bool {
    match receiver.map(|r| &unwrap_parens(r).kind) {
        Some(ExprKind::Variable(var)) => {
            let var = mir_types::Name::from(var.trim_start_matches('$'));
            (ctx.summarizing_taint && var.as_str() == "this")
                || ctx
                    .tainted_props
                    .iter()
                    .any(|((object, _), taint)| *object == var && !taint.concrete().is_empty())
        }
        Some(ExprKind::New(new)) => constructed_prop_taints(new, ctx, db, file)
            .iter()
            .any(|(_, taint)| !taint.concrete().is_empty()),
        _ => false,
    }
}

/// The part of an argument's taint worth inferring a callee summary for:
/// parameter labels only matter while a summary is being built.
pub(super) fn demanding_taint(taint: Taint, ctx: &FlowState) -> Taint {
    if ctx.summarizing_taint {
        taint
    } else {
        taint.concrete()
    }
}

/// Taint of a call to user code, from the callee's annotations and inferred summary.
#[allow(clippy::too_many_arguments)]
pub(super) fn user_call_taint(
    params: &[mir_codebase::DeclaredParam],
    args: &[php_ast::owned::Arg],
    return_type: Option<&mir_types::Type>,
    annotations: Option<&mir_codebase::TaintAnnotations>,
    body: CalleeBody,
    receiver: Option<&Expr>,
    returns_param: impl FnOnce() -> bool,
    returns_prop: impl FnOnce() -> bool,
    reads_source: impl FnOnce() -> bool,
    summary: impl FnOnce() -> Option<TaintSummary>,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    if return_type.is_some_and(return_drops_taint) {
        return Taint::NONE;
    }
    let arg_taint = |index| param_arg_taint(params, args, index, ctx, db, file);
    // Inferring a summary only pays off when it can return source data, tainted
    // arguments or the receiver's tainted properties.
    let worth_summary = || {
        (returns_param() && !demanding_taint(args_value_taint(args, ctx, db, file), ctx).is_empty())
            || (receiver_demands_summary(receiver, ctx, db, file) && returns_prop())
            || reads_source()
    };
    let mut taint = match body {
        CalleeBody::Summarized if worth_summary() => summary().map_or(Taint::NONE, |s| {
            resolve_labels(s.returns, |slot| {
                s.slot_taint(slot, receiver, params, args, ctx, db, file)
            })
        }),
        CalleeBody::Dependency => args_value_taint(args, ctx, db, file),
        _ => Taint::NONE,
    };
    let Some(annotations) = annotations else {
        return taint;
    };
    for name in &annotations.flow_params {
        if let Some(index) = params.iter().position(|p| p.name.as_ref() == name.as_ref()) {
            taint |= arg_taint(index);
        }
    }
    let escaped = named_kinds(&annotations.escape);
    let unescaped = named_kinds(&annotations.unescape);
    // An annotated escaper or decoder without a body to summarize passes its arguments through.
    if body == CalleeBody::None && annotations.flow_params.is_empty() {
        taint = args_taint(args, ctx, db, file);
    }
    taint.without(escaped).restore(unescaped)
}

/// Taint a user callee writes into by-ref parameter `param`, beyond what its arguments carry.
#[allow(clippy::too_many_arguments)]
pub fn byref_output_taint(
    param: usize,
    location: impl FnOnce() -> Option<mir_types::Location>,
    params: &[mir_codebase::DeclaredParam],
    args: &[php_ast::owned::Arg],
    receiver: Option<&Expr>,
    reads_source: impl FnOnce() -> bool,
    summary: impl FnOnce() -> Option<TaintSummary>,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    if !db.taint_analysis()
        || CalleeBody::of(location().as_ref(), file) != CalleeBody::Summarized
        || !(reads_source() || receiver_demands_summary(receiver, ctx, db, file))
    {
        return Taint::NONE;
    }
    let Some(summary) = summary() else {
        return Taint::NONE;
    };
    resolve_labels(summary.out_param(param), |slot| {
        summary.slot_taint(slot, receiver, params, args, ctx, db, file)
    })
}

/// Strip any number of redundant parens (`((expr))` -> `expr`) — a call
/// target keeps its parens in the AST (`(fn() => ...)()`), unlike most
/// other expression positions.
pub fn unwrap_parens(expr: &Expr) -> &Expr {
    let mut e = expr;
    while let ExprKind::Parenthesized(inner) = &e.kind {
        e = inner;
    }
    e
}

/// Union of the taint every argument of a call carries.
pub fn args_taint(
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    args.iter()
        .filter_map(|a| a.value())
        .fold(Taint::NONE, |acc, v| acc | expr_taint(v, ctx, db, file))
}

/// Taint of the key names a builtin copies out of its array argument into its result.
fn builtin_key_names_taint(
    lower_name: &str,
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
) -> Taint {
    if ctx.key_names.is_empty() {
        return Taint::NONE;
    }
    match lower_name {
        "array_keys" | "array_key_first" | "array_key_last" | "key" | "array_flip"
        | "json_encode" | "http_build_query" | "serialize" | "print_r" | "var_export" => args
            .first()
            .and_then(|a| a.value())
            .map_or(Taint::NONE, |array| key_names_taint(array, ctx)),
        _ => Taint::NONE,
    }
}

/// Taint of the key names of the array `expr` evaluates to.
pub fn key_names_taint(expr: &Expr, ctx: &FlowState) -> Taint {
    match &unwrap_parens(expr).kind {
        ExprKind::Variable(var) if !ctx.key_names.is_empty() => ctx.key_names_taint(var),
        _ => Taint::NONE,
    }
}

/// Taint of an argument or element: the value's own, plus what the object it holds keeps in properties.
pub fn value_taint(
    expr: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    expr_taint(expr, ctx, db, file) | held_prop_taint(expr, ctx, db, file)
}

/// Taint the object `expr` evaluates to keeps in its properties.
pub fn held_prop_taint(
    expr: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    match &unwrap_parens(expr).kind {
        ExprKind::Variable(var) if !ctx.tainted_props.is_empty() => {
            let var = mir_types::Name::from(var.trim_start_matches('$'));
            ctx.tainted_props
                .iter()
                .filter(|((object, _), _)| *object == var)
                .fold(Taint::NONE, |acc, (_, taint)| acc | *taint)
        }
        ExprKind::New(new) => constructed_prop_taints(new, ctx, db, file)
            .into_iter()
            .fold(Taint::NONE, |acc, (_, taint)| acc | taint),
        ExprKind::Ternary(t) => {
            held_prop_taint(
                t.then_expr.as_deref().unwrap_or(&t.condition),
                ctx,
                db,
                file,
            ) | held_prop_taint(&t.else_expr, ctx, db, file)
        }
        ExprKind::NullCoalesce(nc) => {
            held_prop_taint(&nc.left, ctx, db, file) | held_prop_taint(&nc.right, ctx, db, file)
        }
        _ => Taint::NONE,
    }
}

/// Union of the taint every argument of a call passes on, objects' properties included.
pub fn args_value_taint(
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    args.iter()
        .filter_map(|a| a.value())
        .fold(Taint::NONE, |acc, v| acc | value_taint(v, ctx, db, file))
}

/// Argument positions whose data a builtin's result is built from; `None` when any argument may contribute.
fn carrying_positions(lower_name: &str) -> Option<&'static [usize]> {
    Some(match lower_name {
        "get_debug_type" | "gettype" | "range" | "random_bytes" => &[],
        // Only the first array contributes values; the rest are compared by key.
        "array_intersect_key"
        | "array_diff_key"
        | "array_intersect_ukey"
        | "array_diff_ukey"
        | "substr"
        | "mb_substr"
        | "iconv_substr"
        | "mb_strcut"
        | "str_repeat"
        | "str_split"
        | "mb_str_split"
        | "trim"
        | "ltrim"
        | "rtrim"
        | "chop"
        | "nl2br"
        | "ucwords"
        | "mb_convert_case"
        | "array_slice"
        | "array_chunk"
        | "array_unique"
        | "array_reverse"
        | "array_column"
        | "array_keys"
        | "strstr"
        | "stristr"
        | "strrchr"
        | "preg_quote"
        | "json_decode"
        | "hex2bin"
        | "base_convert"
        | "iterator_to_array"
        | "date"
        | "gmdate"
        | "strftime"
        | "gmstrftime" => &[0],
        "explode" | "array_search" | "preg_split" => &[1],
        "str_pad" | "wordwrap" | "chunk_split" | "array_pad" => &[0, 2],
        "substr_replace" => &[0, 1],
        // The search term or pattern is matched against the subject, never copied into the result.
        "str_replace" | "str_ireplace" | "preg_replace" => &[1, 2],
        "mb_strimwidth" => &[0, 3],
        "array_fill" => &[2],
        _ => return None,
    })
}

/// Taint of a builtin's result: the arguments its value is built from, not the offsets, counts and formats.
fn builtin_args_taint(
    lower_name: &str,
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    let positional = args.iter().all(|a| !a.unpack() && a.name.is_none());
    // Without a format argument it returns a word count, not the words.
    if lower_name == "str_word_count" && positional && args.len() == 1 {
        return Taint::NONE;
    }
    match carrying_positions(lower_name) {
        Some(positions) if positional => positions
            .iter()
            .filter_map(|&i| args.get(i)?.value())
            .fold(Taint::NONE, |acc, v| {
                acc | stringified_taint(v, ctx, db, file)
            }),
        _ => args
            .iter()
            .filter_map(|a| a.value())
            .fold(Taint::NONE, |acc, v| {
                acc | stringified_taint(v, ctx, db, file)
            }),
    }
}

/// Taint of `expr` converted to a string: an object also yields what its `__toString()` returns.
pub fn stringified_taint(
    expr: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    expr_taint(expr, ctx, db, file) | to_string_taint(expr, ctx, db, file)
}

fn to_string_taint(
    expr: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    if !db.taint_analysis() {
        return Taint::NONE;
    }
    let receiver = unwrap_parens(expr);
    let classes: Vec<String> = match &receiver.kind {
        ExprKind::Ternary(t) => {
            let then_expr = t.then_expr.as_deref().unwrap_or(&t.condition);
            return to_string_taint(then_expr, ctx, db, file)
                | to_string_taint(&t.else_expr, ctx, db, file);
        }
        ExprKind::NullCoalesce(nc) => {
            return to_string_taint(&nc.left, ctx, db, file)
                | to_string_taint(&nc.right, ctx, db, file);
        }
        ExprKind::New(_) => produced_object_classes(receiver, db, file),
        ExprKind::Variable(var) => ctx
            .vars
            .get(&mir_types::Name::from(var.trim_start_matches('$')))
            .map(|ty| {
                ty.types
                    .iter()
                    .filter_map(|atom| match atom {
                        mir_types::Atomic::TNamedObject { fqcn, .. } => Some(fqcn.to_string()),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default(),
        _ => return Taint::NONE,
    };
    classes
        .iter()
        .filter(|class| !class.is_empty())
        .filter_map(|class| {
            method_call_taint(
                crate::db::Fqcn::from_str(db, class),
                "__tostring",
                &[],
                Some(receiver),
                false,
                ctx,
                db,
                file,
            )
        })
        .fold(Taint::NONE, |acc, taint| acc | taint)
}

/// The kinds of attacker-controlled data the expression could carry, given the
/// current `FlowState` taint state.
///
/// This is a conservative over-approximation:
/// - Any reference to a superglobal (or an array offset thereof) is tainted.
/// - Any variable that was previously marked tainted in `ctx.tainted_vars` is tainted.
/// - Binary string-concat or arithmetic on tainted operands propagates taint.
/// - Interpolated strings are tainted if any embedded variable is tainted.
/// - A call to a `@taint-source`-annotated function/method is tainted.
/// - A sanitizer (`htmlspecialchars`, `escapeshellarg`, …) removes only its own kinds.
pub fn expr_taint(
    expr: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    if !db.taint_analysis() {
        return Taint::NONE;
    }
    match &expr.kind {
        ExprKind::Variable(name) => {
            let n = name.trim_start_matches('$');
            if is_superglobal(n) {
                return Taint::ALL;
            }
            let sym = mir_types::Name::from(n);
            if ctx
                .vars
                .get(&sym)
                .is_some_and(|ty| is_injection_safe_type(ty))
            {
                return Taint::NONE;
            }
            // A variable with no tracked definition may be one `extract()`
            // defined from a tainted array; its name isn't known statically.
            if ctx.has_dynamic_tainted_var_def && !ctx.var_is_defined_sym(sym) {
                return Taint::ALL;
            }
            ctx.var_taint(n)
        }

        // `$$name` where `$name` holds a known literal string resolves to
        // that variable's own taint state.
        ExprKind::VariableVariable(inner) => {
            let Some(var_name) = crate::expr::helpers::extract_simple_var(inner) else {
                return Taint::NONE;
            };
            ctx.get_var(&var_name)
                .types
                .iter()
                .fold(Taint::NONE, |acc, atomic| match atomic {
                    mir_types::Atomic::TLiteralString(accessed) => {
                        acc | ctx.var_taint(accessed.as_ref())
                    }
                    _ => acc,
                })
        }

        ExprKind::ArrayAccess(aa) => {
            if is_trusted_superglobal_field(aa) {
                Taint::NONE
            } else if let Some(key) = server_dynamic_key(aa) {
                server_dynamic_read_taint(key, ctx, db, file)
            } else if is_globals_superglobal_alias(aa) {
                Taint::ALL
            } else if let Some(var) = globals_entry(aa) {
                crate::db::global_var_taint(db, file, var)
            } else if let (ExprKind::Variable(name), Some(key)) = (
                &aa.array.kind,
                crate::expr::assignment::literal_key_name(aa.index.as_deref()),
            ) {
                let n = name.trim_start_matches('$');
                if ctx
                    .vars
                    .get(&mir_types::Name::from(n))
                    .is_some_and(|ty| shape_entry_is_safe(ty, key))
                {
                    Taint::NONE
                } else if is_superglobal(n) {
                    ctx.key_taints(n).map_or(Taint::ALL, |keys| keys.get(key))
                } else {
                    ctx.key_taint(n, key)
                }
            } else {
                expr_taint(&aa.array, ctx, db, file)
            }
        }

        // The error-suppression operator only silences a notice, it doesn't
        // sanitize the value.
        ExprKind::Parenthesized(inner) | ExprKind::ErrorSuppress(inner) => {
            expr_taint(inner, ctx, db, file)
        }

        // Only a simple-variable receiver is tracked, matching prop_refined's
        // narrowing scope.
        ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa) => {
            let Some(prop_name) = crate::expr::helpers::extract_string_from_expr(&pa.property)
            else {
                return expr_taint(&pa.object, ctx, db, file);
            };
            if let Some(recv_ty) =
                crate::expr::assignment::resolve_chained_receiver_type(&pa.object, ctx, db, file)
            {
                let is_bag = recv_ty.types.iter().any(|atom| {
                    matches!(atom, mir_types::Atomic::TNamedObject { fqcn, .. }
                        if is_request_bag_property(db, fqcn.as_ref(), &prop_name))
                });
                if is_bag {
                    return Taint::ALL;
                }
            }
            // A property of a tainted object (`json_decode($_GET['j'])->name`) is tainted too.
            let own = match &unwrap_parens(&pa.object).kind {
                ExprKind::Variable(obj_var) => {
                    ctx.prop_taint(obj_var.trim_start_matches('$'), &prop_name)
                }
                ExprKind::New(new) => constructed_prop_taint(new, &prop_name, ctx, db, file),
                _ => Taint::NONE,
            };
            let classes = receiver_classes(&pa.object, ctx, db, file);
            let declared_safe = !classes.is_empty()
                && classes.iter().all(|class| {
                    crate::db::find_property_in_chain(
                        db,
                        crate::db::Fqcn::from_str(db, class.as_ref()),
                        &prop_name,
                    )
                    .is_some_and(|(_, def)| def.ty.as_deref().is_some_and(is_injection_safe_type))
                });
            if declared_safe {
                return Taint::NONE;
            }
            let key = crate::db::taint_sources::prop_key(&prop_name, false);
            let stored = classes.iter().fold(Taint::NONE, |acc, class| {
                acc | class_property_taint(db, file, class, key)
            });
            own | stored | expr_taint(&pa.object, ctx, db, file)
        }

        ExprKind::Assign(a) => expr_taint(&a.value, ctx, db, file),

        ExprKind::Binary(op) if op.op == php_ast::ast::BinaryOp::Pipe => {
            expr_taint(&crate::expr::pipe_as_call(op, expr.span), ctx, db, file)
        }

        ExprKind::Binary(op) if yields_number_or_bool(op.op, &op.left, &op.right, ctx) => {
            Taint::NONE
        }

        ExprKind::Binary(op) => {
            let concat = op.op == php_ast::ast::BinaryOp::Concat;
            let operand = |e: &Expr| {
                if concat {
                    stringified_taint(e, ctx, db, file)
                } else {
                    expr_taint(e, ctx, db, file)
                }
            };
            let taint = operand(&op.left) | operand(&op.right);
            if concat && starts_with_fixed_url_origin(&op.left) {
                taint.without(Taint::SSRF | Taint::FILE)
            } else {
                taint
            }
        }

        ExprKind::UnaryPrefix(u) => match u.op {
            php_ast::ast::UnaryPrefixOp::Negate
            | php_ast::ast::UnaryPrefixOp::Plus
            | php_ast::ast::UnaryPrefixOp::BooleanNot => Taint::NONE,
            _ => expr_taint(&u.operand, ctx, db, file),
        },

        ExprKind::InterpolatedString(parts)
        | ExprKind::Heredoc { parts, .. }
        | ExprKind::ShellExec(parts) => {
            let taint = parts.iter().fold(Taint::NONE, |acc, p| match p {
                StringPart::Expr(e) => acc | stringified_taint(e, ctx, db, file),
                StringPart::Literal(_) => acc,
                _ => acc,
            });
            if !matches!(expr.kind, ExprKind::ShellExec(_)) && starts_with_fixed_url_origin(expr) {
                taint.without(Taint::SSRF | Taint::FILE)
            } else {
                taint
            }
        }

        // Short ternary (`$x ?: $y`): the true branch's value is the condition itself.
        ExprKind::Ternary(t) => {
            let then_taint = match &t.then_expr {
                Some(then_e) => branch_taint(&t.condition, true, then_e, ctx, db, file),
                None => expr_taint(&t.condition, ctx, db, file),
            };
            then_taint | branch_taint(&t.condition, false, &t.else_expr, ctx, db, file)
        }

        ExprKind::NullCoalesce(nc) => {
            expr_taint(&nc.left, ctx, db, file) | expr_taint(&nc.right, ctx, db, file)
        }

        // Numeric/boolean casts sanitize; string/array/object casts keep the payload.
        ExprKind::Cast(kind, inner) => match kind {
            php_ast::ast::CastKind::Int
            | php_ast::ast::CastKind::Float
            | php_ast::ast::CastKind::Bool => Taint::NONE,
            php_ast::ast::CastKind::String => stringified_taint(inner, ctx, db, file),
            _ => expr_taint(inner, ctx, db, file),
        },

        ExprKind::Match(m) => m.arms.iter().fold(Taint::NONE, |acc, arm| {
            acc | expr_taint(&arm.body, ctx, db, file)
        }),

        // An element's key can carry taint just as much as its value.
        ExprKind::Array(elements) => elements.iter().fold(Taint::NONE, |acc, el| {
            let key = el
                .key
                .as_ref()
                .map_or(Taint::NONE, |k| expr_taint(k, ctx, db, file));
            acc | key | value_taint(el.value(), ctx, db, file)
        }),

        ExprKind::FunctionCall(fc) => {
            if let Some(closure) = callee_closure(&fc.name, ctx, db, file) {
                return closure_call_taint(&closure, &fc.args, ctx, db, file);
            }
            if let Some(name) = fc.name.kind.function_ref(db, file) {
                let lower_name = name
                    .global_name(db, file)
                    .map_or_else(String::new, crate::util::php_ident_lowercase);
                if matches!(
                    lower_name.as_str(),
                    "getallheaders" | "apache_request_headers"
                ) {
                    return Taint::ALL;
                }
                if reads_raw_request_body(&lower_name, &fc.args)
                    || is_request_helper_source(&lower_name, &fc.args)
                {
                    return Taint::ALL;
                }
                if let Some(taint) = filter_call_taint(&lower_name, &fc.args, ctx, db, file) {
                    return taint;
                }
                if let Some(removed) = sanitized_kinds(&lower_name) {
                    return args_taint(&fc.args, ctx, db, file).without(removed);
                }
                if lower_name == "preg_replace" {
                    if let Some(safe) = crate::taint_guard::replace_safe_kinds(&fc.args) {
                        return builtin_args_taint(&lower_name, &fc.args, ctx, db, file)
                            .without(safe)
                            | builtin_key_names_taint(&lower_name, &fc.args, ctx);
                    }
                }
                if let Some(restored) = unescaped_kinds(&lower_name) {
                    return args_taint(&fc.args, ctx, db, file).restore(restored);
                }
                if let Some(text) = format_text_args(&lower_name, &fc.args) {
                    return text
                        .into_iter()
                        .fold(Taint::NONE, |acc, e| acc | expr_taint(e, ctx, db, file));
                }
                // `JSON_HEX_TAG` encodes `<` and `>`, so the JSON can't open a tag.
                if lower_name == "json_encode" {
                    let taint = args_taint(&fc.args[..fc.args.len().min(1)], ctx, db, file)
                        | builtin_key_names_taint(&lower_name, &fc.args, ctx);
                    let hex_tag = fc
                        .args
                        .get(1)
                        .and_then(|a| a.value())
                        .is_some_and(|flags| has_flag(flags, "JSON_HEX_TAG"));
                    return if hex_tag {
                        taint.without(Taint::HTML)
                    } else {
                        taint
                    };
                }
                // The keys of an array whose key names are known exactly don't carry its values' taint.
                if matches!(
                    lower_name.as_str(),
                    "array_keys" | "array_key_first" | "array_key_last" | "key"
                ) {
                    if let Some(keys) = fc
                        .args
                        .first()
                        .and_then(|a| a.value())
                        .and_then(|array| exact_key_names(array, ctx, db, file))
                    {
                        return keys;
                    }
                }
                if let Some(taint) = callback_builtin_taint(&lower_name, &fc.args, ctx, db, file) {
                    return taint;
                }
                // `compact('id')` copies `$id`'s current value into the
                // returned array; only literal names are resolved, matching
                // `compact_return_type`.
                if lower_name == "compact" {
                    return fc
                        .args
                        .iter()
                        .filter_map(|a| a.value())
                        .fold(Taint::NONE, |acc, v| match &v.kind {
                            ExprKind::String(var_name) => acc | ctx.var_taint(var_name.as_str()),
                            _ => acc,
                        });
                }
                let found = name.find(db, file);
                let resolved = found
                    .as_ref()
                    .map_or(name.global.unwrap_or(&name.qualified), |(n, _)| n);
                let func = found.as_ref().map(|(_, f)| f);
                if func.as_ref().is_some_and(|f| f.is_taint_source) {
                    return Taint::ALL;
                }
                if let Some(f) = func.as_ref().filter(|f| is_user_code(f.location.as_ref())) {
                    return user_call_taint(
                        &f.params,
                        &fc.args,
                        f.return_type.as_deref(),
                        f.taint.as_deref(),
                        CalleeBody::of(f.location.as_ref(), file),
                        None,
                        || {
                            crate::db::taint_sources::function_returns_param(
                                db,
                                file,
                                f.fqn.as_ref(),
                            )
                        },
                        || false,
                        || {
                            crate::db::taint_sources::function_reads_source(
                                db,
                                file,
                                f.fqn.as_ref(),
                            )
                        },
                        || crate::db::function_taint_summary(db, file, f.fqn.as_ref()),
                        ctx,
                        db,
                        file,
                    );
                }
                // A builtin's result is derived from its arguments unless it can
                // only return values that carry no payload (`strlen`, `is_numeric`).
                if !resolved.contains('\\')
                    && crate::stubs::is_builtin_function(&crate::util::php_ident_lowercase(
                        resolved,
                    ))
                {
                    let safe_return = func
                        .as_ref()
                        .and_then(|f| f.return_type.as_deref())
                        .is_some_and(|ty| ty.is_void() || is_injection_safe_type(ty));
                    // A path sink returns file data, not the path; a tainted path
                    // is reported at the call itself.
                    let reads_path = classify_sink(&lower_name) == Some(SinkKind::File);
                    if !safe_return && !reads_path {
                        return builtin_args_taint(&lower_name, &fc.args, ctx, db, file)
                            | builtin_key_names_taint(&lower_name, &fc.args, ctx);
                    }
                }
            }
            Taint::NONE
        }

        ExprKind::New(new) => iterator_wrapper_taint(new, ctx, db, file),

        ExprKind::StaticPropertyAccess(spa) => crate::expr::assignment::resolve_static_prop_target(
            spa, ctx, db, file,
        )
        .map_or(Taint::NONE, |(fqcn, prop_name)| {
            let key = crate::db::taint_sources::prop_key(&prop_name, true);
            ctx.static_prop_taint(&fqcn, &prop_name) | class_property_taint(db, file, &fqcn, key)
        }),

        ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => {
            if let Some(key) = server_bag_read_key(&mc.object, &mc.method, &mc.args) {
                if is_server_bag_receiver(&mc.object, ctx, db, file) {
                    return server_bag_read_taint(key, &mc.args, ctx, db, file);
                }
            }
            // A tainted object's methods return its data (`$request->query->get('q')`).
            let receiver = expr_taint(&mc.object, ctx, db, file);
            let ExprKind::Identifier(method_name) = &mc.method.kind else {
                return receiver;
            };
            let method_lower = crate::util::php_ident_lowercase(method_name.as_ref());
            let Some(recv_ty) =
                crate::expr::assignment::resolve_chained_receiver_type(&mc.object, ctx, db, file)
            else {
                let produced = produced_object_classes(&mc.object, db, file);
                if !produced.is_empty()
                    && produced
                        .iter()
                        .all(|class| result_ignores_receiver(db, class, &method_lower))
                {
                    return args_taint(&mc.args, ctx, db, file);
                }
                // The method's own body decides what an object built in place returns.
                return produced
                    .iter()
                    .filter(|class| !class.is_empty())
                    .filter_map(|class| {
                        method_call_taint(
                            crate::db::Fqcn::from_str(db, class),
                            &method_lower,
                            &mc.args,
                            Some(&mc.object),
                            false,
                            ctx,
                            db,
                            file,
                        )
                    })
                    .fold(receiver, |acc, taint| acc | taint);
            };
            let mut taint = Taint::NONE;
            let mut receiver_flows = true;
            let (mut ignoring_receiver, mut objects) = (0, 0);
            for atom in &recv_ty.types {
                let mir_types::Atomic::TNamedObject { fqcn, .. } = atom else {
                    continue;
                };
                objects += 1;
                // Laravel's `route()` without a key returns the `Route` object.
                let route_object = method_lower == "route" && mc.args.is_empty();
                if !route_object
                    && is_framework_request_source(db, fqcn.as_ref(), &method_lower)
                    && !absorbed_by_supertype(db, fqcn.as_ref(), &recv_ty)
                {
                    return Taint::ALL;
                }
                if let Some(removed) = sanitized_method_kinds(db, fqcn.as_ref(), &method_lower) {
                    return args_taint(&mc.args, ctx, db, file).without(removed);
                }
                if result_ignores_receiver(db, fqcn.as_ref(), &method_lower) {
                    ignoring_receiver += 1;
                    taint |= args_taint(&mc.args, ctx, db, file);
                    continue;
                }
                let here = crate::db::Fqcn::from_str(db, fqcn.as_ref());
                if let Some((_, m)) =
                    crate::db::find_method_respecting_precedence(db, here, &method_lower)
                {
                    receiver_flows &= !m
                        .return_type
                        .as_deref()
                        .is_some_and(|ty| ty.is_void() || is_injection_safe_type(ty));
                }
                if let Some(t) = method_call_taint(
                    here,
                    &method_lower,
                    &mc.args,
                    Some(&mc.object),
                    false,
                    ctx,
                    db,
                    file,
                ) {
                    taint |= t;
                }
            }
            if receiver_flows && ignoring_receiver < objects {
                taint |= receiver;
            }
            taint
        }

        ExprKind::StaticMethodCall(smc) => {
            let fqcn_opt: Option<std::sync::Arc<str>> = if let Some(resolved) =
                smc.class.kind.resolve_name_ref(db, file)
            {
                match resolved.as_str() {
                    "self" | "static" => ctx.self_fqcn.clone().or_else(|| ctx.static_fqcn.clone()),
                    "parent" => ctx.parent_fqcn.clone(),
                    s => Some(std::sync::Arc::from(s)),
                }
            } else {
                // `$class::method()` / `$this::method()`.
                crate::expr::assignment::resolve_chained_receiver_type(&smc.class, ctx, db, file)
                    .and_then(|ty| {
                        ty.types.iter().find_map(|atom| match atom {
                            mir_types::Atomic::TClassString(Some(fqcn))
                            | mir_types::Atomic::TNamedObject { fqcn, .. }
                            | mir_types::Atomic::TSelf { fqcn }
                            | mir_types::Atomic::TStaticObject { fqcn }
                            | mir_types::Atomic::TParent { fqcn } => {
                                Some(std::sync::Arc::from(fqcn.as_ref()))
                            }
                            _ => None,
                        })
                    })
            };
            if let (Some(fqcn), ExprKind::Identifier(method_name)) = (fqcn_opt, &smc.method.kind) {
                let method_lower = crate::util::php_ident_lowercase(method_name.as_ref());
                if is_laravel_request_facade_source(fqcn.as_ref(), &method_lower) {
                    return Taint::ALL;
                }
                let here = crate::db::Fqcn::from_str(db, fqcn.as_ref());
                let forwards_this = match &smc.class.kind {
                    ExprKind::Variable(v) => v.trim_start_matches('$') == "this",
                    kind => kind.name_bare().is_some_and(|id| {
                        ["self", "static", "parent"]
                            .iter()
                            .any(|k| id.eq_ignore_ascii_case(k))
                    }),
                };
                let this_receiver = forwards_this.then(|| Expr {
                    kind: ExprKind::Variable("this".into()),
                    span: smc.class.span,
                });
                if let Some(taint) = method_call_taint(
                    here,
                    &method_lower,
                    &smc.args,
                    this_receiver.as_ref(),
                    false,
                    ctx,
                    db,
                    file,
                ) {
                    return taint;
                }
            }
            Taint::NONE
        }

        _ => Taint::NONE,
    }
}

pub(super) fn is_server_bag_receiver(
    object: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> bool {
    let (ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa)) =
        &unwrap_parens(object).kind
    else {
        return false;
    };
    crate::expr::assignment::resolve_chained_receiver_type(&pa.object, ctx, db, file).is_some_and(
        |ty| {
            ty.types.iter().all(|atom| {
                matches!(atom, mir_types::Atomic::TNamedObject { fqcn, .. }
                    if is_request_bag_property(db, fqcn.as_ref(), "server"))
            })
        },
    )
}

/// Classes an object expression may be an instance of.
pub(super) fn receiver_classes(
    object: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Vec<std::sync::Arc<str>> {
    if matches!(&object.kind, ExprKind::Variable(v) if v.trim_start_matches('$') == "this") {
        return ctx.self_fqcn.iter().cloned().collect();
    }
    crate::expr::assignment::resolve_chained_receiver_type(object, ctx, db, file)
        .map(|ty| {
            ty.types
                .iter()
                .filter_map(|atom| match atom {
                    mir_types::Atomic::TNamedObject { fqcn, .. } => {
                        Some(std::sync::Arc::from(fqcn.as_ref()))
                    }
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Whether a declaration at `location` has a body worth summarizing.
pub(super) fn is_user_code(location: Option<&mir_types::Location>) -> bool {
    location.is_some_and(|loc| !crate::stubs::is_stub_path(loc.file.as_ref()))
}

/// `…/vendor/<vendor>/<package>` prefix of a Composer dependency's path.
pub(super) fn vendor_package(path: &str) -> Option<&str> {
    let start = path
        .match_indices("vendor")
        .filter(|(i, _)| *i == 0 || path[..*i].ends_with(['/', '\\']))
        .map(|(i, _)| i + "vendor".len())
        .filter(|end| path[*end..].starts_with(['/', '\\']))
        .last()?;
    let mut separators = path[start..]
        .match_indices(['/', '\\'])
        .map(|(i, _)| start + i);
    separators.next()?;
    separators.next()?;
    Some(separators.next().map_or(path, |end| &path[..end]))
}

/// A callee in another Composer package than `from` is treated like a builtin:
/// its arguments pass through, and its body isn't summarized.
pub fn is_dependency_of(callee_path: &str, from: &str) -> bool {
    vendor_package(callee_path).is_some_and(|package| vendor_package(from) != Some(package))
}

/// Taint of a call to `method_lower` on `class`, or `None` when the method is
/// unknown; `dispatched` when the call names an abstract method `class` implements.
#[allow(clippy::too_many_arguments)]
pub(super) fn method_call_taint(
    class: crate::db::Fqcn<'_>,
    method_lower: &str,
    args: &[php_ast::owned::Arg],
    receiver: Option<&Expr>,
    dispatched: bool,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<Taint> {
    let (owner, m) = crate::db::find_method_respecting_precedence(db, class, method_lower)?;
    if m.is_taint_source {
        return Some(Taint::ALL);
    }
    if !is_user_code(m.location.as_ref()) && m.taint.is_none() {
        return None;
    }
    if m.taint.is_none() {
        let implementations = crate::db::taint_sources::method_implementations(
            db,
            class.name(db).as_str(),
            method_lower,
        );
        if !implementations.is_empty() {
            let taint = implementations
                .iter()
                .fold(Taint::NONE, |acc, implementor| {
                    let here = crate::db::Fqcn::from_str(db, implementor.as_str());
                    acc | method_call_taint(here, method_lower, args, receiver, true, ctx, db, file)
                        .unwrap_or_default()
                });
            return (!taint.is_empty()).then_some(taint);
        }
    }
    let body = match CalleeBody::of(m.location.as_ref(), file) {
        _ if m.is_abstract || crate::db::is_interface(db, owner.as_ref()) => CalleeBody::None,
        // Like the abstract declaration, an implementation in another package
        // is described by its annotations alone.
        CalleeBody::Dependency if dispatched => CalleeBody::None,
        body => body,
    };
    let taint = user_call_taint(
        &m.params,
        args,
        m.return_type.as_deref(),
        m.taint.as_deref(),
        body,
        receiver,
        || crate::db::taint_sources::method_returns_param(db, file, owner.as_ref(), method_lower),
        || crate::db::taint_sources::method_returns_prop(db, file, owner.as_ref(), method_lower),
        || crate::db::taint_sources::method_reads_source(db, file, owner.as_ref(), method_lower),
        || crate::db::method_taint_summary(db, file, owner.as_ref(), method_lower),
        ctx,
        db,
        file,
    );
    (!taint.is_empty()).then_some(taint)
}
/// Positions (among the values after the format) a printf-style format
/// interpolates as text; numeric conversions (`%d`, `%.2f`, `%x`) can't carry
/// a payload. `None` when the format is malformed.
pub(super) fn format_text_positions(format: &str) -> Option<Vec<usize>> {
    let mut positions = Vec::new();
    let mut next = 0;
    let mut chars = format.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            continue;
        }
        if chars.peek() == Some(&'%') {
            chars.next();
            continue;
        }
        let mut digits = String::new();
        while let Some(d) = chars.peek().copied().filter(char::is_ascii_digit) {
            digits.push(d);
            chars.next();
        }
        let position = if chars.peek() == Some(&'$') && !digits.is_empty() {
            chars.next();
            digits.parse::<usize>().ok()?.checked_sub(1)?
        } else {
            let position = next;
            next += 1;
            position
        };
        loop {
            match chars.next()? {
                '-' | '+' | ' ' | '0'..='9' | '.' => {}
                '\'' => {
                    chars.next()?;
                }
                's' | 'c' => {
                    positions.push(position);
                    break;
                }
                'b' | 'd' | 'e' | 'E' | 'f' | 'F' | 'g' | 'G' | 'h' | 'H' | 'o' | 'u' | 'x'
                | 'X' => break,
                _ => return None,
            }
        }
    }
    Some(positions)
}

/// The arguments of a printf-family call whose text ends up in the result:
/// the format itself and the values it interpolates with `%s`. `None` when the
/// call isn't printf-style.
pub fn format_text_args<'e>(
    fn_name_lower: &str,
    args: &'e [php_ast::owned::Arg],
) -> Option<Vec<&'e Expr>> {
    let (format_index, values_as_array) = match fn_name_lower {
        "sprintf" | "printf" => (0, false),
        "fprintf" => (1, false),
        "vsprintf" | "vprintf" => (0, true),
        "vfprintf" => (1, true),
        _ => return None,
    };
    if args.iter().any(|a| a.name.is_some() || a.unpack()) {
        return None;
    }
    let format = args.get(format_index)?.value()?;
    let values = || args.iter().skip(format_index + 1).filter_map(|a| a.value());
    let ExprKind::String(literal) = &format.kind else {
        return Some(std::iter::once(format).chain(values()).collect());
    };
    let Some(positions) = format_text_positions(literal.as_str()) else {
        return Some(std::iter::once(format).chain(values()).collect());
    };
    if values_as_array {
        return Some(if positions.is_empty() {
            Vec::new()
        } else {
            values().collect()
        });
    }
    Some(
        values()
            .enumerate()
            .filter(|(i, _)| positions.contains(i))
            .map(|(_, v)| v)
            .collect(),
    )
}

/// Classes of the object a `new` or function call yields, when that is declared.
pub(super) fn produced_object_classes(
    expr: &Expr,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Vec<String> {
    match &unwrap_parens(expr).kind {
        ExprKind::New(n) => match n.class.kind.resolve_name_ref(db, file) {
            Some(resolved) => {
                if matches!(resolved.as_str(), "self" | "static" | "parent") {
                    Vec::new()
                } else {
                    vec![resolved]
                }
            }
            _ => Vec::new(),
        },
        ExprKind::FunctionCall(fc) => {
            let Some(name) = fc.name.kind.function_ref(db, file) else {
                return Vec::new();
            };
            let Some(ty) = name
                .find(db, file)
                .map(|(_, f)| f)
                .filter(|f| !f.is_taint_source)
                .and_then(|f| f.return_type.as_deref().cloned())
            else {
                return Vec::new();
            };
            ty.types
                .iter()
                .filter_map(|atom| match atom {
                    mir_types::Atomic::TNamedObject { fqcn, .. } => Some(fqcn.to_string()),
                    mir_types::Atomic::TFalse | mir_types::Atomic::TNull => None,
                    _ => Some(String::new()),
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

/// Whether a binary operator's result is a number or a bool, never a copy of an operand's text.
fn yields_number_or_bool(
    op: php_ast::ast::BinaryOp,
    left: &Expr,
    right: &Expr,
    ctx: &FlowState,
) -> bool {
    use php_ast::ast::BinaryOp::*;
    match op {
        Sub | Mul | Div | Mod | Pow | ShiftLeft | ShiftRight | Spaceship | Equal | NotEqual
        | Identical | NotIdentical | Less | Greater | LessOrEqual | GreaterOrEqual | BooleanAnd
        | BooleanOr | LogicalAnd | LogicalOr | LogicalXor | Instanceof => true,
        // `+` unions arrays and the bitwise operators combine strings byte by byte; a scalar operand rules both out.
        Add | BitwiseAnd | BitwiseOr | BitwiseXor => {
            is_known_scalar(left, ctx) || is_known_scalar(right, ctx)
        }
        _ => false,
    }
}

fn is_known_scalar(expr: &Expr, ctx: &FlowState) -> bool {
    match &expr.kind {
        ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Bool(_) | ExprKind::Null => true,
        ExprKind::Parenthesized(inner) => is_known_scalar(inner, ctx),
        ExprKind::UnaryPrefix(u) => matches!(
            u.op,
            php_ast::ast::UnaryPrefixOp::Negate
                | php_ast::ast::UnaryPrefixOp::Plus
                | php_ast::ast::UnaryPrefixOp::BooleanNot
        ),
        ExprKind::Cast(kind, _) => matches!(
            kind,
            php_ast::ast::CastKind::Int
                | php_ast::ast::CastKind::Float
                | php_ast::ast::CastKind::Bool
        ),
        ExprKind::Variable(name) => {
            is_injection_safe_type(&ctx.get_var(name.trim_start_matches('$')))
        }
        _ => false,
    }
}

/// Taint of a ternary branch, seeing the validation guards its condition proves.
fn branch_taint(
    condition: &Expr,
    holds: bool,
    branch: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    if !has_taint_guard(condition) {
        return expr_taint(branch, ctx, db, file);
    }
    let mut branch_ctx = ctx.clone();
    crate::narrowing::narrow_from_condition(condition, &mut branch_ctx, holds, db, file);
    expr_taint(branch, &branch_ctx, db, file)
}

fn has_taint_guard(condition: &Expr) -> bool {
    if crate::taint_guard::guarded_when(condition, true).is_some()
        || crate::taint_guard::guarded_when(condition, false).is_some()
    {
        return true;
    }
    match &condition.kind {
        ExprKind::Parenthesized(inner) => has_taint_guard(inner),
        ExprKind::UnaryPrefix(u) if u.op == php_ast::ast::UnaryPrefixOp::BooleanNot => {
            has_taint_guard(&u.operand)
        }
        ExprKind::Binary(b) => has_taint_guard(&b.left) || has_taint_guard(&b.right),
        _ => false,
    }
}
