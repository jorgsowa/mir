//! Stack-overflow protection for the recursive analysis passes.
//!
//! Expression/statement analysis recurses along the AST (e.g. a fluent
//! `->a()->b()->…` chain), and on-demand return-type inference nests whole
//! body analyses of other files on the same stack. Worker threads usually have
//! only 2 MiB, so deep input must grow the stack on the heap instead of aborting.

use std::ops::ControlFlow;

use php_ast::owned::visitor::{walk_owned_block, walk_owned_expr, walk_owned_stmt, OwnedVisitor};
use php_ast::owned::{Block, Expr, Stmt};

/// Stack size for analysis worker threads; matches rust-analyzer and ty.
pub const STACK_SIZE: usize = 16 * 1024 * 1024;

/// Remaining stack below which a guarded call continues on a fresh segment.
/// Must exceed the stack used between two guards (frames are large in debug builds).
const RED_ZONE: usize = 512 * 1024;
const SEGMENT_SIZE: usize = 8 * 1024 * 1024;

/// Parsing and the hand-written AST walkers are not guarded per node, so each
/// analysis phase is guaranteed this much stack up front.
const PHASE_RED_ZONE: usize = 4 * 1024 * 1024;
const PHASE_SEGMENT_SIZE: usize = 32 * 1024 * 1024;

/// Runs `f`, continuing on a heap-allocated stack segment when the current one is nearly exhausted.
#[inline]
pub fn ensure_stack<R>(f: impl FnOnce() -> R) -> R {
    grow(RED_ZONE, SEGMENT_SIZE, f)
}

/// Runs `f` with enough stack for an unguarded analysis phase.
#[inline]
pub fn ensure_phase_stack<R>(f: impl FnOnce() -> R) -> R {
    grow(PHASE_RED_ZONE, PHASE_SEGMENT_SIZE, f)
}

/// The owned AST, errors and line index of a parse, without the copy of the source text and the comments `php_rs_parser::ParseResult` carries.
pub struct ParsedSource {
    pub program: php_ast::owned::Program,
    pub errors: Vec<php_rs_parser::diagnostics::ParseError>,
    pub source_map: php_rs_parser::source_map::SourceMap,
}

/// A parse result whose deeply nested AST is dropped on a stack large enough for the recursive `Drop`.
pub struct GuardedParse(Option<ParsedSource>);

impl std::ops::Deref for GuardedParse {
    type Target = ParsedSource;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("present until dropped")
    }
}

impl Drop for GuardedParse {
    fn drop(&mut self) {
        let parsed = self.0.take();
        ensure_phase_stack(|| drop(parsed));
    }
}

/// Parses `source`, guaranteeing enough stack for deeply nested input.
pub fn parse_versioned(source: &str, version: php_ast::PhpVersion) -> GuardedParse {
    parse_with(
        source,
        &php_rs_parser::ParseOptions::default().version(version),
    )
}

fn parse_with(source: &str, options: &php_rs_parser::ParseOptions) -> GuardedParse {
    GuardedParse(Some(ensure_phase_stack(|| {
        let arena = php_rs_parser::bumpalo::Bump::new();
        let parsed = php_rs_parser::parse_arena_with(&arena, source, options);
        ParsedSource {
            program: php_ast::owned::to_owned_program(&parsed.program),
            errors: parsed.errors,
            source_map: parsed.source_map,
        }
    })))
}

/// Parses `path` at `php_version`; embedded stubs gate symbols by `#[...]` attributes, so they parse at the latest version.
pub fn parse_for_file(path: &str, source: &str, php_version: crate::PhpVersion) -> GuardedParse {
    let version = if crate::stubs::is_stub_path(path) {
        php_version.max(crate::PhpVersion::LATEST)
    } else {
        php_version
    };
    parse_versioned(source, version.parser_version())
}

/// Parses `source` with the default PHP version; see [`parse_versioned`].
pub fn parse(source: &str) -> GuardedParse {
    parse_with(source, &php_rs_parser::ParseOptions::default())
}

#[cfg(not(target_arch = "wasm32"))]
fn grow<R>(red_zone: usize, segment: usize, f: impl FnOnce() -> R) -> R {
    stacker::maybe_grow(red_zone, segment, f)
}

#[cfg(target_arch = "wasm32")]
fn grow<R>(_red_zone: usize, _segment: usize, f: impl FnOnce() -> R) -> R {
    f()
}

/// Guarded [`php_ast::owned::visitor::walk_owned_expr`]; visitors recurse through it.
pub fn walk_expr<V: OwnedVisitor + ?Sized>(visitor: &mut V, expr: &Expr) -> ControlFlow<V::Break> {
    ensure_stack(|| walk_owned_expr(visitor, expr))
}

/// Guarded [`php_ast::owned::visitor::walk_owned_stmt`].
pub fn walk_stmt<V: OwnedVisitor + ?Sized>(visitor: &mut V, stmt: &Stmt) -> ControlFlow<V::Break> {
    ensure_stack(|| walk_owned_stmt(visitor, stmt))
}

/// Guarded [`php_ast::owned::visitor::walk_owned_block`].
pub fn walk_block<V: OwnedVisitor + ?Sized>(
    visitor: &mut V,
    block: &Block,
) -> ControlFlow<V::Break> {
    ensure_stack(|| walk_owned_block(visitor, block))
}
