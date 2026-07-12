/// Expression analyzer — infers the `Type` type of any PHP expression.
use std::sync::Arc;

use php_ast::owned::ExprKind;

use mir_issues::{Issue, IssueBuffer, IssueKind, Location, Severity};
use mir_types::{Atomic, CloneValidity, Type};

use crate::body_analysis::AnalysisMode;
use crate::db::MirDatabase;
use crate::flow_state::FlowState;
use crate::php_version::PhpVersion;
use crate::symbol::{ReferenceKind, ResolvedSymbol};

mod arrays;
mod assignment;
mod binary;
mod casts;
mod closures;
mod conditional;
pub(crate) mod helpers;
mod intrinsics;
mod literals;
mod objects;
mod unary;
mod variables;

pub(crate) use binary::operand_is_definitely_zero;
#[allow(unused_imports)]
pub use helpers::{duplicate_literal_conditions, extract_simple_var, infer_arithmetic, infer_div};

// ---------------------------------------------------------------------------
// ExpressionAnalyzer
// ---------------------------------------------------------------------------

pub struct ExpressionAnalyzer<'a> {
    pub db: &'a dyn MirDatabase,
    pub file: Arc<str>,
    pub source: &'a str,
    pub source_map: &'a php_rs_parser::source_map::SourceMap,
    pub issues: &'a mut IssueBuffer,
    pub symbols: &'a mut Vec<ResolvedSymbol>,
    pub php_version: PhpVersion,
    pub mode: AnalysisMode,
    /// Whether `declare(strict_types=1)` is active for the calling file.
    /// When true, coercive PHP typing (e.g. Stringable → string) must not be
    /// silently allowed — the runtime would throw a TypeError.
    pub strict_types: bool,
    /// When false, `record_symbol*` calls are no-ops — see
    /// `StatementsAnalyzer::collect_symbols`.
    pub collect_symbols: bool,
    /// When true, we are inside an existence-check context (isset/empty/??) where missing
    /// variables and missing array offsets are not errors — they are what is being tested.
    in_existence_check: bool,
    /// `(key type, value type)` for every `yield`/`yield from` encountered so
    /// far in the enclosing function body, regardless of which branch/loop it
    /// syntactically sits in — the AST is walked once, so no branch-merge
    /// logic is needed. Borrowed from the owning `StatementsAnalyzer` so it
    /// survives across the many short-lived `ExpressionAnalyzer`s created
    /// while walking one function body. Read back by `body_analysis` to
    /// build the function's inferred `Generator<K, V, S, R>` return type.
    pub(crate) yielded_types: &'a mut Vec<(Type, Type)>,
}

impl<'a> ExpressionAnalyzer<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db: &'a dyn MirDatabase,
        file: Arc<str>,
        source: &'a str,
        source_map: &'a php_rs_parser::source_map::SourceMap,
        issues: &'a mut IssueBuffer,
        symbols: &'a mut Vec<ResolvedSymbol>,
        php_version: PhpVersion,
        mode: AnalysisMode,
        yielded_types: &'a mut Vec<(Type, Type)>,
    ) -> Self {
        Self {
            db,
            file,
            source,
            source_map,
            issues,
            symbols,
            php_version,
            mode,
            strict_types: false,
            collect_symbols: true,
            in_existence_check: false,
            yielded_types,
        }
    }

    /// Run `f` in an existence-check context (isset/empty/??/??=), suppressing
    /// missing-variable and missing-offset diagnostics for the duration.
    pub(super) fn with_existence_check<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let old = self.in_existence_check;
        self.in_existence_check = true;
        let result = f(self);
        self.in_existence_check = old;
        result
    }

    /// Record a resolved symbol.
    pub fn record_symbol(&mut self, span: php_ast::Span, kind: ReferenceKind, resolved_type: Type) {
        if !self.collect_symbols {
            return;
        }
        self.symbols.push(ResolvedSymbol {
            file: self.file.clone(),
            span,
            expr_span: None,
            kind,
            resolved_type,
        });
    }

    pub fn record_symbol_with_expr_span(
        &mut self,
        span: php_ast::Span,
        expr_span: php_ast::Span,
        kind: ReferenceKind,
        resolved_type: Type,
    ) {
        if !self.collect_symbols {
            return;
        }
        self.symbols.push(ResolvedSymbol {
            file: self.file.clone(),
            span,
            expr_span: Some(expr_span),
            kind,
            resolved_type,
        });
    }

    pub fn analyze(&mut self, expr: &php_ast::owned::Expr, ctx: &mut FlowState) -> Type {
        match &expr.kind {
            // --- Literals ---------------------------------------------------
            ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::String(_)
            | ExprKind::Bool(_)
            | ExprKind::Null => literals::analyze(&expr.kind),

            ExprKind::InterpolatedString(parts) | ExprKind::Heredoc { parts, .. } => {
                for part in parts.iter() {
                    if let php_ast::owned::StringPart::Expr(e) = part {
                        let expr_ty = self.analyze(e, ctx);
                        self.check_interpolation_implicit_to_string_cast(&expr_ty, e.span);
                    }
                }
                Type::single(Atomic::TString)
            }
            ExprKind::Nowdoc { .. } => Type::single(Atomic::TString),
            ExprKind::ShellExec(_) => {
                self.emit(
                    IssueKind::ForbiddenCode {
                        message: "Use of shell_exec (backtick) is forbidden".to_string(),
                    },
                    Severity::Warning,
                    expr.span,
                );
                Type::single(Atomic::TString)
            }

            // --- Variables --------------------------------------------------
            ExprKind::Variable(name) => self.analyze_variable(name.as_ref(), expr, ctx),
            ExprKind::VariableVariable(inner) => self.analyze_variable_variable(inner, ctx),
            ExprKind::Identifier(name) => self.analyze_identifier(name.as_ref(), expr, ctx),

            // --- Assignment -------------------------------------------------
            ExprKind::Assign(a) => self.analyze_assign(a, expr.span, ctx),

            // --- Binary operations ------------------------------------------
            ExprKind::Binary(b) => self.analyze_binary_expr(b, expr.span, ctx),

            // --- Unary ------------------------------------------------------
            ExprKind::UnaryPrefix(u) => self.analyze_unary_prefix(u, ctx),
            ExprKind::UnaryPostfix(u) => self.analyze_unary_postfix(u, ctx),

            // --- Ternary / null coalesce ------------------------------------
            ExprKind::Ternary(t) => self.analyze_ternary(t, ctx),
            ExprKind::NullCoalesce(nc) => self.analyze_null_coalesce(nc, ctx),

            // --- Casts ------------------------------------------------------
            ExprKind::Cast(kind, inner) => self.analyze_cast(kind, inner, ctx),

            // --- Error suppression ------------------------------------------
            ExprKind::ErrorSuppress(inner) => self.analyze(inner, ctx),

            // --- Parenthesized ----------------------------------------------
            ExprKind::Parenthesized(inner) => self.analyze(inner, ctx),

            // --- Array literals ---------------------------------------------
            ExprKind::Array(elements) => self.analyze_array(elements, ctx),

            // --- Array access -----------------------------------------------
            ExprKind::ArrayAccess(aa) => self.analyze_array_access(aa, expr, ctx),

            // --- isset / empty ----------------------------------------------
            ExprKind::Isset(exprs) => {
                self.with_existence_check(|ea| {
                    for e in exprs.iter() {
                        ea.analyze(e, ctx);
                    }
                });
                Type::single(Atomic::TBool)
            }
            ExprKind::Empty(inner) => {
                self.with_existence_check(|ea| ea.analyze(inner, ctx));
                Type::single(Atomic::TBool)
            }

            // --- print ------------------------------------------------------
            ExprKind::Print(inner) => {
                let expr_ty = self.analyze(inner, ctx);
                self.check_interpolation_implicit_to_string_cast(&expr_ty, inner.span);
                Type::single(Atomic::TLiteralInt(1))
            }

            // --- clone ------------------------------------------------------
            ExprKind::Clone(inner) => {
                let ty = self.analyze(inner, ctx);
                self.check_clone_target(&ty, expr.span);
                self.check_clone_visibility(&ty, expr.span, ctx.self_fqcn.as_deref());
                self.check_clone_deprecated(&ty, expr.span);
                ty
            }
            ExprKind::CloneWith(inner, _props) => {
                let ty = self.analyze(inner, ctx);
                self.check_clone_target(&ty, expr.span);
                self.check_clone_visibility(&ty, expr.span, ctx.self_fqcn.as_deref());
                ty
            }

            // --- new ClassName(...) ----------------------------------------
            ExprKind::New(n) => self.analyze_new(n, expr.span, ctx),

            // --- Anonymous class -------------------------------------------
            ExprKind::AnonymousClass(anon) => {
                let mut sa = crate::stmt::StatementsAnalyzer::new(
                    self.db,
                    self.file.clone(),
                    self.source,
                    self.source_map,
                    self.issues,
                    self.symbols,
                    self.php_version,
                    self.mode,
                );
                sa.collect_symbols = self.collect_symbols;
                sa.analyze_class_decl_stmt(anon, ctx);
                Type::single(Atomic::TObject)
            }

            // --- Property access -------------------------------------------
            ExprKind::PropertyAccess(pa) => self.analyze_property_access(pa, expr.span, ctx),

            ExprKind::NullsafePropertyAccess(pa) => self.analyze_nullsafe_property_access(pa, ctx),

            ExprKind::StaticPropertyAccess(spa) => {
                if matches!(&spa.class.kind, ExprKind::Variable(_)) {
                    let _ = self.analyze(&spa.class, ctx);
                }
                self.analyze_static_property_access(spa, ctx)
            }

            ExprKind::ClassConstAccess(cca) => {
                // When the class part is a variable (e.g. `$obj::class`, `$obj::CONST`),
                // analyze it so the variable read is tracked and the write is consumed.
                if matches!(&cca.class.kind, ExprKind::Variable(_)) {
                    let _ = self.analyze(&cca.class, ctx);
                }
                self.analyze_class_const_access(cca, expr.span, ctx)
            }

            ExprKind::ClassConstAccessDynamic { class, member } => {
                if matches!(&class.kind, ExprKind::Variable(_)) {
                    let _ = self.analyze(class, ctx);
                }
                let _ = self.analyze(member, ctx);
                Type::mixed()
            }
            ExprKind::StaticPropertyAccessDynamic { class, member } => {
                if matches!(&class.kind, ExprKind::Variable(_)) {
                    let _ = self.analyze(class, ctx);
                }
                let _ = self.analyze(member, ctx);
                Type::mixed()
            }

            // --- Method calls ----------------------------------------------
            ExprKind::MethodCall(mc) => {
                crate::call::CallAnalyzer::analyze_method_call(self, mc, ctx, expr.span, false)
            }

            ExprKind::NullsafeMethodCall(mc) => {
                crate::call::CallAnalyzer::analyze_method_call(self, mc, ctx, expr.span, true)
            }

            ExprKind::StaticMethodCall(smc) => {
                crate::call::CallAnalyzer::analyze_static_method_call(self, smc, ctx, expr.span)
            }

            ExprKind::StaticDynMethodCall(smc) => {
                crate::call::CallAnalyzer::analyze_static_dyn_method_call(self, smc, ctx)
            }

            // --- Function calls --------------------------------------------
            ExprKind::FunctionCall(fc) => {
                crate::call::CallAnalyzer::analyze_function_call(self, fc, ctx, expr.span)
            }

            // --- Closures / arrow functions --------------------------------
            ExprKind::Closure(c) => self.analyze_closure(c, expr.span, ctx),

            ExprKind::ArrowFunction(af) => self.analyze_arrow_function(af, expr.span, ctx),

            ExprKind::CallableCreate(cc) => self.callable_create_type(cc, expr.span, ctx),

            // --- Match expression ------------------------------------------
            ExprKind::Match(m) => self.analyze_match(m, expr.span, ctx),

            // --- Throw as expression (PHP 8) --------------------------------
            ExprKind::ThrowExpr(e) => {
                self.analyze(e, ctx);
                Type::single(Atomic::TNever)
            }

            // --- Yield -----------------------------------------------------
            ExprKind::Yield(y) => self.analyze_yield(y, ctx),

            // --- Magic constants -------------------------------------------
            ExprKind::MagicConst(kind) => ExpressionAnalyzer::analyze_magic_const(kind),

            // --- Include/require --------------------------------------------
            ExprKind::Include(_, inner) => {
                self.analyze(inner, ctx);
                // A require/include can read any variable currently in scope, so
                // mark all pending writes as consumed and mark the names as read.
                // This covers both the last_write_locs path and the assigned_vars
                // fallback in emit_unused_variables.
                let names: Vec<mir_types::Name> = ctx.last_write_locs.keys().copied().collect();
                for name in names {
                    ctx.read_vars.insert(name);
                }
                for (name, locs) in ctx.last_write_locs.drain() {
                    for loc in locs {
                        ctx.consumed_write_locs.insert((name, loc));
                    }
                }
                Type::mixed()
            }

            // --- Eval -------------------------------------------------------
            ExprKind::Eval(inner) => {
                self.analyze(inner, ctx);
                Type::mixed()
            }

            // --- Exit -------------------------------------------------------
            ExprKind::Exit(opt) => {
                if let Some(e) = opt {
                    self.analyze(e, ctx);
                }
                ctx.diverges = true;
                Type::single(Atomic::TNever)
            }

            // --- Error node (parse error placeholder) ----------------------
            ExprKind::Error => Type::mixed(),

            // --- Omitted array slot (e.g. [, $b] destructuring) ------------
            ExprKind::Omit => Type::single(Atomic::TNull),
        }
    }

    // -----------------------------------------------------------------------
    // Issue emission
    // -----------------------------------------------------------------------

    fn offset_to_line_col(&self, offset: u32) -> (u32, u16) {
        crate::diagnostics::offset_to_line_col(self.source, offset, self.source_map)
    }

    /// Convert an AST span to `(line, col_start, col_end)` for reference recording.
    fn callable_create_type(
        &mut self,
        cc: &php_ast::owned::CallableCreateExpr,
        outer_span: php_ast::Span,
        ctx: &mut FlowState,
    ) -> Type {
        use php_ast::owned::CallableCreateKind;
        match &cc.kind {
            CallableCreateKind::Function(name_expr) => {
                if let ExprKind::Identifier(name) = &name_expr.kind {
                    let resolved_fqn =
                        crate::db::resolve_name(self.db, self.file.as_ref(), name.as_ref());
                    let db = self.db;
                    let here = crate::db::Fqcn::from_str(db, &resolved_fqn);
                    if let Some(f) = crate::db::find_function(db, here) {
                        if let Some((used, canonical)) =
                            crate::fqcn_case_mismatch(&resolved_fqn, f.fqn.as_ref())
                        {
                            let span = if name_expr.span.start < name_expr.span.end {
                                name_expr.span
                            } else {
                                outer_span
                            };
                            self.emit(
                                mir_issues::IssueKind::WrongCaseFunction { used, canonical },
                                mir_issues::Severity::Info,
                                span,
                            );
                        }
                        let return_ty = f
                            .return_type
                            .as_deref()
                            .cloned()
                            .unwrap_or_else(Type::mixed);
                        // No receiver for a plain function — self/static/parent can't
                        // legally appear in its signature, so an empty fqcn/type-param
                        // list is a safe no-op for build_closure_from_resolved_params's
                        // static-substitution step. Routed through the same helper as
                        // the method FCC cases below so the function's own @template
                        // params get bound to their declared bound (see the helper's
                        // doc comment) instead of leaking as bare, unchecked
                        // TTemplateParam atoms into every call through the closure.
                        let empty_fqcn: Arc<str> = Arc::from("");
                        return Type::single(Self::build_closure_from_resolved_params(
                            &f.params,
                            return_ty,
                            &rustc_hash::FxHashMap::default(),
                            &f.template_params,
                            &empty_fqcn,
                            &[],
                        ));
                    }
                }
                Type::single(Atomic::TCallable {
                    params: None,
                    return_type: None,
                })
            }

            CallableCreateKind::Method { object, method }
            | CallableCreateKind::NullsafeMethod { object, method } => {
                let nullsafe = matches!(&cc.kind, CallableCreateKind::NullsafeMethod { .. });
                let obj_ty = self.analyze(object, ctx);
                let method_name = match &method.kind {
                    ExprKind::Identifier(name) => name.clone(),
                    _ => {
                        return Type::single(Atomic::TCallable {
                            params: None,
                            return_type: None,
                        })
                    }
                };
                let method_name_lower = crate::util::php_ident_lowercase(method_name.as_ref());
                if let Some((fqcn, receiver_type_params)) =
                    obj_ty.remove_null().types.iter().find_map(|a| {
                        a.named_object_fqcn().map(|fqcn| {
                            let type_params = match a {
                                Atomic::TNamedObject { type_params, .. } => type_params.to_vec(),
                                _ => Vec::new(),
                            };
                            (fqcn, type_params)
                        })
                    })
                {
                    let fqcn_resolved = crate::db::resolve_name(self.db, self.file.as_ref(), fqcn);
                    let fqcn_arc: Arc<str> = Arc::from(fqcn_resolved.as_str());
                    if let Some(resolved) = crate::call::method::resolve_method_from_db(
                        self,
                        &fqcn_arc,
                        &method_name_lower,
                    ) {
                        // Substitute this receiver's own bound type params (e.g.
                        // `Box<int>`'s T -> int) into the method's raw param/return
                        // types before building the callable — otherwise a
                        // first-class-callable on a generic method loses the
                        // binding that the direct-call path already applies.
                        let class_tps = crate::db::class_template_params(self.db, &fqcn_arc)
                            .map(|tps| tps.to_vec())
                            .unwrap_or_default();
                        let mut bindings =
                            crate::generic::build_class_bindings(&class_tps, &receiver_type_params);
                        for (k, v) in
                            crate::db::inherited_template_bindings(self.db, &fqcn_arc, &bindings)
                        {
                            bindings.entry(k).or_insert(v);
                        }
                        let closure = Self::build_closure_from_resolved_params(
                            &resolved.params,
                            resolved.return_ty_raw,
                            &bindings,
                            &resolved.template_params,
                            &fqcn_arc,
                            &receiver_type_params,
                        );
                        return if nullsafe {
                            Type::nullable(closure)
                        } else {
                            Type::single(closure)
                        };
                    }
                }
                Type::single(Atomic::TCallable {
                    params: None,
                    return_type: None,
                })
            }

            CallableCreateKind::StaticMethod { class, method } => {
                let method_name = match &method.kind {
                    ExprKind::Identifier(name) => name.clone(),
                    _ => {
                        return Type::single(Atomic::TCallable {
                            params: None,
                            return_type: None,
                        })
                    }
                };
                let method_name_lower = crate::util::php_ident_lowercase(method_name.as_ref());
                let mut receiver_type_params: Vec<Type> = Vec::new();
                let fqcn = match &class.kind {
                    ExprKind::Identifier(name) => {
                        let resolved =
                            crate::db::resolve_name(self.db, self.file.as_ref(), name.as_ref());
                        match crate::util::php_ident_lowercase(&resolved).as_str() {
                            "self" => ctx.self_fqcn.as_deref().unwrap_or("self").to_string(),
                            "parent" => ctx.parent_fqcn.as_deref().unwrap_or("parent").to_string(),
                            "static" => ctx
                                .static_fqcn
                                .as_deref()
                                .unwrap_or(ctx.self_fqcn.as_deref().unwrap_or("static"))
                                .to_string(),
                            _ => resolved,
                        }
                    }
                    _ => {
                        let ty = self.analyze(class, ctx);
                        match ty.types.iter().find_map(|a| {
                            a.named_object_fqcn().map(|fqcn| {
                                let type_params = match a {
                                    Atomic::TNamedObject { type_params, .. } => {
                                        type_params.to_vec()
                                    }
                                    _ => Vec::new(),
                                };
                                (fqcn.to_string(), type_params)
                            })
                        }) {
                            Some((fqcn, type_params)) => {
                                receiver_type_params = type_params;
                                fqcn
                            }
                            None => {
                                return Type::single(Atomic::TCallable {
                                    params: None,
                                    return_type: None,
                                })
                            }
                        }
                    }
                };
                let fqcn_arc: Arc<str> = Arc::from(fqcn.as_str());
                if let Some(resolved) =
                    crate::call::method::resolve_method_from_db(self, &fqcn_arc, &method_name_lower)
                {
                    // Same reasoning as the instance-method FCC case above: substitute
                    // the receiver's own bound type params before building the callable.
                    let class_tps = crate::db::class_template_params(self.db, &fqcn_arc)
                        .map(|tps| tps.to_vec())
                        .unwrap_or_default();
                    let mut bindings =
                        crate::generic::build_class_bindings(&class_tps, &receiver_type_params);
                    for (k, v) in
                        crate::db::inherited_template_bindings(self.db, &fqcn_arc, &bindings)
                    {
                        bindings.entry(k).or_insert(v);
                    }
                    return Type::single(Self::build_closure_from_resolved_params(
                        &resolved.params,
                        resolved.return_ty_raw,
                        &bindings,
                        &resolved.template_params,
                        &fqcn_arc,
                        &receiver_type_params,
                    ));
                }
                Type::single(Atomic::TCallable {
                    params: None,
                    return_type: None,
                })
            }
        }
    }

    fn build_closure_from_resolved_params(
        params: &[mir_codebase::definitions::DeclaredParam],
        return_ty: Type,
        bindings: &rustc_hash::FxHashMap<mir_types::Name, Type>,
        own_template_params: &[mir_codebase::definitions::TemplateParam],
        receiver_fqcn: &Arc<str>,
        receiver_type_params: &[Type],
    ) -> Atomic {
        // A method-level `@template` SHADOWS a same-named class template, so it
        // must not get the class-level binding baked in here. Unlike a direct
        // call (where `check_args` sees the real arguments and infers+bound-checks
        // the method's own templates per call), a first-class-callable value is
        // built once and may be called anywhere later with no template_params in
        // scope to infer against (see the `template_params: &[]` call to
        // `check_args` for a closure-typed callee) — so a bare, still-templated
        // param would silently accept any argument. Substituting each of the
        // method's own templates with its declared bound (or `mixed` if
        // unbounded, the same fallback `infer_template_bindings` uses) instead
        // keeps every call through the closure checked against at least that
        // bound, which is always sound: an argument to the real method must
        // satisfy the bound regardless of what the template later infers to.
        let mut bindings = bindings.clone();
        for tp in own_template_params {
            let bound_ty = tp.bound.as_deref().cloned().unwrap_or_else(Type::mixed);
            bindings.insert(mir_types::Name::from(tp.name.as_ref()), bound_ty);
        }
        let resolve = |t: Type| -> Type {
            // `self`/`static` must resolve to the receiver's concrete class, the
            // same way a direct call's return type already does — otherwise a
            // late-static-bound method's FCC closure carries the class that
            // physically declares the method instead of the real receiver.
            crate::call::substitute_static_in_return(t, receiver_fqcn, receiver_type_params)
                .substitute_templates(&bindings)
        };
        let fn_params: Box<[mir_types::atomic::FnParam]> = params
            .iter()
            .map(|p| mir_types::atomic::FnParam {
                name: mir_types::Name::from(p.name.as_ref()),
                ty: p
                    .ty
                    .as_deref()
                    .cloned()
                    .map(resolve)
                    .map(mir_types::compact::SimpleType::from_union),
                out_ty: p
                    .out_ty
                    .as_deref()
                    .cloned()
                    .map(resolve)
                    .map(mir_types::compact::SimpleType::from_union),
                default: if p.has_default {
                    Some(mir_types::compact::SimpleType::from_union(Type::mixed()))
                } else {
                    None
                },
                is_variadic: p.is_variadic,
                is_byref: p.is_byref,
                is_optional: p.is_optional,
            })
            .collect();
        Atomic::TClosure {
            data: Box::new(mir_types::atomic::ClosureData {
                params: fn_params,
                return_type: resolve(return_ty),
                this_type: None,
            }),
        }
    }

    /// Record a reference location for `symbol_key` at `span`, unless in inference-only mode.
    pub(crate) fn record_ref(&self, symbol_key: Arc<str>, span: php_ast::Span) {
        if self.mode == AnalysisMode::InferenceOnly {
            return;
        }
        let (line, col_start) = self.offset_to_line_col(span.start);
        let (_, col_end) = self.offset_to_line_col(span.end);
        self.db.record_reference_location(crate::db::RefLoc {
            symbol_key,
            file: self.file.clone(),
            line,
            col_start,
            col_end,
        });
    }

    /// Walk a type hint and emit `UndefinedClass` for any named type not in the codebase.
    fn check_type_hint(&mut self, hint: &php_ast::owned::TypeHint) {
        use php_ast::owned::TypeHintKind;
        match &hint.kind {
            TypeHintKind::Named(name) => {
                let name_str = crate::parser::name_to_string_owned(name);
                if matches!(
                    crate::util::php_ident_lowercase(&name_str).as_str(),
                    "self"
                        | "static"
                        | "parent"
                        | "null"
                        | "true"
                        | "false"
                        | "never"
                        | "void"
                        | "mixed"
                        | "object"
                        | "callable"
                        | "iterable"
                ) {
                    return;
                }
                let resolved = crate::db::resolve_name(self.db, &self.file, &name_str);
                if !crate::db::class_exists(self.db, &resolved) {
                    self.emit(
                        IssueKind::UndefinedClass { name: resolved },
                        Severity::Error,
                        hint.span,
                    );
                } else {
                    let fqcn: Arc<str> = Arc::from(resolved.as_str());
                    self.record_ref(Arc::from(format!("cls:{fqcn}")), hint.span);
                    self.record_symbol(
                        hint.span,
                        ReferenceKind::ClassReference(fqcn),
                        mir_types::Type::single(mir_types::Atomic::TClassString(None)),
                    );
                }
            }
            TypeHintKind::Nullable(inner) => self.check_type_hint(inner),
            TypeHintKind::Union(parts) | TypeHintKind::Intersection(parts) => {
                for part in parts.iter() {
                    self.check_type_hint(part);
                }
            }
            TypeHintKind::Keyword(_, _) => {}
        }
    }

    pub fn emit(&mut self, kind: IssueKind, severity: Severity, span: php_ast::Span) {
        let (line, col_start) = self.offset_to_line_col(span.start);

        let (line_end, col_end) = if span.start < span.end {
            let (end_line, end_col) = self.offset_to_line_col(span.end);
            (end_line, end_col)
        } else {
            (line, col_start)
        };

        let mut issue = Issue::new(
            kind,
            Location {
                file: self.file.clone(),
                line,
                line_end,
                col_start,
                col_end: crate::diagnostics::clamp_col_end(line, line_end, col_start, col_end),
            },
        );
        issue.severity = severity;
        // Store the source snippet for baseline matching.
        if span.start < span.end {
            let s = span.start as usize;
            let e = (span.end as usize).min(self.source.len());
            if let Some(text) = self.source.get(s..e) {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    issue.snippet = Some(trimmed.to_string());
                }
            }
        }
        self.issues.add(issue);
    }

    /// Emit a clone diagnostic when `ty` is (possibly) not an object. `mixed`
    /// takes precedence (matching the historical `MixedClone` behaviour), then
    /// definite non-objects (`InvalidClone`) and mixed object/non-object unions
    /// (`PossiblyInvalidClone`).
    fn check_clone_target(&mut self, ty: &Type, span: php_ast::Span) {
        if ty.is_mixed() {
            self.emit(IssueKind::MixedClone, Severity::Info, span);
            return;
        }
        match ty.clone_validity() {
            CloneValidity::Invalid => self.emit(
                IssueKind::InvalidClone { ty: ty.to_string() },
                Severity::Error,
                span,
            ),
            CloneValidity::PossiblyInvalid => self.emit(
                IssueKind::PossiblyInvalidClone { ty: ty.to_string() },
                Severity::Info,
                span,
            ),
            CloneValidity::Cloneable | CloneValidity::Unknown => {}
        }
    }

    /// Emit DeprecatedMethodCall if the cloned object has a deprecated __clone() method.
    fn check_clone_deprecated(&mut self, ty: &Type, span: php_ast::Span) {
        for atomic in &ty.types {
            if let Atomic::TNamedObject { fqcn, .. } = atomic {
                let fqcn_str = fqcn.as_ref();
                if let Some((_, method)) = crate::db::find_method_in_chain(
                    self.db,
                    crate::db::Fqcn::from_str(self.db, fqcn_str),
                    "__clone",
                ) {
                    if let Some(msg) = &method.deprecated {
                        self.emit(
                            IssueKind::DeprecatedMethodCall {
                                class: fqcn_str.to_string(),
                                method: "__clone".to_string(),
                                message: Some(msg.clone()).filter(|m| !m.is_empty()),
                            },
                            Severity::Info,
                            span,
                        );
                    }
                }
            }
        }
    }

    /// Emit InvalidClone if the object's __clone() is private and the current
    /// class context doesn't have access to it.
    /// Only runs when the type union is purely cloneable (not already flagged
    /// by check_clone_target for non-object members).
    fn check_clone_visibility(
        &mut self,
        ty: &Type,
        span: php_ast::Span,
        caller_fqcn: Option<&str>,
    ) {
        use mir_codebase::definitions::Visibility;
        use mir_types::CloneValidity;
        // Skip if the type already has structural clone issues (non-object components)
        match ty.clone_validity() {
            CloneValidity::Invalid | CloneValidity::PossiblyInvalid => return,
            _ => {}
        }
        for atomic in &ty.types {
            if let Atomic::TNamedObject { fqcn, .. } = atomic {
                let fqcn_str = fqcn.as_ref();
                if let Some((owner_fqcn, method)) = crate::db::find_method_in_chain(
                    self.db,
                    crate::db::Fqcn::from_str(self.db, fqcn_str),
                    "__clone",
                ) {
                    if method.visibility == Visibility::Private {
                        // Private __clone is accessible only from within the declaring class
                        let accessible = caller_fqcn
                            .map(|c| c.eq_ignore_ascii_case(owner_fqcn.as_ref()))
                            .unwrap_or(false);
                        if !accessible {
                            self.emit(
                                IssueKind::InvalidClone {
                                    ty: fqcn_str.to_string(),
                                },
                                Severity::Error,
                                span,
                            );
                        }
                    }
                }
            }
        }
    }

    fn check_interpolation_implicit_to_string_cast(&mut self, ty: &Type, span: php_ast::Span) {
        for atomic in &ty.types {
            if let Atomic::TNamedObject { fqcn, .. } = atomic {
                let fqcn_str = fqcn.as_ref();
                if !crate::db::has_method_in_chain(self.db, fqcn_str, "__toString")
                    && !crate::db::extends_or_implements(self.db, fqcn_str, "Stringable")
                {
                    self.emit(
                        IssueKind::ImplicitToStringCast {
                            class: fqcn_str.to_string(),
                        },
                        Severity::Warning,
                        span,
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    /// Helper to create a SourceMap from PHP source code
    fn create_source_map(source: &str) -> php_rs_parser::source_map::SourceMap {
        php_rs_parser::parse(source).source_map
    }

    /// Helper to test offset_to_line_col conversion (Unicode char-count columns).
    fn test_offset_conversion(source: &str, offset: u32) -> (u32, u16) {
        let source_map = create_source_map(source);
        let lc = source_map.offset_to_line_col(offset);
        let line = lc.line + 1;

        let byte_offset = offset as usize;
        let line_start_byte = if byte_offset == 0 {
            0
        } else {
            source[..byte_offset]
                .rfind('\n')
                .map(|p| p + 1)
                .unwrap_or(0)
        };

        let col = source[line_start_byte..byte_offset].chars().count() as u16;

        (line, col)
    }

    #[test]
    fn col_conversion_simple_ascii() {
        let source = "<?php\n$var = 123;";

        // '$' on line 2, column 0
        let (line, col) = test_offset_conversion(source, 6);
        assert_eq!(line, 2);
        assert_eq!(col, 0);

        // 'v' on line 2, column 1
        let (line, col) = test_offset_conversion(source, 7);
        assert_eq!(line, 2);
        assert_eq!(col, 1);
    }

    #[test]
    fn col_conversion_different_lines() {
        let source = "<?php\n$x = 1;\n$y = 2;";
        // Line 1: <?php     (bytes 0-4, newline at 5)
        // Line 2: $x = 1;  (bytes 6-12, newline at 13)
        // Line 3: $y = 2;  (bytes 14-20)

        let (line, col) = test_offset_conversion(source, 0);
        assert_eq!((line, col), (1, 0));

        let (line, col) = test_offset_conversion(source, 6);
        assert_eq!((line, col), (2, 0));

        let (line, col) = test_offset_conversion(source, 14);
        assert_eq!((line, col), (3, 0));
    }

    #[test]
    fn col_conversion_accented_characters() {
        // é is 2 UTF-8 bytes but 1 Unicode char (and 1 UTF-16 unit — same result either way)
        let source = "<?php\n$café = 1;";
        // Line 2: $ c a f é ...
        // bytes:  6 7 8 9 10(2 bytes)

        // 'f' at byte 9 → char col 3
        let (line, col) = test_offset_conversion(source, 9);
        assert_eq!((line, col), (2, 3));

        // 'é' at byte 10 → char col 4
        let (line, col) = test_offset_conversion(source, 10);
        assert_eq!((line, col), (2, 4));
    }

    #[test]
    fn col_conversion_emoji_counts_as_one_char() {
        // 🎉 (U+1F389) is 4 UTF-8 bytes and 2 UTF-16 units, but 1 Unicode char.
        // A char after the emoji must land at col 7, not col 8.
        let source = "<?php\n$y = \"🎉x\";";
        // Line 2: $ y   =   " 🎉 x " ;
        // chars:  0 1 2 3 4 5  6  7 8 9

        let emoji_start = source.find("🎉").unwrap();
        let after_emoji = emoji_start + "🎉".len(); // skip 4 bytes

        // position at 'x' (right after the emoji)
        let (line, col) = test_offset_conversion(source, after_emoji as u32);
        assert_eq!(line, 2);
        assert_eq!(col, 7); // emoji counts as 1, not 2
    }

    #[test]
    fn col_conversion_emoji_start_position() {
        // The opening quote is at col 5; the emoji immediately follows at col 6.
        let source = "<?php\n$y = \"🎉\";";
        // Line 2: $ y   =   " 🎉 " ;
        // chars:  0 1 2 3 4 5  6  7 8

        let quote_pos = source.find('"').unwrap();
        let emoji_pos = quote_pos + 1; // byte after opening quote = emoji start

        let (line, col) = test_offset_conversion(source, quote_pos as u32);
        assert_eq!(line, 2);
        assert_eq!(col, 5); // '"' is the 6th char on line 2 (0-based: col 5)

        let (line, col) = test_offset_conversion(source, emoji_pos as u32);
        assert_eq!(line, 2);
        assert_eq!(col, 6); // emoji follows the quote
    }

    #[test]
    fn col_end_minimum_width() {
        // Same-line span: ensure col_end is at least col_start + 1 (1 character minimum).
        let effective_col_end = crate::diagnostics::clamp_col_end(1, 1, 0, 0);
        assert_eq!(
            effective_col_end, 1,
            "col_end should be at least col_start + 1 on the same line"
        );
    }

    #[test]
    fn col_end_not_clamped_across_lines() {
        // A multi-line span (e.g. a `new Foo(\n ...\n)` call) legitimately has
        // col_end smaller than col_start, since they're columns on different
        // lines. Regression for a bug where `.max(col_start + 1)` was applied
        // unconditionally, producing a nonsensical column past the end of the
        // end line's actual content (see `TooManyArguments` on 0-arg `new` calls).
        let col_start = 11u16; // e.g. "    return new Foo(" — "new" starts at col 11
        let col_end = 5u16; // e.g. "    );" — ")" ends at col 5
        let effective_col_end = crate::diagnostics::clamp_col_end(1, 8, col_start, col_end);

        assert_eq!(
            effective_col_end, col_end,
            "col_end on a different line must not be clamped up to col_start + 1"
        );
    }

    #[test]
    fn col_conversion_multiline_span() {
        // Test span that starts on one line and ends on another
        let source = "<?php\n$x = [\n  'a',\n  'b'\n];";
        //           Line 1: <?php
        //           Line 2: $x = [
        //           Line 3:   'a',
        //           Line 4:   'b'
        //           Line 5: ];

        // Start of array bracket on line 2
        let bracket_open = source.find('[').unwrap();
        let (line_start, _col_start) = test_offset_conversion(source, bracket_open as u32);
        assert_eq!(line_start, 2);

        // End of array bracket on line 5
        let bracket_close = source.rfind(']').unwrap();
        let (line_end, col_end) = test_offset_conversion(source, bracket_close as u32);
        assert_eq!(line_end, 5);
        assert_eq!(col_end, 0); // ']' is at column 0 on line 5
    }

    #[test]
    fn col_end_handles_emoji_in_span() {
        // Test that col_end correctly handles emoji spanning
        let source = "<?php\n$greeting = \"Hello 🎉\";";

        // Find emoji position
        let emoji_pos = source.find('🎉').unwrap();
        let hello_pos = source.find("Hello").unwrap();

        // Column at "Hello" on line 2
        let (line, col) = test_offset_conversion(source, hello_pos as u32);
        assert_eq!(line, 2);
        assert_eq!(col, 13); // Position of 'H' after "$greeting = \""

        // Column at emoji
        let (line, col) = test_offset_conversion(source, emoji_pos as u32);
        assert_eq!(line, 2);
        // Should be after "Hello " (13 + 5 + 1 = 19 chars)
        assert_eq!(col, 19);
    }
}
