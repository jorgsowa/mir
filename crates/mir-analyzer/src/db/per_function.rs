//! Per-function inference tracked query (free functions only — prototype).
//!
//! [`infer_function`] is the rust-analyzer-style primitive: salsa memoizes
//! one function's diagnostics + inferred return type at function granularity.
//! Editing file A's `bar()` does not invalidate cached results for `foo()`.
//!
//! Today the query is keyed by `(SourceFile, fn_fqn)`; the PHP version is
//! read from the [`AnalyzeFileInput`] singleton (a tracked field read). Edits
//! to the file's source text invalidate the entire file's set of function
//! caches via salsa's dependency on `parse_file(db, file)`; finer per-function
//! invalidation would require giving function bodies their own salsa input
//! granularity, which is out of scope for this prototype.
//!
//! Methods and closures are deferred. The driver method
//! [`crate::body_analysis::BodyAnalyzer::analyze_fn_decl_pure`] is the pure entry point
//! that produces the result without mutating caller-owned buffers.

use std::sync::Arc;

use mir_issues::Issue;
use mir_types::Type;

use super::*;

/// Output of [`infer_function`]: everything body-analysis produces for one free function
/// that we want salsa to memoize.
///
/// Notably excludes [`crate::symbol::ResolvedSymbol`]s — those are intentionally
/// re-walked on demand to keep the cache small (see the comment on
/// `RefLocAccumulator` in `db/reference_locations.rs`).
#[derive(Clone, Debug)]
pub struct FunctionInferenceResult {
    pub issues: Vec<Issue>,
    pub ref_locs: Vec<RefLoc>,
    pub return_type: Option<Type>,
}

impl PartialEq for FunctionInferenceResult {
    fn eq(&self, other: &Self) -> bool {
        self.issues == other.issues
            && self.ref_locs == other.ref_locs
            && self.return_type == other.return_type
    }
}

impl Eq for FunctionInferenceResult {}

unsafe impl salsa::Update for FunctionInferenceResult {
    unsafe fn maybe_update(old_ptr: *mut Self, new_val: Self) -> bool {
        let old = unsafe { &mut *old_ptr };
        if *old == new_val {
            return false;
        }
        *old = new_val;
        true
    }
}

/// Find the FunctionDecl in `program` whose resolved FQN equals `target_fqn`.
///
/// Recurses through control-flow wrappers and braced namespaces via
/// [`crate::body_analysis::for_each_file_scope_decl`], so a function declared
/// inside an `if (! function_exists('foo')) { … }` guard (the Laravel helper
/// pattern) is located just like a top-level one. Name resolution goes through
/// `resolve_name`, which consults `db.file_namespace`, so braced and unbraced
/// namespaces resolve identically. Returns the first match (PHP doesn't allow
/// duplicate function definitions).
fn find_function_decl<'a>(
    program: &'a php_ast::owned::Program,
    db: &dyn MirDatabase,
    file: &str,
    target_fqn: &str,
) -> Option<&'a php_ast::owned::FunctionDecl> {
    use php_ast::owned::StmtKind;
    let mut found: Option<&'a php_ast::owned::FunctionDecl> = None;
    crate::body_analysis::for_each_file_scope_decl(&program.stmts, &mut |stmt| {
        if found.is_some() {
            return;
        }
        if let StmtKind::Function(decl) = &stmt.kind {
            let name = decl.name.as_deref().unwrap_or("");
            if !name.is_empty() && crate::db::resolve_name(db, file, name) == target_fqn {
                found = Some(decl);
            }
        }
    });
    found
}

/// Per-function inference tracked query.
///
/// Runs body-analysis analysis on the single function `fn_fqn` declared in `file`.
/// Returns memoized issues + reference-locations + inferred return type.
/// Returns `None` only when the function declaration can't be located in the
/// file's AST (e.g. fn_fqn does not refer to a function declared in this file).
#[salsa::tracked]
pub fn infer_function(
    db: &dyn MirDatabase,
    file: SourceFile,
    fn_fqn: Arc<str>,
) -> Option<Arc<FunctionInferenceResult>> {
    use std::str::FromStr as _;

    let path = file.path(db);
    let text = file.text(db);
    let php_version_str = db.analyze_config().php_version(db);
    let php_version = crate::php_version::PhpVersion::from_str(php_version_str.as_ref())
        .unwrap_or(crate::php_version::PhpVersion::LATEST);

    let parsed_file = crate::db::parse_file(db, file);
    let parsed = &*parsed_file.0;

    if parsed.errors.iter().any(crate::parser::is_hard_parse_error) {
        return None;
    }

    let decl = find_function_decl(&parsed.program, db, path.as_ref(), fn_fqn.as_ref())?;

    let driver = crate::body_analysis::BodyAnalyzer::new(db, php_version);
    let result = driver.analyze_fn_decl_pure(decl, &path, text.as_ref(), &parsed.source_map);
    Some(Arc::new(result))
}
