/// Expression analyzer — infers the `Union` type of any PHP expression.
use std::sync::Arc;

use php_ast::ast::{
    AssignOp, BinaryOp, CastKind, ExprKind, MagicConstKind, UnaryPostfixOp, UnaryPrefixOp,
};

use mir_codebase::Codebase;
use mir_issues::{Issue, IssueBuffer, IssueKind, Location, Severity};
use mir_types::{Atomic, Union};

use crate::call::CallAnalyzer;
use crate::context::Context;
use crate::symbol::{ResolvedSymbol, SymbolKind};

// ---------------------------------------------------------------------------
// ExpressionAnalyzer
// ---------------------------------------------------------------------------

pub struct ExpressionAnalyzer<'a> {
    pub codebase: &'a Codebase,
    pub file: Arc<str>,
    pub source: &'a str,
    pub source_map: &'a php_ast::source_map::SourceMap,
    pub issues: &'a mut IssueBuffer,
    pub symbols: &'a mut Vec<ResolvedSymbol>,
}

impl<'a> ExpressionAnalyzer<'a> {
    pub fn new(
        codebase: &'a Codebase,
        file: Arc<str>,
        source: &'a str,
        source_map: &'a php_ast::source_map::SourceMap,
        issues: &'a mut IssueBuffer,
        symbols: &'a mut Vec<ResolvedSymbol>,
    ) -> Self {
        Self {
            codebase,
            file,
            source,
            source_map,
            issues,
            symbols,
        }
    }

    /// Record a resolved symbol.
    pub fn record_symbol(&mut self, span: php_ast::Span, kind: SymbolKind, resolved_type: Union) {
        self.symbols.push(ResolvedSymbol {
            file: self.file.clone(),
            span,
            kind,
            resolved_type,
        });
    }

    pub fn analyze<'arena, 'src>(
        &mut self,
        expr: &php_ast::ast::Expr<'arena, 'src>,
        ctx: &mut Context,
    ) -> Union {
        match &expr.kind {
            // --- Literals ---------------------------------------------------
            ExprKind::Int(n) => Union::single(Atomic::TLiteralInt(*n)),
            ExprKind::Float(f) => {
                let bits = f.to_bits();
                Union::single(Atomic::TLiteralFloat(
                    (bits >> 32) as i64,
                    (bits & 0xFFFF_FFFF) as i64,
                ))
            }
            ExprKind::String(s) => Union::single(Atomic::TLiteralString((*s).into())),
            ExprKind::Bool(b) => {
                if *b {
                    Union::single(Atomic::TTrue)
                } else {
                    Union::single(Atomic::TFalse)
                }
            }
            ExprKind::Null => Union::single(Atomic::TNull),

            // Interpolated strings always produce TString
            ExprKind::InterpolatedString(parts) | ExprKind::Heredoc { parts, .. } => {
                for part in parts.iter() {
                    if let php_ast::StringPart::Expr(e) = part {
                        self.analyze(e, ctx);
                    }
                }
                Union::single(Atomic::TString)
            }

            ExprKind::Nowdoc { .. } => Union::single(Atomic::TString),
            ExprKind::ShellExec(_) => Union::single(Atomic::TString),

            // --- Variables --------------------------------------------------
            ExprKind::Variable(name) => {
                let name_str = name.as_ref().trim_start_matches('$');
                if !ctx.var_is_defined(name_str) {
                    if ctx.var_possibly_defined(name_str) {
                        self.emit(
                            IssueKind::PossiblyUndefinedVariable {
                                name: name_str.to_string(),
                            },
                            Severity::Info,
                            expr.span,
                        );
                    } else if name_str != "this" {
                        self.emit(
                            IssueKind::UndefinedVariable {
                                name: name_str.to_string(),
                            },
                            Severity::Error,
                            expr.span,
                        );
                    }
                }
                ctx.read_vars.insert(name_str.to_string());
                let ty = ctx.get_var(name_str);
                self.record_symbol(
                    expr.span,
                    SymbolKind::Variable(name_str.to_string()),
                    ty.clone(),
                );
                ty
            }

            ExprKind::VariableVariable(_) => Union::mixed(), // $$x — unknowable

            ExprKind::Identifier(_name) => {
                // Bare identifier used as value (e.g. class constant, global const)
                Union::mixed()
            }

            // --- Assignment -------------------------------------------------
            ExprKind::Assign(a) => {
                let rhs_tainted = crate::taint::is_expr_tainted(a.value, ctx);
                let rhs_ty = self.analyze(a.value, ctx);
                match a.op {
                    AssignOp::Assign => {
                        self.assign_to_target(a.target, rhs_ty.clone(), ctx, expr.span);
                        // Propagate taint: if RHS is tainted, taint LHS variable (M19)
                        if rhs_tainted {
                            if let ExprKind::Variable(name) = &a.target.kind {
                                ctx.taint_var(name.as_ref());
                            }
                        }
                        rhs_ty
                    }
                    AssignOp::Concat => {
                        // .= always produces string
                        if let Some(var_name) = extract_simple_var(a.target) {
                            ctx.set_var(&var_name, Union::single(Atomic::TString));
                        }
                        Union::single(Atomic::TString)
                    }
                    AssignOp::Plus
                    | AssignOp::Minus
                    | AssignOp::Mul
                    | AssignOp::Div
                    | AssignOp::Mod
                    | AssignOp::Pow => {
                        let lhs_ty = self.analyze(a.target, ctx);
                        let result_ty = infer_arithmetic(&lhs_ty, &rhs_ty);
                        if let Some(var_name) = extract_simple_var(a.target) {
                            ctx.set_var(&var_name, result_ty.clone());
                        }
                        result_ty
                    }
                    AssignOp::Coalesce => {
                        // ??= — assign only if null
                        let lhs_ty = self.analyze(a.target, ctx);
                        let merged = Union::merge(&lhs_ty.remove_null(), &rhs_ty);
                        if let Some(var_name) = extract_simple_var(a.target) {
                            ctx.set_var(&var_name, merged.clone());
                        }
                        merged
                    }
                    _ => {
                        if let Some(var_name) = extract_simple_var(a.target) {
                            ctx.set_var(&var_name, Union::mixed());
                        }
                        Union::mixed()
                    }
                }
            }

            // --- Binary operations ------------------------------------------
            ExprKind::Binary(b) => self.analyze_binary(b, expr.span, ctx),

            // --- Unary ------------------------------------------------------
            ExprKind::UnaryPrefix(u) => {
                let operand_ty = self.analyze(u.operand, ctx);
                match u.op {
                    UnaryPrefixOp::BooleanNot => Union::single(Atomic::TBool),
                    UnaryPrefixOp::Negate => {
                        if operand_ty.contains(|t| t.is_int()) {
                            Union::single(Atomic::TInt)
                        } else {
                            Union::single(Atomic::TFloat)
                        }
                    }
                    UnaryPrefixOp::Plus => operand_ty,
                    UnaryPrefixOp::BitwiseNot => Union::single(Atomic::TInt),
                    UnaryPrefixOp::PreIncrement | UnaryPrefixOp::PreDecrement => {
                        // ++$x / --$x: increment and return new value
                        if let Some(var_name) = extract_simple_var(u.operand) {
                            let ty = ctx.get_var(&var_name);
                            let new_ty = if ty.contains(|t| {
                                matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..))
                            }) {
                                Union::single(Atomic::TFloat)
                            } else {
                                Union::single(Atomic::TInt)
                            };
                            ctx.set_var(&var_name, new_ty.clone());
                            new_ty
                        } else {
                            Union::single(Atomic::TInt)
                        }
                    }
                }
            }

            ExprKind::UnaryPostfix(u) => {
                let operand_ty = self.analyze(u.operand, ctx);
                // $x++ / $x-- returns original value, but mutates variable
                match u.op {
                    UnaryPostfixOp::PostIncrement | UnaryPostfixOp::PostDecrement => {
                        if let Some(var_name) = extract_simple_var(u.operand) {
                            let new_ty = if operand_ty.contains(|t| {
                                matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..))
                            }) {
                                Union::single(Atomic::TFloat)
                            } else {
                                Union::single(Atomic::TInt)
                            };
                            ctx.set_var(&var_name, new_ty);
                        }
                        operand_ty // returns original value
                    }
                }
            }

            // --- Ternary / null coalesce ------------------------------------
            ExprKind::Ternary(t) => {
                let cond_ty = self.analyze(t.condition, ctx);
                match &t.then_expr {
                    Some(then_expr) => {
                        let mut then_ctx = ctx.fork();
                        crate::narrowing::narrow_from_condition(
                            t.condition,
                            &mut then_ctx,
                            true,
                            self.codebase,
                            &self.file,
                        );
                        let then_ty =
                            self.with_ctx(&mut then_ctx, |ea, c| ea.analyze(then_expr, c));

                        let mut else_ctx = ctx.fork();
                        crate::narrowing::narrow_from_condition(
                            t.condition,
                            &mut else_ctx,
                            false,
                            self.codebase,
                            &self.file,
                        );
                        let else_ty =
                            self.with_ctx(&mut else_ctx, |ea, c| ea.analyze(t.else_expr, c));

                        // Propagate variable reads from both branches
                        for name in then_ctx.read_vars.iter().chain(else_ctx.read_vars.iter()) {
                            ctx.read_vars.insert(name.clone());
                        }

                        Union::merge(&then_ty, &else_ty)
                    }
                    None => {
                        // $x ?: $y — short ternary: if $x truthy, return $x; else return $y
                        let else_ty = self.analyze(t.else_expr, ctx);
                        let truthy_ty = cond_ty.narrow_to_truthy();
                        if truthy_ty.is_empty() {
                            else_ty
                        } else {
                            Union::merge(&truthy_ty, &else_ty)
                        }
                    }
                }
            }

            ExprKind::NullCoalesce(nc) => {
                let left_ty = self.analyze(nc.left, ctx);
                let right_ty = self.analyze(nc.right, ctx);
                // result = remove_null(left) | right
                let non_null_left = left_ty.remove_null();
                if non_null_left.is_empty() {
                    right_ty
                } else {
                    Union::merge(&non_null_left, &right_ty)
                }
            }

            // --- Casts ------------------------------------------------------
            ExprKind::Cast(kind, inner) => {
                let _inner_ty = self.analyze(inner, ctx);
                match kind {
                    CastKind::Int => Union::single(Atomic::TInt),
                    CastKind::Float => Union::single(Atomic::TFloat),
                    CastKind::String => Union::single(Atomic::TString),
                    CastKind::Bool => Union::single(Atomic::TBool),
                    CastKind::Array => Union::single(Atomic::TArray {
                        key: Box::new(Union::single(Atomic::TMixed)),
                        value: Box::new(Union::mixed()),
                    }),
                    CastKind::Object => Union::single(Atomic::TObject),
                    CastKind::Unset | CastKind::Void => Union::single(Atomic::TNull),
                }
            }

            // --- Error suppression ------------------------------------------
            ExprKind::ErrorSuppress(inner) => self.analyze(inner, ctx),

            // --- Parenthesized ----------------------------------------------
            ExprKind::Parenthesized(inner) => self.analyze(inner, ctx),

            // --- Array literals ---------------------------------------------
            ExprKind::Array(elements) => {
                use mir_types::atomic::{ArrayKey, KeyedProperty};

                if elements.is_empty() {
                    return Union::single(Atomic::TKeyedArray {
                        properties: indexmap::IndexMap::new(),
                        is_open: false,
                        is_list: true,
                    });
                }

                // Try to build a TKeyedArray when all keys are literal strings/ints
                // (or no keys — pure list). Fall back to TArray on spread or dynamic keys.
                let mut keyed_props: indexmap::IndexMap<ArrayKey, KeyedProperty> =
                    indexmap::IndexMap::new();
                let mut is_list = true;
                let mut can_be_keyed = true;
                let mut next_int_key: i64 = 0;

                for elem in elements.iter() {
                    if elem.unpack {
                        self.analyze(&elem.value, ctx);
                        can_be_keyed = false;
                        break;
                    }
                    let value_ty = self.analyze(&elem.value, ctx);
                    let array_key = if let Some(key_expr) = &elem.key {
                        is_list = false;
                        let key_ty = self.analyze(key_expr, ctx);
                        // Only build keyed array if key is a string or int literal
                        match key_ty.types.as_slice() {
                            [Atomic::TLiteralString(s)] => ArrayKey::String(s.clone()),
                            [Atomic::TLiteralInt(i)] => {
                                next_int_key = *i + 1;
                                ArrayKey::Int(*i)
                            }
                            _ => {
                                can_be_keyed = false;
                                break;
                            }
                        }
                    } else {
                        let k = ArrayKey::Int(next_int_key);
                        next_int_key += 1;
                        k
                    };
                    keyed_props.insert(
                        array_key,
                        KeyedProperty {
                            ty: value_ty,
                            optional: false,
                        },
                    );
                }

                if can_be_keyed {
                    return Union::single(Atomic::TKeyedArray {
                        properties: keyed_props,
                        is_open: false,
                        is_list,
                    });
                }

                // Fallback: generic TArray — re-evaluate elements to build merged types
                let mut all_value_types = Union::empty();
                let mut key_union = Union::empty();
                let mut has_unpack = false;
                for elem in elements.iter() {
                    let value_ty = self.analyze(&elem.value, ctx);
                    if elem.unpack {
                        has_unpack = true;
                    } else {
                        all_value_types = Union::merge(&all_value_types, &value_ty);
                        if let Some(key_expr) = &elem.key {
                            let key_ty = self.analyze(key_expr, ctx);
                            key_union = Union::merge(&key_union, &key_ty);
                        } else {
                            key_union.add_type(Atomic::TInt);
                        }
                    }
                }
                if has_unpack {
                    return Union::single(Atomic::TArray {
                        key: Box::new(Union::single(Atomic::TMixed)),
                        value: Box::new(Union::mixed()),
                    });
                }
                if key_union.is_empty() {
                    key_union.add_type(Atomic::TInt);
                }
                Union::single(Atomic::TArray {
                    key: Box::new(key_union),
                    value: Box::new(all_value_types),
                })
            }

            // --- Array access -----------------------------------------------
            ExprKind::ArrayAccess(aa) => {
                let arr_ty = self.analyze(aa.array, ctx);

                // Analyze the index expression for variable read tracking
                if let Some(idx) = &aa.index {
                    self.analyze(idx, ctx);
                }

                // Check for null access
                if arr_ty.contains(|t| matches!(t, Atomic::TNull)) && arr_ty.is_single() {
                    self.emit(IssueKind::NullArrayAccess, Severity::Error, expr.span);
                    return Union::mixed();
                }
                if arr_ty.is_nullable() {
                    self.emit(
                        IssueKind::PossiblyNullArrayAccess,
                        Severity::Info,
                        expr.span,
                    );
                }

                // Determine the key being accessed (if it's a literal)
                let literal_key: Option<mir_types::atomic::ArrayKey> =
                    aa.index.as_ref().and_then(|idx| match &idx.kind {
                        ExprKind::String(s) => {
                            Some(mir_types::atomic::ArrayKey::String(Arc::from(&**s)))
                        }
                        ExprKind::Int(i) => Some(mir_types::atomic::ArrayKey::Int(*i)),
                        _ => None,
                    });

                // Infer element type
                for atomic in &arr_ty.types {
                    match atomic {
                        Atomic::TKeyedArray { properties, .. } => {
                            // If we know the key, look it up precisely
                            if let Some(ref key) = literal_key {
                                if let Some(prop) = properties.get(key) {
                                    return prop.ty.clone();
                                }
                            }
                            // Unknown key — return union of all value types
                            let mut result = Union::empty();
                            for prop in properties.values() {
                                result = Union::merge(&result, &prop.ty);
                            }
                            return if result.types.is_empty() {
                                Union::mixed()
                            } else {
                                result
                            };
                        }
                        Atomic::TArray { value, .. } | Atomic::TNonEmptyArray { value, .. } => {
                            return *value.clone();
                        }
                        Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
                            return *value.clone();
                        }
                        Atomic::TString | Atomic::TLiteralString(_) => {
                            return Union::single(Atomic::TString);
                        }
                        _ => {}
                    }
                }
                Union::mixed()
            }

            // --- isset / empty ----------------------------------------------
            ExprKind::Isset(exprs) => {
                for e in exprs.iter() {
                    self.analyze(e, ctx);
                }
                Union::single(Atomic::TBool)
            }
            ExprKind::Empty(inner) => {
                self.analyze(inner, ctx);
                Union::single(Atomic::TBool)
            }

            // --- print ------------------------------------------------------
            ExprKind::Print(inner) => {
                self.analyze(inner, ctx);
                Union::single(Atomic::TLiteralInt(1))
            }

            // --- clone ------------------------------------------------------
            ExprKind::Clone(inner) => self.analyze(inner, ctx),
            ExprKind::CloneWith(inner, _props) => self.analyze(inner, ctx),

            // --- new ClassName(...) ----------------------------------------
            ExprKind::New(n) => {
                // Evaluate args first (needed for taint / type check)
                let arg_types: Vec<Union> = n
                    .args
                    .iter()
                    .map(|a| {
                        let ty = self.analyze(&a.value, ctx);
                        if a.unpack {
                            crate::call::spread_element_type(&ty)
                        } else {
                            ty
                        }
                    })
                    .collect();
                let arg_spans: Vec<php_ast::Span> = n.args.iter().map(|a| a.span).collect();
                let arg_names: Vec<Option<String>> = n
                    .args
                    .iter()
                    .map(|a| a.name.as_ref().map(|nm| nm.to_string()))
                    .collect();

                let class_ty = match &n.class.kind {
                    ExprKind::Identifier(name) => {
                        let resolved = self.codebase.resolve_class_name(&self.file, name.as_ref());
                        // `self`, `static`, `parent` resolve to the current class — use ctx
                        let fqcn: Arc<str> = match resolved.as_str() {
                            "self" | "static" => ctx
                                .self_fqcn
                                .clone()
                                .or_else(|| ctx.static_fqcn.clone())
                                .unwrap_or_else(|| Arc::from(resolved.as_str())),
                            "parent" => ctx
                                .parent_fqcn
                                .clone()
                                .unwrap_or_else(|| Arc::from(resolved.as_str())),
                            _ => Arc::from(resolved.as_str()),
                        };
                        if !matches!(resolved.as_str(), "self" | "static" | "parent")
                            && !self.codebase.type_exists(&fqcn)
                        {
                            self.emit(
                                IssueKind::UndefinedClass {
                                    name: resolved.clone(),
                                },
                                Severity::Error,
                                n.class.span,
                            );
                        } else if self.codebase.type_exists(&fqcn) {
                            // Check constructor arguments
                            if let Some(ctor) = self.codebase.get_method(&fqcn, "__construct") {
                                crate::call::check_constructor_args(
                                    self,
                                    &fqcn,
                                    crate::call::CheckArgsParams {
                                        fn_name: "__construct",
                                        params: &ctor.params,
                                        arg_types: &arg_types,
                                        arg_spans: &arg_spans,
                                        arg_names: &arg_names,
                                        call_span: expr.span,
                                        has_spread: n.args.iter().any(|a| a.unpack),
                                    },
                                );
                            }
                        }
                        let ty = Union::single(Atomic::TNamedObject {
                            fqcn: fqcn.clone(),
                            type_params: vec![],
                        });
                        self.record_symbol(
                            n.class.span,
                            SymbolKind::ClassReference(fqcn),
                            ty.clone(),
                        );
                        ty
                    }
                    _ => {
                        self.analyze(n.class, ctx);
                        Union::single(Atomic::TObject)
                    }
                };
                class_ty
            }

            ExprKind::AnonymousClass(_) => Union::single(Atomic::TObject),

            // --- Property access -------------------------------------------
            ExprKind::PropertyAccess(pa) => {
                let obj_ty = self.analyze(pa.object, ctx);
                let prop_name = extract_string_from_expr(pa.property)
                    .unwrap_or_else(|| "<dynamic>".to_string());

                if obj_ty.contains(|t| matches!(t, Atomic::TNull)) && obj_ty.is_single() {
                    self.emit(
                        IssueKind::NullPropertyFetch {
                            property: prop_name.clone(),
                        },
                        Severity::Error,
                        expr.span,
                    );
                    return Union::mixed();
                }
                if obj_ty.is_nullable() {
                    self.emit(
                        IssueKind::PossiblyNullPropertyFetch {
                            property: prop_name.clone(),
                        },
                        Severity::Info,
                        expr.span,
                    );
                }

                // Dynamic property access ($obj->$varName) — can't resolve statically.
                if prop_name == "<dynamic>" {
                    return Union::mixed();
                }
                let resolved = self.resolve_property_type(&obj_ty, &prop_name, expr.span);
                // Record property access symbol for each named object in the receiver type
                for atomic in &obj_ty.types {
                    if let Atomic::TNamedObject { fqcn, .. } = atomic {
                        self.record_symbol(
                            expr.span,
                            SymbolKind::PropertyAccess {
                                class: fqcn.clone(),
                                property: Arc::from(prop_name.as_str()),
                            },
                            resolved.clone(),
                        );
                        break;
                    }
                }
                resolved
            }

            ExprKind::NullsafePropertyAccess(pa) => {
                let obj_ty = self.analyze(pa.object, ctx);
                let prop_name = extract_string_from_expr(pa.property)
                    .unwrap_or_else(|| "<dynamic>".to_string());
                if prop_name == "<dynamic>" {
                    return Union::mixed();
                }
                // ?-> strips null from receiver
                let non_null_ty = obj_ty.remove_null();
                let mut prop_ty = self.resolve_property_type(&non_null_ty, &prop_name, expr.span);
                prop_ty.add_type(Atomic::TNull); // result is nullable because receiver may be null
                prop_ty
            }

            ExprKind::StaticPropertyAccess(_spa) => {
                // Class::$prop
                Union::mixed()
            }

            ExprKind::ClassConstAccess(cca) => {
                // Foo::CONST or Foo::class
                if cca.member.as_ref() == "class" {
                    // Resolve the class name so Foo::class gives the correct FQCN string
                    let fqcn = if let ExprKind::Identifier(id) = &cca.class.kind {
                        let resolved = self.codebase.resolve_class_name(&self.file, id.as_ref());
                        Some(Arc::from(resolved.as_str()))
                    } else {
                        None
                    };
                    return Union::single(Atomic::TClassString(fqcn));
                }
                Union::mixed()
            }

            ExprKind::ClassConstAccessDynamic { .. } => Union::mixed(),
            ExprKind::StaticPropertyAccessDynamic { .. } => Union::mixed(),

            // --- Method calls ----------------------------------------------
            ExprKind::MethodCall(mc) => {
                CallAnalyzer::analyze_method_call(self, mc, ctx, expr.span, false)
            }

            ExprKind::NullsafeMethodCall(mc) => {
                CallAnalyzer::analyze_method_call(self, mc, ctx, expr.span, true)
            }

            ExprKind::StaticMethodCall(smc) => {
                CallAnalyzer::analyze_static_method_call(self, smc, ctx, expr.span)
            }

            // --- Function calls --------------------------------------------
            ExprKind::FunctionCall(fc) => {
                CallAnalyzer::analyze_function_call(self, fc, ctx, expr.span)
            }

            // --- Closures / arrow functions --------------------------------
            ExprKind::Closure(c) => {
                let params = ast_params_to_fn_params_resolved(
                    &c.params,
                    ctx.self_fqcn.as_deref(),
                    self.codebase,
                    &self.file,
                );
                let return_ty_hint = c
                    .return_type
                    .as_ref()
                    .map(|h| crate::parser::type_from_hint(h, ctx.self_fqcn.as_deref()))
                    .map(|u| resolve_named_objects_in_union(u, self.codebase, &self.file));

                // Build closure context — capture declared use-vars from outer scope
                // Note: is_static only prevents $this binding; self_fqcn is still accessible
                // for resolving `self::` references and private/protected visibility checks.
                let mut closure_ctx = crate::context::Context::for_function(
                    &params,
                    return_ty_hint.clone(),
                    ctx.self_fqcn.clone(),
                    ctx.parent_fqcn.clone(),
                    ctx.static_fqcn.clone(),
                    ctx.strict_types,
                );
                for use_var in c.use_vars.iter() {
                    let name = use_var.name.trim_start_matches('$');
                    closure_ctx.set_var(name, ctx.get_var(name));
                    if ctx.is_tainted(name) {
                        closure_ctx.taint_var(name);
                    }
                }

                // Analyze closure body, collecting issues into the same buffer
                let inferred_return = {
                    let mut sa = crate::stmt::StatementsAnalyzer::new(
                        self.codebase,
                        self.file.clone(),
                        self.source,
                        self.source_map,
                        self.issues,
                        self.symbols,
                    );
                    sa.analyze_stmts(&c.body, &mut closure_ctx);
                    let ret = crate::project::merge_return_types(&sa.return_types);
                    drop(sa);
                    ret
                };

                // Propagate variable reads from closure back to outer scope
                for name in &closure_ctx.read_vars {
                    ctx.read_vars.insert(name.clone());
                }

                let return_ty = return_ty_hint.unwrap_or(inferred_return);
                let closure_params: Vec<mir_types::atomic::FnParam> = params
                    .iter()
                    .map(|p| mir_types::atomic::FnParam {
                        name: p.name.clone(),
                        ty: p.ty.clone(),
                        default: p.default.clone(),
                        is_variadic: p.is_variadic,
                        is_byref: p.is_byref,
                        is_optional: p.is_optional,
                    })
                    .collect();

                Union::single(Atomic::TClosure {
                    params: closure_params,
                    return_type: Box::new(return_ty),
                    this_type: ctx.self_fqcn.clone().map(|f| {
                        Box::new(Union::single(Atomic::TNamedObject {
                            fqcn: f,
                            type_params: vec![],
                        }))
                    }),
                })
            }

            ExprKind::ArrowFunction(af) => {
                let params = ast_params_to_fn_params_resolved(
                    &af.params,
                    ctx.self_fqcn.as_deref(),
                    self.codebase,
                    &self.file,
                );
                let return_ty_hint = af
                    .return_type
                    .as_ref()
                    .map(|h| crate::parser::type_from_hint(h, ctx.self_fqcn.as_deref()))
                    .map(|u| resolve_named_objects_in_union(u, self.codebase, &self.file));

                // Arrow functions implicitly capture the outer scope by value
                // Note: is_static only prevents $this binding; self_fqcn is still accessible
                // for resolving `self::` references and private/protected visibility checks.
                let mut arrow_ctx = crate::context::Context::for_function(
                    &params,
                    return_ty_hint.clone(),
                    ctx.self_fqcn.clone(),
                    ctx.parent_fqcn.clone(),
                    ctx.static_fqcn.clone(),
                    ctx.strict_types,
                );
                // Copy outer vars into arrow context (implicit capture)
                for (name, ty) in &ctx.vars {
                    if !arrow_ctx.vars.contains_key(name) {
                        arrow_ctx.set_var(name, ty.clone());
                    }
                }

                // Analyze single-expression body
                let inferred_return = self.analyze(af.body, &mut arrow_ctx);

                // Propagate variable reads from arrow function back to outer scope
                for name in &arrow_ctx.read_vars {
                    ctx.read_vars.insert(name.clone());
                }

                let return_ty = return_ty_hint.unwrap_or(inferred_return);
                let closure_params: Vec<mir_types::atomic::FnParam> = params
                    .iter()
                    .map(|p| mir_types::atomic::FnParam {
                        name: p.name.clone(),
                        ty: p.ty.clone(),
                        default: p.default.clone(),
                        is_variadic: p.is_variadic,
                        is_byref: p.is_byref,
                        is_optional: p.is_optional,
                    })
                    .collect();

                Union::single(Atomic::TClosure {
                    params: closure_params,
                    return_type: Box::new(return_ty),
                    this_type: if af.is_static {
                        None
                    } else {
                        ctx.self_fqcn.clone().map(|f| {
                            Box::new(Union::single(Atomic::TNamedObject {
                                fqcn: f,
                                type_params: vec![],
                            }))
                        })
                    },
                })
            }

            ExprKind::CallableCreate(_) => Union::single(Atomic::TCallable {
                params: None,
                return_type: None,
            }),

            // --- Match expression ------------------------------------------
            ExprKind::Match(m) => {
                let subject_ty = self.analyze(m.subject, ctx);
                // Extract the variable name of the subject for narrowing
                let subject_var = match &m.subject.kind {
                    ExprKind::Variable(name) => {
                        Some(name.as_ref().trim_start_matches('$').to_string())
                    }
                    _ => None,
                };

                let mut result = Union::empty();
                for arm in m.arms.iter() {
                    // Fork context for each arm so arms don't bleed into each other
                    let mut arm_ctx = ctx.fork();

                    // Narrow the subject variable in this arm's context
                    if let (Some(var), Some(conditions)) = (&subject_var, &arm.conditions) {
                        // Build a union of all condition types for this arm
                        let mut arm_ty = Union::empty();
                        for cond in conditions.iter() {
                            let cond_ty = self.analyze(cond, ctx);
                            arm_ty = Union::merge(&arm_ty, &cond_ty);
                        }
                        // Intersect subject type with the arm condition types
                        if !arm_ty.is_empty() && !arm_ty.is_mixed() {
                            // Narrow to the matched literal/type if possible
                            let narrowed = subject_ty.intersect_with(&arm_ty);
                            if !narrowed.is_empty() {
                                arm_ctx.set_var(var, narrowed);
                            }
                        }
                    }

                    // For `match(true) { $x instanceof Y => ... }` patterns:
                    // narrow from each condition expression even when subject is not a simple var.
                    if let Some(conditions) = &arm.conditions {
                        for cond in conditions.iter() {
                            crate::narrowing::narrow_from_condition(
                                cond,
                                &mut arm_ctx,
                                true,
                                self.codebase,
                                &self.file,
                            );
                        }
                    }

                    let arm_body_ty = self.analyze(&arm.body, &mut arm_ctx);
                    result = Union::merge(&result, &arm_body_ty);

                    // Propagate variable reads from arm back to outer scope
                    for name in &arm_ctx.read_vars {
                        ctx.read_vars.insert(name.clone());
                    }
                }
                if result.is_empty() {
                    Union::mixed()
                } else {
                    result
                }
            }

            // --- Throw as expression (PHP 8) --------------------------------
            ExprKind::ThrowExpr(e) => {
                self.analyze(e, ctx);
                Union::single(Atomic::TNever)
            }

            // --- Yield -----------------------------------------------------
            ExprKind::Yield(y) => {
                if let Some(key) = &y.key {
                    self.analyze(key, ctx);
                }
                if let Some(value) = &y.value {
                    self.analyze(value, ctx);
                }
                Union::mixed()
            }

            // --- Magic constants -------------------------------------------
            ExprKind::MagicConst(kind) => match kind {
                MagicConstKind::Line => Union::single(Atomic::TInt),
                MagicConstKind::File
                | MagicConstKind::Dir
                | MagicConstKind::Function
                | MagicConstKind::Class
                | MagicConstKind::Method
                | MagicConstKind::Namespace
                | MagicConstKind::Trait
                | MagicConstKind::Property => Union::single(Atomic::TString),
            },

            // --- Include/require --------------------------------------------
            ExprKind::Include(_, inner) => {
                self.analyze(inner, ctx);
                Union::mixed()
            }

            // --- Eval -------------------------------------------------------
            ExprKind::Eval(inner) => {
                self.analyze(inner, ctx);
                Union::mixed()
            }

            // --- Exit -------------------------------------------------------
            ExprKind::Exit(opt) => {
                if let Some(e) = opt {
                    self.analyze(e, ctx);
                }
                Union::single(Atomic::TNever)
            }

            // --- Error node (parse error placeholder) ----------------------
            ExprKind::Error => Union::mixed(),

            // --- Omitted array slot (e.g. [, $b] destructuring) ------------
            ExprKind::Omit => Union::single(Atomic::TNull),
        }
    }

    // -----------------------------------------------------------------------
    // Binary operations
    // -----------------------------------------------------------------------

    fn analyze_binary<'arena, 'src>(
        &mut self,
        b: &php_ast::ast::BinaryExpr<'arena, 'src>,
        _span: php_ast::Span,
        ctx: &mut Context,
    ) -> Union {
        // Short-circuit operators: narrow the context for the right operand based on
        // the left operand's truthiness (just like the then/else branches of an if).
        // We evaluate the right side in a forked context so that the narrowing
        // (e.g. `instanceof`) applies to method/property calls on the right side
        // without permanently mutating the caller's context.
        use php_ast::ast::BinaryOp as B;
        if matches!(
            b.op,
            B::BooleanAnd | B::LogicalAnd | B::BooleanOr | B::LogicalOr
        ) {
            let _left_ty = self.analyze(b.left, ctx);
            let mut right_ctx = ctx.fork();
            let is_and = matches!(b.op, B::BooleanAnd | B::LogicalAnd);
            crate::narrowing::narrow_from_condition(
                b.left,
                &mut right_ctx,
                is_and,
                self.codebase,
                &self.file,
            );
            // If narrowing made the right side statically unreachable, skip it
            // (e.g. `$x === null || $x->method()` — right is dead when $x is only null).
            if !right_ctx.diverges {
                let _right_ty = self.analyze(b.right, &mut right_ctx);
            }
            // Propagate read-var tracking and any new variable assignments back.
            // New assignments from the right side are only "possibly" made (short-circuit),
            // so mark them in possibly_assigned_vars but not assigned_vars.
            for v in right_ctx.read_vars {
                ctx.read_vars.insert(v.clone());
            }
            for (name, ty) in &right_ctx.vars {
                if !ctx.vars.contains_key(name.as_str()) {
                    // Variable first assigned in the right side — possibly assigned
                    ctx.vars.insert(name.clone(), ty.clone());
                    ctx.possibly_assigned_vars.insert(name.clone());
                }
            }
            return Union::single(Atomic::TBool);
        }

        let left_ty = self.analyze(b.left, ctx);
        let right_ty = self.analyze(b.right, ctx);

        match b.op {
            // Arithmetic
            BinaryOp::Add
            | BinaryOp::Sub
            | BinaryOp::Mul
            | BinaryOp::Div
            | BinaryOp::Mod
            | BinaryOp::Pow => infer_arithmetic(&left_ty, &right_ty),

            // String concatenation
            BinaryOp::Concat => Union::single(Atomic::TString),

            // Comparisons always return bool
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Identical
            | BinaryOp::NotIdentical
            | BinaryOp::Less
            | BinaryOp::Greater
            | BinaryOp::LessOrEqual
            | BinaryOp::GreaterOrEqual => Union::single(Atomic::TBool),

            BinaryOp::Instanceof => {
                // Check that the class on the right side of `instanceof` exists.
                if let ExprKind::Identifier(name) = &b.right.kind {
                    let resolved = self.codebase.resolve_class_name(&self.file, name.as_ref());
                    let fqcn: std::sync::Arc<str> = std::sync::Arc::from(resolved.as_str());
                    if !matches!(resolved.as_str(), "self" | "static" | "parent")
                        && !self.codebase.type_exists(&fqcn)
                    {
                        self.emit(
                            IssueKind::UndefinedClass { name: resolved },
                            Severity::Error,
                            b.right.span,
                        );
                    }
                }
                Union::single(Atomic::TBool)
            }

            // Spaceship returns -1|0|1
            BinaryOp::Spaceship => Union::single(Atomic::TIntRange {
                min: Some(-1),
                max: Some(1),
            }),

            // Logical
            BinaryOp::BooleanAnd
            | BinaryOp::BooleanOr
            | BinaryOp::LogicalAnd
            | BinaryOp::LogicalOr
            | BinaryOp::LogicalXor => Union::single(Atomic::TBool),

            // Bitwise
            BinaryOp::BitwiseAnd
            | BinaryOp::BitwiseOr
            | BinaryOp::BitwiseXor
            | BinaryOp::ShiftLeft
            | BinaryOp::ShiftRight => Union::single(Atomic::TInt),

            // Pipe (FirstClassCallable-style) — rare
            BinaryOp::Pipe => right_ty,
        }
    }

    // -----------------------------------------------------------------------
    // Property resolution
    // -----------------------------------------------------------------------

    fn resolve_property_type(
        &mut self,
        obj_ty: &Union,
        prop_name: &str,
        span: php_ast::Span,
    ) -> Union {
        for atomic in &obj_ty.types {
            match atomic {
                Atomic::TNamedObject { fqcn, .. } => {
                    if self.codebase.classes.contains_key(fqcn.as_ref()) {
                        if let Some(prop) = self.codebase.get_property(fqcn.as_ref(), prop_name) {
                            // Record reference for dead-code detection (M18)
                            self.codebase.mark_property_referenced(fqcn, prop_name);
                            return prop.ty.clone().unwrap_or_else(Union::mixed);
                        }
                        // Only emit UndefinedProperty if all ancestors are known and no __get magic.
                        if !self.codebase.has_unknown_ancestor(fqcn.as_ref())
                            && !self.codebase.has_magic_get(fqcn.as_ref())
                        {
                            self.emit(
                                IssueKind::UndefinedProperty {
                                    class: fqcn.to_string(),
                                    property: prop_name.to_string(),
                                },
                                Severity::Warning,
                                span,
                            );
                        }
                        return Union::mixed();
                    }
                    // Class not in codebase (external/vendor) — skip silently.
                }
                Atomic::TMixed => return Union::mixed(),
                _ => {}
            }
        }
        Union::mixed()
    }

    // -----------------------------------------------------------------------
    // Assignment helpers
    // -----------------------------------------------------------------------

    fn assign_to_target<'arena, 'src>(
        &mut self,
        target: &php_ast::ast::Expr<'arena, 'src>,
        ty: Union,
        ctx: &mut Context,
        span: php_ast::Span,
    ) {
        match &target.kind {
            ExprKind::Variable(name) => {
                let name_str = name.as_ref().trim_start_matches('$').to_string();
                ctx.set_var(name_str, ty);
            }
            ExprKind::Array(elements) => {
                // [$a, $b] = $arr  — destructuring
                // If the RHS can be false/null (e.g. unpack() returns array|false),
                // the destructuring may fail → PossiblyInvalidArrayAccess.
                let has_non_array = ty.contains(|a| matches!(a, Atomic::TFalse | Atomic::TNull));
                let has_array = ty.contains(|a| {
                    matches!(
                        a,
                        Atomic::TArray { .. }
                            | Atomic::TList { .. }
                            | Atomic::TNonEmptyArray { .. }
                            | Atomic::TNonEmptyList { .. }
                            | Atomic::TKeyedArray { .. }
                    )
                });
                if has_non_array && has_array {
                    let actual = format!("{}", ty);
                    self.emit(
                        IssueKind::PossiblyInvalidArrayOffset {
                            expected: "array".to_string(),
                            actual,
                        },
                        Severity::Warning,
                        span,
                    );
                }

                // Extract the element value type from the RHS array type (if known).
                let value_ty: Union = ty
                    .types
                    .iter()
                    .find_map(|a| match a {
                        Atomic::TArray { value, .. }
                        | Atomic::TList { value }
                        | Atomic::TNonEmptyArray { value, .. }
                        | Atomic::TNonEmptyList { value } => Some(*value.clone()),
                        _ => None,
                    })
                    .unwrap_or_else(Union::mixed);

                for elem in elements.iter() {
                    self.assign_to_target(&elem.value, value_ty.clone(), ctx, span);
                }
            }
            ExprKind::PropertyAccess(pa) => {
                // Check readonly (M19 readonly enforcement)
                let obj_ty = self.analyze(pa.object, ctx);
                if let Some(prop_name) = extract_string_from_expr(pa.property) {
                    for atomic in &obj_ty.types {
                        if let Atomic::TNamedObject { fqcn, .. } = atomic {
                            if let Some(cls) = self.codebase.classes.get(fqcn.as_ref()) {
                                if let Some(prop) = cls.get_property(&prop_name) {
                                    if prop.is_readonly && !ctx.inside_constructor {
                                        self.emit(
                                            IssueKind::ReadonlyPropertyAssignment {
                                                class: fqcn.to_string(),
                                                property: prop_name.clone(),
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
            }
            ExprKind::StaticPropertyAccess(_) => {
                // static property assignment — could add readonly check here too
            }
            ExprKind::ArrayAccess(aa) => {
                // $arr[$k] = v  — PHP auto-initialises $arr as an array if undefined.
                // Analyze the index expression for variable read tracking.
                if let Some(idx) = &aa.index {
                    self.analyze(idx, ctx);
                }
                // Walk the base to find the root variable and update its type to include
                // the new value, so loop analysis can widen correctly.
                let mut base = aa.array;
                loop {
                    match &base.kind {
                        ExprKind::Variable(name) => {
                            let name_str = name.as_ref().trim_start_matches('$');
                            if !ctx.var_is_defined(name_str) {
                                ctx.vars.insert(
                                    name_str.to_string(),
                                    Union::single(Atomic::TArray {
                                        key: Box::new(Union::mixed()),
                                        value: Box::new(ty.clone()),
                                    }),
                                );
                                ctx.assigned_vars.insert(name_str.to_string());
                            } else {
                                // Widen the existing array type to include the new value type.
                                // This ensures loop analysis can see the type change and widen properly.
                                let current = ctx.get_var(name_str);
                                let updated = widen_array_with_value(&current, &ty);
                                ctx.set_var(name_str, updated);
                            }
                            break;
                        }
                        ExprKind::ArrayAccess(inner) => {
                            if let Some(idx) = &inner.index {
                                self.analyze(idx, ctx);
                            }
                            base = inner.array;
                        }
                        _ => break,
                    }
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Issue emission
    // -----------------------------------------------------------------------

    pub fn emit(&mut self, kind: IssueKind, severity: Severity, span: php_ast::Span) {
        let lc = self.source_map.offset_to_line_col(span.start);
        let (line, col) = (lc.line + 1, lc.col as u16);
        let mut issue = Issue::new(
            kind,
            Location {
                file: self.file.clone(),
                line,
                col_start: col,
                col_end: col,
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

    // Helper to call a closure with a mutable context reference while holding &mut self.
    fn with_ctx<F, R>(&mut self, ctx: &mut Context, f: F) -> R
    where
        F: FnOnce(&mut ExpressionAnalyzer<'a>, &mut Context) -> R,
    {
        f(self, ctx)
    }
}

// ---------------------------------------------------------------------------
// Free functions
// ---------------------------------------------------------------------------

/// Widen an array type to include a new element value type.
/// Used when `$arr[$k] = $val` is analyzed — updates the array's value type
/// so loop analysis can detect the change and widen properly.
fn widen_array_with_value(current: &Union, new_value: &Union) -> Union {
    let mut result = Union::empty();
    result.possibly_undefined = current.possibly_undefined;
    result.from_docblock = current.from_docblock;
    let mut found_array = false;
    for atomic in &current.types {
        match atomic {
            Atomic::TKeyedArray { properties, .. } => {
                // Merge all existing keyed values with the new value type, converting to TArray
                let mut all_values = new_value.clone();
                for prop in properties.values() {
                    all_values = Union::merge(&all_values, &prop.ty);
                }
                result.add_type(Atomic::TArray {
                    key: Box::new(Union::mixed()),
                    value: Box::new(all_values),
                });
                found_array = true;
            }
            Atomic::TArray { key, value } => {
                let merged = Union::merge(value, new_value);
                result.add_type(Atomic::TArray {
                    key: key.clone(),
                    value: Box::new(merged),
                });
                found_array = true;
            }
            Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
                let merged = Union::merge(value, new_value);
                result.add_type(Atomic::TList {
                    value: Box::new(merged),
                });
                found_array = true;
            }
            Atomic::TMixed => {
                return Union::mixed();
            }
            other => {
                result.add_type(other.clone());
            }
        }
    }
    if !found_array {
        // Current type has no array component — don't introduce one.
        // (e.g. typed object; return the original type unchanged.)
        return current.clone();
    }
    result
}

pub fn infer_arithmetic(left: &Union, right: &Union) -> Union {
    // If either operand is mixed, result is mixed (could be numeric or array addition)
    if left.is_mixed() || right.is_mixed() {
        return Union::mixed();
    }

    // PHP array union: array + array → array (union of keys)
    let left_is_array = left.contains(|t| {
        matches!(
            t,
            Atomic::TArray { .. }
                | Atomic::TNonEmptyArray { .. }
                | Atomic::TList { .. }
                | Atomic::TNonEmptyList { .. }
                | Atomic::TKeyedArray { .. }
        )
    });
    let right_is_array = right.contains(|t| {
        matches!(
            t,
            Atomic::TArray { .. }
                | Atomic::TNonEmptyArray { .. }
                | Atomic::TList { .. }
                | Atomic::TNonEmptyList { .. }
                | Atomic::TKeyedArray { .. }
        )
    });
    if left_is_array || right_is_array {
        // Merge the two array types (simplified: return mixed array)
        let merged_left = if left_is_array {
            left.clone()
        } else {
            Union::single(Atomic::TArray {
                key: Box::new(Union::single(Atomic::TMixed)),
                value: Box::new(Union::mixed()),
            })
        };
        return merged_left;
    }

    let left_is_float = left.contains(|t| matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..)));
    let right_is_float =
        right.contains(|t| matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..)));
    if left_is_float || right_is_float {
        Union::single(Atomic::TFloat)
    } else if left.contains(|t| t.is_int()) && right.contains(|t| t.is_int()) {
        Union::single(Atomic::TInt)
    } else {
        // Could be int or float (e.g. mixed + int)
        let mut u = Union::empty();
        u.add_type(Atomic::TInt);
        u.add_type(Atomic::TFloat);
        u
    }
}

pub fn extract_simple_var<'arena, 'src>(expr: &php_ast::ast::Expr<'arena, 'src>) -> Option<String> {
    match &expr.kind {
        ExprKind::Variable(name) => Some(name.as_ref().trim_start_matches('$').to_string()),
        ExprKind::Parenthesized(inner) => extract_simple_var(inner),
        _ => None,
    }
}

/// Extract all variable names from a list/array destructure pattern.
/// e.g. `[$a, $b]` or `list($a, $b)` → `["a", "b"]`
/// Returns an empty vec if the expression is not a destructure.
pub fn extract_destructure_vars<'arena, 'src>(
    expr: &php_ast::ast::Expr<'arena, 'src>,
) -> Vec<String> {
    match &expr.kind {
        ExprKind::Array(elements) => {
            let mut vars = vec![];
            for elem in elements.iter() {
                // Nested destructure or simple variable
                let sub = extract_destructure_vars(&elem.value);
                if sub.is_empty() {
                    if let Some(v) = extract_simple_var(&elem.value) {
                        vars.push(v);
                    }
                } else {
                    vars.extend(sub);
                }
            }
            vars
        }
        _ => vec![],
    }
}

/// Like `ast_params_to_fn_params` but resolves type names through the file's import table.
fn ast_params_to_fn_params_resolved<'arena, 'src>(
    params: &php_ast::ast::ArenaVec<'arena, php_ast::ast::Param<'arena, 'src>>,
    self_fqcn: Option<&str>,
    codebase: &mir_codebase::Codebase,
    file: &str,
) -> Vec<mir_codebase::FnParam> {
    params
        .iter()
        .map(|p| {
            let ty = p
                .type_hint
                .as_ref()
                .map(|h| crate::parser::type_from_hint(h, self_fqcn))
                .map(|u| resolve_named_objects_in_union(u, codebase, file));
            mir_codebase::FnParam {
                name: p.name.trim_start_matches('$').into(),
                ty,
                default: p.default.as_ref().map(|_| Union::mixed()),
                is_variadic: p.variadic,
                is_byref: p.by_ref,
                is_optional: p.default.is_some() || p.variadic,
            }
        })
        .collect()
}

/// Resolve TNamedObject fqcns in a union through the file's import table.
fn resolve_named_objects_in_union(
    union: Union,
    codebase: &mir_codebase::Codebase,
    file: &str,
) -> Union {
    use mir_types::Atomic;
    let from_docblock = union.from_docblock;
    let possibly_undefined = union.possibly_undefined;
    let types: Vec<Atomic> = union
        .types
        .into_iter()
        .map(|a| match a {
            Atomic::TNamedObject { fqcn, type_params } => {
                let resolved = codebase.resolve_class_name(file, fqcn.as_ref());
                Atomic::TNamedObject {
                    fqcn: resolved.into(),
                    type_params,
                }
            }
            other => other,
        })
        .collect();
    let mut result = Union::from_vec(types);
    result.from_docblock = from_docblock;
    result.possibly_undefined = possibly_undefined;
    result
}

fn extract_string_from_expr<'arena, 'src>(
    expr: &php_ast::ast::Expr<'arena, 'src>,
) -> Option<String> {
    match &expr.kind {
        ExprKind::Identifier(s) => Some(s.trim_start_matches('$').to_string()),
        // Variable in property position means dynamic access ($obj->$prop) — not a literal name.
        ExprKind::Variable(_) => None,
        ExprKind::String(s) => Some(s.to_string()),
        _ => None,
    }
}
