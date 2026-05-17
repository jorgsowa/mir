/// Expression analyzer — infers the `Union` type of any PHP expression.
use std::sync::Arc;

use php_ast::ast::ExprKind;

use mir_issues::{Issue, IssueBuffer, IssueKind, Location, Severity};
use mir_types::{Atomic, Union};

use crate::call::CallAnalyzer;
use crate::context::Context;
use crate::db::MirDatabase;
use crate::php_version::PhpVersion;
use crate::symbol::{ResolvedSymbol, SymbolKind};

mod arrays;
mod assignment;
mod binary;
mod casts;
mod closures;
mod conditional;
mod helpers;
mod intrinsics;
mod literals;
mod objects;
mod unary;
mod variables;

#[allow(unused_imports)]
pub use helpers::{extract_destructure_vars, extract_simple_var, infer_arithmetic};

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
    /// When true, skip all reference-tracking side-effects (used by the
    /// inference priming pass so reference locations aren't double-counted).
    pub inference_only: bool,
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
        inference_only: bool,
    ) -> Self {
        Self {
            db,
            file,
            source,
            source_map,
            issues,
            symbols,
            php_version,
            inference_only,
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
            ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::String(_)
            | ExprKind::Bool(_)
            | ExprKind::Null => literals::analyze(&expr.kind),

            ExprKind::InterpolatedString(parts) | ExprKind::Heredoc { parts, .. } => {
                for part in parts.iter() {
                    if let php_ast::StringPart::Expr(e) = part {
                        let expr_ty = self.analyze(e, ctx);
                        self.check_interpolation_implicit_to_string_cast(&expr_ty, e.span);
                    }
                }
                Union::single(Atomic::TString)
            }
            ExprKind::Nowdoc { .. } => Union::single(Atomic::TString),
            ExprKind::ShellExec(_) => Union::single(Atomic::TString),

            // --- Variables --------------------------------------------------
            ExprKind::Variable(name) => self.analyze_variable(name, expr, ctx),
            ExprKind::VariableVariable(inner) => self.analyze_variable_variable(inner, ctx),
            ExprKind::Identifier(name) => self.analyze_identifier(name, expr, ctx),

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
                let expr_ty = self.analyze(inner, ctx);
                self.check_interpolation_implicit_to_string_cast(&expr_ty, inner.span);
                Union::single(Atomic::TLiteralInt(1))
            }

            // --- clone ------------------------------------------------------
            ExprKind::Clone(inner) => {
                let ty = self.analyze(inner, ctx);
                if ty.is_mixed() {
                    self.emit(IssueKind::MixedClone, Severity::Info, expr.span);
                }
                ty
            }
            ExprKind::CloneWith(inner, _props) => {
                let ty = self.analyze(inner, ctx);
                if ty.is_mixed() {
                    self.emit(IssueKind::MixedClone, Severity::Info, expr.span);
                }
                ty
            }

            // --- new ClassName(...) ----------------------------------------
            ExprKind::New(n) => self.analyze_new(n, expr.span, ctx),

            ExprKind::AnonymousClass(anon) => {
                for member in anon.members.iter() {
                    if let php_ast::ast::ClassMemberKind::Method(method) = &member.kind {
                        let Some(body) = &method.body else { continue };
                        let mut sa = crate::stmt::StatementsAnalyzer::new(
                            self.db,
                            self.file.clone(),
                            self.source,
                            self.source_map,
                            self.issues,
                            self.symbols,
                            self.php_version,
                            self.inference_only,
                        );
                        let mut method_ctx = crate::context::Context::for_function(
                            &[],
                            None,
                            std::sync::Arc::from([]),
                            None,
                            None,
                            None,
                            ctx.strict_types,
                            false,
                        );
                        sa.analyze_stmts(body, &mut method_ctx);
                    }
                }
                Union::single(Atomic::TObject)
            }

            // --- Property access -------------------------------------------
            ExprKind::PropertyAccess(pa) => self.analyze_property_access(pa, expr.span, ctx),

            ExprKind::NullsafePropertyAccess(pa) => self.analyze_nullsafe_property_access(pa, ctx),

            ExprKind::StaticPropertyAccess(spa) => self.analyze_static_property_access(spa),

            ExprKind::ClassConstAccess(cca) => self.analyze_class_const_access(cca, expr.span, ctx),

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

            ExprKind::StaticDynMethodCall(smc) => {
                CallAnalyzer::analyze_static_dyn_method_call(self, smc, ctx)
            }

            // --- Function calls --------------------------------------------
            ExprKind::FunctionCall(fc) => {
                CallAnalyzer::analyze_function_call(self, fc, ctx, expr.span)
            }

            // --- Closures / arrow functions --------------------------------
            ExprKind::Closure(c) => self.analyze_closure(c, ctx),

            ExprKind::ArrowFunction(af) => self.analyze_arrow_function(af, ctx),

            ExprKind::CallableCreate(_) => Union::single(Atomic::TCallable {
                params: None,
                return_type: None,
            }),

            // --- Match expression ------------------------------------------
            ExprKind::Match(m) => self.analyze_match(m, ctx),

            // --- Throw as expression (PHP 8) --------------------------------
            ExprKind::ThrowExpr(e) => {
                self.analyze(e, ctx);
                Union::single(Atomic::TNever)
            }

            // --- Yield -----------------------------------------------------
            ExprKind::Yield(y) => self.analyze_yield(y, ctx),

            // --- Magic constants -------------------------------------------
            ExprKind::MagicConst(kind) => ExpressionAnalyzer::analyze_magic_const(kind),

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
                ctx.diverges = true;
                Union::single(Atomic::TNever)
            }

            // --- Error node (parse error placeholder) ----------------------
            ExprKind::Error => Union::mixed(),

            // --- Omitted array slot (e.g. [, $b] destructuring) ------------
            ExprKind::Omit => Union::single(Atomic::TNull),
        }
    }

    // -----------------------------------------------------------------------
    // Issue emission
    // -----------------------------------------------------------------------

    /// Convert a byte offset to a Unicode char-count column on a given line.
    /// Returns (line, col) where col is a 0-based Unicode code-point count.
    fn offset_to_line_col(&self, offset: u32) -> (u32, u16) {
        let lc = self.source_map.offset_to_line_col(offset);
        let line = lc.line + 1;

        let byte_offset = offset as usize;
        let line_start_byte = if byte_offset == 0 {
            0
        } else {
            self.source[..byte_offset]
                .rfind('\n')
                .map(|p| p + 1)
                .unwrap_or(0)
        };

        let col = self.source[line_start_byte..byte_offset].chars().count() as u16;

        (line, col)
    }

    /// Convert an AST span to `(line, col_start, col_end)` for reference recording.
    pub(crate) fn span_to_ref_loc(&self, span: php_ast::Span) -> (u32, u16, u16) {
        let (line, col_start) = self.offset_to_line_col(span.start);
        let end_off = (span.end as usize).min(self.source.len());
        let end_line_start = self.source[..end_off]
            .rfind('\n')
            .map(|p| p + 1)
            .unwrap_or(0);
        let col_end = self.source[end_line_start..end_off].chars().count() as u16;
        (line, col_start, col_end)
    }

    /// Walk a type hint and emit `UndefinedClass` for any named type not in the codebase.
    fn check_type_hint(&mut self, hint: &php_ast::ast::TypeHint<'_, '_>) {
        use php_ast::ast::TypeHintKind;
        match &hint.kind {
            TypeHintKind::Named(name) => {
                let name_str = crate::parser::name_to_string(name);
                if matches!(
                    name_str.to_lowercase().as_str(),
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
                let resolved = crate::db::resolve_name_via_db(self.db, &self.file, &name_str);
                if !crate::db::type_exists_via_db(self.db, &resolved) {
                    self.emit(
                        IssueKind::UndefinedClass { name: resolved },
                        Severity::Error,
                        hint.span,
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
                col_end: col_end.max(col_start + 1),
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

    fn check_interpolation_implicit_to_string_cast(&mut self, ty: &Union, span: php_ast::Span) {
        for atomic in &ty.types {
            if let Atomic::TNamedObject { fqcn, .. } = atomic {
                let fqcn_str = fqcn.as_ref();
                if crate::db::lookup_method_in_chain(self.db, fqcn_str, "__toString").is_none()
                    && !crate::db::extends_or_implements_via_db(self.db, fqcn_str, "Stringable")
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
        let bump = crate::arena::create_parse_arena(source.len());
        let result = php_rs_parser::parse(&bump, source);
        result.source_map
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
        // Ensure col_end is at least col_start + 1 (1 character minimum)
        let col_start = 0u16;
        let col_end = 0u16; // Would happen if span.start == span.end
        let effective_col_end = col_end.max(col_start + 1);

        assert_eq!(
            effective_col_end, 1,
            "col_end should be at least col_start + 1"
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
