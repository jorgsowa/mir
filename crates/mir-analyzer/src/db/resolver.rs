//! Salsa-tracked FQCN resolution.
//!
//! Bridges the `Arc<dyn ClassResolver>` side-channel (which lives outside
//! salsa because trait objects don't compose with `salsa::Update`) into the
//! salsa invalidation graph via a one-field input ([`ResolverConfig`])
//! whose `revision` is bumped on every resolver change.
//!
//! Phase-3 callers (`find_class` / `find_function` / `find_method` tracked
//! queries) consume [`resolve_fqcn_to_path`] to map FQCN → file path, then
//! demand `collect_file_definitions` on the resulting `SourceFile`. When
//! the resolver changes, salsa correctly invalidates everything downstream.
//!
//! Foundation laid by Phase 2 of
//! `~/.claude/plans/sequential-popping-parasol.md`.

use std::sync::Arc;

use crate::db::MirDatabase;
use mir_types::Symbol;

/// Singleton salsa input that anchors resolver-derived tracked queries on
/// a revision counter. The actual `Arc<dyn ClassResolver>` lives off-db
/// (see `MirDatabase::current_resolver`); `revision` is bumped each time
/// the resolver is replaced so salsa invalidates dependents.
#[salsa::input]
pub struct ResolverConfig {
    pub revision: u64,
}

/// Salsa-interned FQCN used as the key for [`resolve_fqcn_to_path`].
///
/// Salsa requires tracked-function arguments to be salsa structs; this
/// gives the FQCN a stable interned identity so the resolution result can
/// be memoized per name. Cheap to construct (`Fqcn::new(db, symbol)`);
/// equality is by ustr pointer (O(1)).
#[salsa::interned]
pub struct Fqcn<'db> {
    pub name: Symbol,
}

/// Resolve an FQCN to its defining file path via the configured resolver.
///
/// Tracked: depends on [`ResolverConfig::revision`], so callers reading
/// this from a tracked context are invalidated when the resolver changes.
/// Reads the resolver side-channel via [`MirDatabase::current_resolver`]
/// — that read is *not* salsa-tracked, but the revision anchor makes it
/// correct as long as every resolver swap bumps the revision (enforced by
/// `MirDb::set_resolver`).
///
/// Returns `None` when no resolver is configured or the resolver couldn't
/// map `fqcn`.
#[salsa::tracked]
pub fn resolve_fqcn_to_path<'db>(db: &'db dyn MirDatabase, fqcn: Fqcn<'db>) -> Option<Arc<str>> {
    let cfg = db.resolver_config()?;
    // Anchor on the revision so this query is part of salsa's graph.
    let _rev = cfg.revision(db);
    let resolver = db.current_resolver()?;
    let name = fqcn.name(db);
    let path = resolver.resolve(name.as_str())?;
    Some(Arc::from(path.to_string_lossy().as_ref()))
}

/// Composite: resolve an FQCN to a registered [`crate::db::SourceFile`] if
/// the workspace has the defining file's text loaded.
///
/// Not currently tracked: it composes [`resolve_fqcn_to_path`] (tracked)
/// with `MirDatabase::lookup_source_file` (untracked map read). Phase 3
/// will likely promote the path-keyed lookup to a tracked query to fully
/// participate in salsa's invalidation graph.
pub fn source_file_for_fqcn<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
) -> Option<crate::db::SourceFile> {
    if let Some(path) = resolve_fqcn_to_path(db, fqcn) {
        if let Some(sf) = db.lookup_source_file(&path) {
            return Some(sf);
        }
    }
    // Resolver miss / no resolver: consult the workspace symbol index built
    // across all registered SourceFiles.
    let name = fqcn.name(db);
    let lower = name.ascii_lowercase();
    let index = crate::db::workspace_index(db);
    if let Some(loc) = index.class_like.get(&lower) {
        return Some(match loc {
            crate::db::SymbolLoc::Class { file, .. }
            | crate::db::SymbolLoc::Interface { file, .. }
            | crate::db::SymbolLoc::Trait { file, .. }
            | crate::db::SymbolLoc::Enum { file, .. } => *file,
            _ => return None,
        });
    }
    if let Some(crate::db::SymbolLoc::Function { file, .. }) = index.functions.get(&lower) {
        return Some(*file);
    }
    if let Some(crate::db::SymbolLoc::Constant { file, .. }) = index.constants.get(&name) {
        return Some(*file);
    }
    None
}
