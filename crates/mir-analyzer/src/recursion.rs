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

/// Runs `f` with at least [`PHASE_RED_ZONE`] of stack available.
#[inline]
pub fn ensure_phase_stack<R>(f: impl FnOnce() -> R) -> R {
    grow(PHASE_RED_ZONE, PHASE_SEGMENT_SIZE, f)
}

/// A parse result whose deeply nested AST is dropped on a stack large enough for the recursive `Drop`.
pub struct GuardedParse(Option<php_rs_parser::ParseResult>);

impl std::ops::Deref for GuardedParse {
    type Target = php_rs_parser::ParseResult;

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
pub fn parse_versioned(source: &str, version: php_rs_parser::PhpVersion) -> GuardedParse {
    GuardedParse(Some(ensure_phase_stack(|| {
        php_rs_parser::parse_versioned(source, version)
    })))
}

/// Parses `source` with the default PHP version; see [`parse_versioned`].
pub fn parse(source: &str) -> GuardedParse {
    GuardedParse(Some(ensure_phase_stack(|| php_rs_parser::parse(source))))
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
pub fn walk_expr<V: OwnedVisitor + ?Sized>(visitor: &mut V, expr: &Expr) -> ControlFlow<()> {
    ensure_stack(|| walk_owned_expr(visitor, expr))
}

/// Guarded [`php_ast::owned::visitor::walk_owned_stmt`].
pub fn walk_stmt<V: OwnedVisitor + ?Sized>(visitor: &mut V, stmt: &Stmt) -> ControlFlow<()> {
    ensure_stack(|| walk_owned_stmt(visitor, stmt))
}

/// Guarded [`php_ast::owned::visitor::walk_owned_block`].
pub fn walk_block<V: OwnedVisitor + ?Sized>(visitor: &mut V, block: &Block) -> ControlFlow<()> {
    ensure_stack(|| walk_owned_block(visitor, block))
}
