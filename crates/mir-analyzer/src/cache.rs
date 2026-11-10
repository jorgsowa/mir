/// Per-file analysis result cache backed by a binary file on disk.
///
/// Cache key: file path.  Cache validity: BLAKE3 hash of file content.
/// If the content hash matches what was stored, the cached issues are returned
/// and body analysis analysis is skipped for that file.
///
/// Internally, path strings are mapped to compact [`FileId`] integers so the
/// hot-path lookups (`get` / `put`) hash a `u32` instead of a full path string.
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use mir_codebase::{FileId, FileIdMap};
use parking_lot::Mutex;

use serde::{Deserialize, Serialize};

use mir_issues::Issue;

/// Reference location tuple recorded during body analysis:
/// `(symbol_key, line, col_start, col_end)`.
pub type CachedRefLoc = (std::sync::Arc<str>, u32, u16, u16);

/// Cached analysis result returned on a cache hit. Arc-shared with the stored
/// entry so a hit is two refcount bumps, not a deep clone under the cache lock.
pub type CacheHit = (std::sync::Arc<[Issue]>, std::sync::Arc<[CachedRefLoc]>);

// ---------------------------------------------------------------------------
// Hash helper
// ---------------------------------------------------------------------------

/// Compute the BLAKE3 hex digest of `content`.
pub fn hash_content(content: &str) -> String {
    digest_hex(blake3::hash(content.as_bytes()).as_bytes())
}

/// Hex form of a BLAKE3 digest, e.g. one from [`crate::stub_cache::hash_source`].
pub fn digest_hex(digest: &[u8; 32]) -> String {
    blake3::Hash::from_bytes(*digest).to_hex().to_string()
}

/// BLAKE3 hex digest of `source` with the body of every **declared-return**
/// function and method removed.
///
/// This is the cache's *cross-file surface* key: a dependent file's diagnostics
/// are a function of what it can observe of this file, and a declared-return
/// callable exposes only its declared return type (the analyzer prefers the
/// declared type over the body-inferred one). So edits confined to such a body
/// leave this digest — and every dependent's result — unchanged, and don't need
/// to cascade re-analysis.
///
/// Bodies of callables *without* a declared return type are deliberately kept:
/// their inferred return type is body-derived and observable across files
/// (`infer_file_return_types`), so a change there must still cascade. Keeping a
/// body that could have been stripped only ever over-cascades (re-analyzes a
/// dependent needlessly) — never serves a stale result — so this is sound by
/// construction even if a future callable form is missed here.
///
/// Whitespace and comments are ignored too, except docblocks and any comment
/// mentioning `@`, which can carry annotations.
pub fn surface_fingerprint(source: &str, program: &php_ast::owned::Program) -> String {
    use php_ast::owned::visitor::{walk_owned_class_member, walk_owned_program, OwnedVisitor};
    use php_ast::owned::{ClassMember, ClassMemberKind, Stmt, StmtKind};
    use std::ops::ControlFlow;

    struct BodySpans {
        spans: Vec<(u32, u32)>,
    }
    impl OwnedVisitor for BodySpans {
        type Break = ();

        fn visit_stmt(&mut self, stmt: &Stmt) -> ControlFlow<()> {
            if let StmtKind::Function(f) = &stmt.kind {
                if f.return_type.is_some() {
                    self.spans.push((f.body.span.start, f.body.span.end));
                }
            }
            crate::recursion::walk_stmt(self, stmt)
        }
        fn visit_expr(&mut self, expr: &php_ast::owned::Expr) -> ControlFlow<()> {
            crate::recursion::walk_expr(self, expr)
        }
        fn visit_class_member(&mut self, member: &ClassMember) -> ControlFlow<()> {
            if let ClassMemberKind::Method(m) = &member.kind {
                if m.return_type.is_some() {
                    if let Some(body) = &m.body {
                        self.spans.push((body.span.start, body.span.end));
                    }
                }
            }
            walk_owned_class_member(self, member)
        }
    }

    let mut collector = BodySpans { spans: Vec::new() };
    let _ = walk_owned_program(&mut collector, program);
    collector.spans.sort_unstable();

    let mut bodies = collector.spans.into_iter().peekable();
    let mut lexer = php_lexer::Lexer::new(source);
    // Hashed in one call: BLAKE3's per-update overhead dominates for single tokens,
    // and the digest of the concatenation is the same.
    let mut surface: Vec<u8> = Vec::with_capacity(source.len());
    let mut in_body = false;
    loop {
        let token = lexer.next_token();
        if token.kind == php_lexer::TokenKind::Eof {
            break;
        }
        let start = token.span.start;
        while bodies.next_if(|&(_, end)| end <= start).is_some() {}
        if bodies.peek().is_some_and(|&(body, _)| body <= start) {
            if !in_body {
                surface.extend_from_slice(b"\x00body\x00");
                in_body = true;
            }
            continue;
        }
        in_body = false;
        let text = lexer.token_text(&token);
        let plain_comment = matches!(
            token.kind,
            php_lexer::TokenKind::LineComment
                | php_lexer::TokenKind::HashComment
                | php_lexer::TokenKind::BlockComment
        ) && !text.contains('@');
        if !plain_comment {
            surface.extend_from_slice(text.as_bytes());
            surface.push(0);
        }
    }
    if !lexer.errors().is_empty() {
        return hash_content(source);
    }
    blake3::hash(&surface).to_hex().to_string()
}

/// Memoized identity of the running mir build. Computed once per process.
///
/// This is the hash of the running executable's own bytes. Any new mir build —
/// a released version bump, a different commit, or a local rebuild that changed
/// analysis logic — produces different bytes and therefore a different
/// fingerprint, so an old build's cache is never reused (the embedded stub set
/// is part of the binary too, so this subsumes stub-set changes). A bare
/// `CARGO_PKG_VERSION` would only catch explicit version bumps, leaving every
/// same-version rebuild serving stale results.
///
/// Falls back to `CARGO_PKG_VERSION` + the bundled stub catalogue if the
/// executable can't be read (unusual sandboxes) — still correct for the inputs
/// that change most often, just blind to pure-logic changes within a version.
fn build_fingerprint() -> u64 {
    static FP: OnceLock<u64> = OnceLock::new();
    *FP.get_or_init(|| {
        let exe_bytes = std::env::current_exe().and_then(std::fs::read).ok();
        compute_build_fingerprint(exe_bytes.as_deref())
    })
}

fn compute_build_fingerprint(exe_bytes: Option<&[u8]>) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(env!("CARGO_PKG_VERSION").as_bytes());
    hasher.update(&[0]);
    match exe_bytes {
        // Primary: the build's own bytes uniquely identify it.
        Some(bytes) => {
            hasher.update(bytes);
        }
        // Fallback: the bundled stub set (paths + contents).
        None => {
            for (path, content) in crate::stubs::stub_files() {
                hasher.update(path.as_bytes());
                hasher.update(&[0]);
                hasher.update(content.as_bytes());
                hasher.update(&[0]);
            }
        }
    }
    let bytes = hasher.finalize();
    u64::from_le_bytes(bytes.as_bytes()[..8].try_into().unwrap())
}

/// Epoch the on-disk cache is validated against. Folds together every input
/// that affects a file's analysis result *besides the file's own content*:
/// the build fingerprint (the running mir binary's identity, which includes
/// the crate version + embedded stub set) and the target PHP version.
///
/// A per-file cache entry's validity is otherwise keyed only on the file's
/// content hash, which silently assumes those global inputs are unchanged
/// between runs. They aren't:
///
/// - Bundled stubs change (e.g. vendoring `redis`/`memcached`, or a
///   phpstorm-stubs bump) → files referencing a now-resolvable built-in keep
///   a stale `UndefinedClass`.
/// - The target PHP version changes (`--php-version`, or an edited
///   `composer.json` constraint) → version-gated symbols (`@since`/`@removed`,
///   LanguageLevelTypeAware params) resolve differently, but the file's bytes
///   are identical so the stale result is served.
///
/// - User-configured stubs change (`user_stub_fp`, computed by
///   [`crate::stubs::user_stub_fingerprint`]) → same as bundled stubs, but for
///   custom stub files/dirs the project supplies.
///
/// Mixing them all into a cache-wide epoch invalidates the whole on-disk cache
/// on the next run when any of them change, instead of serving stale diagnostics.
fn cache_epoch(php_version: u8, user_stub_fp: u64) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&build_fingerprint().to_le_bytes());
    hasher.update(&[php_version]);
    hasher.update(&user_stub_fp.to_le_bytes());
    let bytes = hasher.finalize();
    u64::from_le_bytes(bytes.as_bytes()[..8].try_into().unwrap())
}

// ---------------------------------------------------------------------------
// CacheEntry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct CacheEntry {
    content_hash: String,
    /// Arc-shared so `get` hands out the stored slice without deep-cloning
    /// every Issue while holding the cache lock.
    issues: std::sync::Arc<[Issue]>,
    /// Reference locations recorded during body analysis. Stored so that cache
    /// hits can replay symbol_reference_locations without re-running
    /// analyze_bodies. Arc-shared for the same reason as `issues`.
    reference_locations: std::sync::Arc<[CachedRefLoc]>,
    /// Digest of this file's cross-file surface (see [`surface_fingerprint`]).
    /// Dependents are evicted only when this changes between runs — a body-only
    /// edit to a declared-return callable leaves it untouched. An empty value
    /// is treated as "unknown" and conservatively cascades.
    surface_hash: String,
}

// ---------------------------------------------------------------------------
// AnalysisCache
// ---------------------------------------------------------------------------

/// On-disk form of `cache.bin`. Every path and reference symbol is stored
/// once in `strings` and referenced by index, since the same few thousand
/// symbols recur across every file's reference locations and every path
/// recurs across the reverse-dependency sets.
#[derive(Deserialize)]
struct CacheFile {
    /// Analyzer epoch the entries were written under (see [`cache_epoch`]).
    /// Kept first: `load` compares the leading 8 bytes before decoding the rest.
    _version: u64,
    strings: Vec<std::sync::Arc<str>>,
    entries: Vec<DiskEntry>,
    /// Defining file → files that depend on it.
    reverse_deps: Vec<(u32, Vec<u32>)>,
    /// Content digests of registered files without an entry of their own
    /// (vendor files that are indexed, never analyzed).
    dep_hashes: Vec<(u32, DepHash)>,
}

#[derive(Deserialize)]
struct DiskEntry {
    path: u32,
    content_hash: String,
    surface_hash: String,
    issues: Vec<Issue>,
    /// `(symbol, line, col_start, col_end)`, the symbol an index into `strings`.
    refs: Vec<(u32, u32, u16, u16)>,
}

/// Borrowing counterpart of [`CacheFile`] for `flush`; bincode encodes it
/// identically.
#[derive(Serialize)]
struct CacheFileView<'a> {
    version: u64,
    strings: Vec<&'a str>,
    entries: Vec<DiskEntryView<'a>>,
    reverse_deps: Vec<(u32, Vec<u32>)>,
    dep_hashes: Vec<(u32, &'a DepHash)>,
}

#[derive(Serialize)]
struct DiskEntryView<'a> {
    path: u32,
    content_hash: &'a str,
    surface_hash: &'a str,
    issues: &'a [Issue],
    refs: Vec<(u32, u32, u16, u16)>,
}

#[derive(Default)]
struct StringTable<'a> {
    ids: HashMap<&'a str, u32>,
    strings: Vec<&'a str>,
}

impl<'a> StringTable<'a> {
    fn id(&mut self, s: &'a str) -> u32 {
        *self.ids.entry(s).or_insert_with(|| {
            self.strings.push(s);
            (self.strings.len() - 1) as u32
        })
    }
}

/// Truncated BLAKE3 digest of a dependency-only file's content.
type DepHash = [u8; 16];

fn dep_hash(content_hex: &str) -> DepHash {
    let mut hash = DepHash::default();
    for (i, byte) in hash.iter_mut().enumerate() {
        *byte = content_hex
            .get(2 * i..2 * i + 2)
            .and_then(|pair| u8::from_str_radix(pair, 16).ok())
            .unwrap_or(0);
    }
    hash
}

/// Declarations newly registered or changed in a session, matched against
/// the missing names of cached results.
#[derive(Default)]
pub struct DeclaredNames {
    /// Lowercased last segments, so a match doesn't depend on how a missing
    /// name was qualified when reported.
    short_names: HashSet<String>,
    class_like: bool,
}

impl DeclaredNames {
    pub fn add_class_like(&mut self, fqcn: &str) {
        self.class_like = true;
        self.short_names.insert(name_key(fqcn));
    }

    pub fn add_function_or_constant(&mut self, name: &str) {
        self.short_names.insert(name_key(name));
    }

    pub(crate) fn add_declarations(&mut self, decls: &crate::db::FileDeclarations) {
        for decl in decls.class_like() {
            self.add_class_like(decl.name.as_str());
        }
        for decl in decls.functions().chain(decls.constants()) {
            self.add_function_or_constant(decl.name.as_str());
        }
    }

    pub fn add_slice(&mut self, slice: &mir_codebase::definitions::StubSlice) {
        for fqcn in slice
            .classes
            .iter()
            .map(|c| &c.fqcn)
            .chain(slice.interfaces.iter().map(|c| &c.fqcn))
            .chain(slice.traits.iter().map(|c| &c.fqcn))
            .chain(slice.enums.iter().map(|c| &c.fqcn))
        {
            self.add_class_like(fqcn);
        }
        for name in slice
            .functions
            .iter()
            .map(|f| &f.fqn)
            .chain(slice.constants.iter().map(|(name, _)| name))
        {
            self.add_function_or_constant(name);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.short_names.is_empty()
    }

    fn may_resolve(&self, issue: &Issue) -> bool {
        use mir_issues::IssueKind as K;
        match &issue.kind {
            K::UndefinedFunction { name }
            | K::UndefinedClass { name }
            | K::UndefinedTrait { name }
            | K::UndefinedDocblockClass { name }
            | K::UndefinedThrowsDocblock { name }
            | K::UndefinedAttributeClass { name } => self.short_names.contains(&name_key(name)),
            K::UndefinedConstant { name } if !name.contains("::") => {
                self.short_names.contains(&name_key(name))
            }
            // A member can come from any newly declared ancestor or trait.
            K::UndefinedConstant { .. }
            | K::UndefinedMethod { .. }
            | K::UndefinedProperty { .. }
            | K::UndefinedTraitAliasMethod { .. } => self.class_like,
            _ => false,
        }
    }
}

fn name_key(name: &str) -> String {
    name.rsplit('\\')
        .next()
        .unwrap_or(name)
        .to_ascii_lowercase()
}

#[derive(Default)]
struct DepGraph {
    /// file → files that depend on it.
    dependents: HashMap<FileId, HashSet<FileId>>,
    /// file → files it depends on (sorted): the inverse of `dependents`,
    /// built on the first per-file update so batch runs never pay for it.
    dependencies: Option<HashMap<FileId, Vec<FileId>>>,
}

fn invert(dependents: &HashMap<FileId, HashSet<FileId>>) -> HashMap<FileId, Vec<FileId>> {
    let mut dependencies: HashMap<FileId, Vec<FileId>> = HashMap::default();
    for (&target, sources) in dependents {
        for &source in sources {
            dependencies.entry(source).or_default().push(target);
        }
    }
    for targets in dependencies.values_mut() {
        targets.sort_unstable();
    }
    dependencies
}

/// Thread-safe, disk-backed cache for per-file analysis results.
pub struct AnalysisCache {
    cache_dir: PathBuf,
    /// Path ↔ FileId mapping; owns the canonical string storage so entry/dep
    /// maps can use 4-byte keys instead of heap-allocated path strings.
    file_id_map: Mutex<FileIdMap>,
    entries: Mutex<HashMap<FileId, CacheEntry>>,
    /// Entries whose diagnostics contain a workspace-resolvable missing name.
    /// Kept separately so workspace growth invalidates only this small set.
    unresolved_entries: Mutex<HashSet<FileId>>,
    /// Reverse dependency graph, loaded from disk and maintained per file.
    reverse_deps: Mutex<DepGraph>,
    dep_hashes: Mutex<HashMap<FileId, DepHash>>,
    /// Content hashes of entries evicted this session, so an evicted file is
    /// not mistaken for an edited one.
    evicted_hashes: Mutex<HashMap<FileId, String>>,
    /// Epoch this cache validates against (build fingerprint + PHP version).
    /// Written into the on-disk file and re-checked on the next `load`.
    epoch: u64,
    dirty: AtomicBool,
}

impl AnalysisCache {
    /// Open or create a cache stored under `cache_dir`.
    /// If the directory or cache file do not exist they are created lazily on
    /// the first `flush()` call.
    /// `user_stub_fp` is [`crate::stubs::user_stub_fingerprint`] for the
    /// session's user stubs (0 when none are configured).
    pub fn open(cache_dir: &Path, php_version: u8, user_stub_fp: u64) -> Self {
        std::fs::create_dir_all(cache_dir).ok();
        let epoch = cache_epoch(php_version, user_stub_fp);
        let disk = Self::load(cache_dir, epoch);

        // Map on-disk string indices to FileIds for O(1) u32-hash lookups at
        // runtime; a path recurs in many reverse-dependency sets.
        let mut id_map = FileIdMap::new();
        let mut file_ids: Vec<Option<FileId>> = vec![None; disk.strings.len()];
        let strings = &disk.strings;
        let mut file_id = |index: u32| -> Option<FileId> {
            let slot = file_ids.get_mut(index as usize)?;
            Some(*slot.get_or_insert_with(|| id_map.assign_or_get(&strings[index as usize])))
        };
        let mut entries: HashMap<FileId, CacheEntry> = HashMap::default();
        for e in disk.entries {
            let Some(id) = file_id(e.path) else { continue };
            let reference_locations = e
                .refs
                .iter()
                .filter_map(|&(sym, line, start, end)| {
                    Some((strings.get(sym as usize)?.clone(), line, start, end))
                })
                .collect();
            entries.insert(
                id,
                CacheEntry {
                    content_hash: e.content_hash,
                    issues: e.issues.into(),
                    reference_locations,
                    surface_hash: e.surface_hash,
                },
            );
        }
        let reverse_deps: HashMap<FileId, HashSet<FileId>> = disk
            .reverse_deps
            .iter()
            .filter_map(|(path, deps)| {
                let id = file_id(*path)?;
                Some((id, deps.iter().filter_map(|&d| file_id(d)).collect()))
            })
            .collect();
        let dep_hashes: HashMap<FileId, DepHash> = disk
            .dep_hashes
            .iter()
            .filter_map(|(path, hash)| Some((file_id(*path)?, *hash)))
            .collect();

        let unresolved_entries = entries
            .iter()
            .filter_map(|(&id, entry)| {
                crate::db::issues_have_unresolved_names(&entry.issues).then_some(id)
            })
            .collect();

        Self {
            cache_dir: cache_dir.to_path_buf(),
            file_id_map: Mutex::new(id_map),
            entries: Mutex::new(entries),
            unresolved_entries: Mutex::new(unresolved_entries),
            reverse_deps: Mutex::new(DepGraph {
                dependents: reverse_deps,
                dependencies: None,
            }),
            dep_hashes: Mutex::new(dep_hashes),
            evicted_hashes: Mutex::new(HashMap::default()),
            epoch,
            dirty: AtomicBool::new(false),
        }
    }

    /// Directory the cache was opened from. Useful for callers that want to
    /// open a sibling cache (e.g. the definition-collection `StubSliceCache`) under the
    /// same root.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Return cached issues and reference locations for `file_path` if its
    /// `content_hash` matches. Returns `None` if there is no entry or the file
    /// has changed. The second element of the tuple is the list of
    /// `(symbol_key, line, col_start, col_end)` entries to replay into the
    /// salsa db via `MirDatabase::set_file_reference_locations` (see
    /// `AnalysisSession::warm_start_files` for the session-level replay path).
    pub fn get(&self, file_path: &str, content_hash: &str) -> Option<CacheHit> {
        let id = self.file_id_map.lock().get(file_path)?;
        let entries = self.entries.lock();
        entries.get(&id).and_then(|e| {
            if e.content_hash == content_hash {
                // Arc clones: O(1) while holding the lock.
                Some((e.issues.clone(), e.reference_locations.clone()))
            } else {
                None
            }
        })
    }

    /// True if `file_path` has an entry stored for exactly `content_hash`.
    /// Presence-only variant of [`Self::get`] that skips cloning the entry.
    pub fn is_valid(&self, file_path: &str, content_hash: &str) -> bool {
        let Some(id) = self.file_id_map.lock().get(file_path) else {
            return false;
        };
        let entries = self.entries.lock();
        entries
            .get(&id)
            .is_some_and(|e| e.content_hash == content_hash)
    }

    /// Record a registered file's content and report whether it is unchanged
    /// since this cache last saw it, analyzed or as a dependency.
    pub fn observe_dependency(&self, file_path: &str, content: &str) -> bool {
        self.observe_content(file_path, &hash_content(content))
    }

    /// [`Self::observe_dependency`] for a known [`hash_content`] digest.
    pub fn observe_content(&self, file_path: &str, content_hex: &str) -> bool {
        let id = self.file_id_map.lock().assign_or_get(file_path);
        let hash = dep_hash(content_hex);
        if self.dep_hashes.lock().get(&id) == Some(&hash) {
            return true;
        }
        let entry_matches = self
            .entries
            .lock()
            .get(&id)
            .map(|e| e.content_hash == content_hex);
        let known = entry_matches.unwrap_or(false)
            || self
                .evicted_hashes
                .lock()
                .get(&id)
                .is_some_and(|h| h == content_hex);
        self.dep_hashes.lock().insert(id, hash);
        // `flush` keeps only hashes of files without an entry.
        if entry_matches.is_none() {
            self.dirty.store(true, Ordering::Relaxed);
        }
        known
    }

    fn remember_evicted(&self, evicted: impl IntoIterator<Item = (FileId, CacheEntry)>) {
        let mut hashes = self.evicted_hashes.lock();
        for (id, entry) in evicted {
            hashes.insert(id, entry.content_hash);
        }
    }

    /// Return the paths of every file that currently has a cache entry.
    /// Used to detect files that were analyzed in a previous run but have since
    /// been deleted, so their dependents can be invalidated.
    pub fn cached_files(&self) -> Vec<String> {
        let id_map = self.file_id_map.lock();
        let entries = self.entries.lock();
        entries
            .keys()
            .filter_map(|&id| id_map.path(id).map(|p| p.to_string()))
            .collect()
    }

    /// Store `issues` and `reference_locations` for `file_path` with the given
    /// `content_hash` and `surface_hash`. `reference_locations` is a list of
    /// `(symbol_key, line, col_start, col_end)` recorded during body analysis.
    pub fn put(
        &self,
        file_path: &str,
        content_hash: String,
        surface_hash: String,
        issues: std::sync::Arc<[Issue]>,
        reference_locations: std::sync::Arc<[CachedRefLoc]>,
    ) {
        let id = self.file_id_map.lock().assign_or_get(file_path);
        let mut entries = self.entries.lock();
        entries.insert(
            id,
            CacheEntry {
                content_hash,
                issues,
                reference_locations,
                surface_hash,
            },
        );
        let mut unresolved = self.unresolved_entries.lock();
        if crate::db::issues_have_unresolved_names(&entries[&id].issues) {
            unresolved.insert(id);
        } else {
            unresolved.remove(&id);
        }
        self.dirty.store(true, Ordering::Relaxed);
    }

    /// The stored cross-file surface digest for `file_path`, if an entry exists.
    /// `None` means no entry; an entry written before surface tracking returns
    /// an empty string (treated as "unknown" by callers).
    pub fn surface_hash(&self, file_path: &str) -> Option<String> {
        let id = self.file_id_map.lock().get(file_path)?;
        let entries = self.entries.lock();
        entries.get(&id).map(|e| e.surface_hash.clone())
    }

    /// Persist the in-memory cache to `{cache_dir}/cache.bin`.
    /// This is a no-op if nothing changed since the last flush.
    pub fn flush(&self) {
        let was_dirty = self.dirty.swap(false, Ordering::Relaxed);
        if !was_dirty {
            return;
        }
        let cache_file = self.cache_dir.join("cache.bin");
        let id_map = self.file_id_map.lock();
        let entries_guard = self.entries.lock();
        let deps_guard = self.reverse_deps.lock();

        let mut table = StringTable::default();
        let entries: Vec<DiskEntryView> = entries_guard
            .iter()
            .filter_map(|(&id, entry)| {
                let path = table.id(id_map.path(id)?);
                let refs = entry
                    .reference_locations
                    .iter()
                    .map(|(sym, line, start, end)| (table.id(sym), *line, *start, *end))
                    .collect();
                Some(DiskEntryView {
                    path,
                    content_hash: &entry.content_hash,
                    surface_hash: &entry.surface_hash,
                    issues: &entry.issues,
                    refs,
                })
            })
            .collect();
        let reverse_deps: Vec<(u32, Vec<u32>)> = deps_guard
            .dependents
            .iter()
            .filter_map(|(&id, dep_ids)| {
                let path = table.id(id_map.path(id)?);
                let deps = dep_ids
                    .iter()
                    .filter_map(|&dep| Some(table.id(id_map.path(dep)?)))
                    .collect();
                Some((path, deps))
            })
            .collect();
        // An entry already records its file's content.
        let dep_hashes_guard = self.dep_hashes.lock();
        let dep_hashes: Vec<(u32, &DepHash)> = dep_hashes_guard
            .iter()
            .filter(|(id, _)| !entries_guard.contains_key(id))
            .filter_map(|(&id, hash)| Some((table.id(id_map.path(id)?), hash)))
            .collect();

        let view = CacheFileView {
            version: self.epoch,
            strings: table.strings,
            entries,
            reverse_deps,
            dep_hashes,
        };
        if let Ok(bytes) = bincode::serialize(&view) {
            // Tempfile in the same directory so the rename is atomic on every
            // POSIX filesystem (cross-mount renames would degrade to copy) —
            // same pattern as stub_cache.rs. A crash mid-write leaves only
            // the tmp file; the prior cache.bin (or none, on a first run)
            // stays intact instead of being truncated in place.
            let tmp = cache_file.with_extension(format!("bin.tmp.{}", crate::tmp_suffix::next()));
            if std::fs::write(&tmp, &bytes).is_ok() {
                let _ = std::fs::rename(&tmp, &cache_file);
            }
        }
    }

    /// Replace the reverse dependency graph (called after each definition collection).
    pub fn set_reverse_deps(&self, deps: HashMap<String, HashSet<String>>) {
        let mut id_map = self.file_id_map.lock();
        let converted: HashMap<FileId, HashSet<FileId>> = deps
            .into_iter()
            .map(|(path, dep_paths)| {
                let id = id_map.assign_or_get(&path);
                let dep_ids = dep_paths.iter().map(|p| id_map.assign_or_get(p)).collect();
                (id, dep_ids)
            })
            .collect();
        drop(id_map);
        *self.reverse_deps.lock() = DepGraph {
            dependents: converted,
            dependencies: None,
        };
        self.dirty.store(true, Ordering::Relaxed);
    }

    /// Update the reverse-dep graph for a single file in place.
    ///
    /// `new_targets` is the set of files `file` depends on (its imports'
    /// defining files plus parent / interfaces / traits' defining files).
    /// This replaces `file`'s outgoing edges, keeping the graph's other edges.
    pub fn update_reverse_deps_for_file(&self, file: &str, new_targets: &HashSet<String>) {
        self.set_file_dependencies(file, new_targets.iter().map(String::as_str));
    }

    /// Replace `file`'s outgoing edges with `targets`; costs `file`'s old and
    /// new degree, not the graph's size.
    pub(crate) fn set_file_dependencies<'a>(
        &self,
        file: &str,
        targets: impl IntoIterator<Item = &'a str>,
    ) {
        self.update_file_dependencies(file, targets, false);
    }

    /// Add `targets` to `file`'s outgoing edges, keeping the existing ones.
    pub(crate) fn add_file_dependencies<'a>(
        &self,
        file: &str,
        targets: impl IntoIterator<Item = &'a str>,
    ) {
        self.update_file_dependencies(file, targets, true);
    }

    fn update_file_dependencies<'a>(
        &self,
        file: &str,
        targets: impl IntoIterator<Item = &'a str>,
        keep_old: bool,
    ) {
        let (file_id, mut target_ids) = {
            let mut id_map = self.file_id_map.lock();
            let file_id = id_map.assign_or_get(file);
            let target_ids: Vec<FileId> = targets
                .into_iter()
                .map(|t| id_map.assign_or_get(t))
                .filter(|&t| t != file_id)
                .collect();
            (file_id, target_ids)
        };
        target_ids.sort_unstable();
        target_ids.dedup();

        let mut graph = self.reverse_deps.lock();
        let DepGraph {
            dependents,
            dependencies,
        } = &mut *graph;
        let dependencies = dependencies.get_or_insert_with(|| invert(dependents));
        if keep_old {
            if let Some(old) = dependencies.get(&file_id) {
                target_ids.extend_from_slice(old);
                target_ids.sort_unstable();
                target_ids.dedup();
            }
        }
        let old = if target_ids.is_empty() {
            dependencies.remove(&file_id)
        } else {
            dependencies.insert(file_id, target_ids.clone())
        }
        .unwrap_or_default();
        if old == target_ids {
            return;
        }
        for target in old {
            if let Some(set) = dependents.get_mut(&target) {
                set.remove(&file_id);
                if set.is_empty() {
                    dependents.remove(&target);
                }
            }
        }
        for target in target_ids {
            dependents.entry(target).or_default().insert(file_id);
        }
        self.dirty.store(true, Ordering::Relaxed);
    }

    /// BFS from each changed file through the reverse dep graph.
    /// Evicts every reachable dependent's cache entry.
    /// Returns the number of entries evicted.
    pub fn evict_with_dependents(&self, changed_files: &[String]) -> usize {
        self.evict_reachable(changed_files, false)
    }

    /// Evict the changed files themselves as well as every known dependent.
    /// Mirror writes use this because their file's cached result can be stale
    /// even before the symbol-index reconciliation runs.
    pub fn evict_files_and_dependents(&self, changed_files: &[String]) -> usize {
        self.evict_reachable(changed_files, true)
    }

    fn evict_reachable(&self, changed_files: &[String], include_changed: bool) -> usize {
        self.evict_reachable_ids(changed_files, include_changed).1
    }

    /// [`Self::evict_files_and_dependents`] (or, without `include_changed`,
    /// [`Self::evict_with_dependents`]), returning every transitive dependent
    /// reached, whether or not it still had an entry.
    pub(crate) fn evict_reporting_dependents(
        &self,
        changed_files: &[String],
        include_changed: bool,
    ) -> Vec<String> {
        let (dependents, _) = self.evict_reachable_ids(changed_files, include_changed);
        if dependents.is_empty() {
            return Vec::new();
        }
        let id_map = self.file_id_map.lock();
        dependents
            .iter()
            .filter_map(|&id| id_map.path(id).map(str::to_string))
            .collect()
    }

    /// `(dependents reached, entries evicted)`.
    fn evict_reachable_ids(
        &self,
        changed_files: &[String],
        include_changed: bool,
    ) -> (Vec<FileId>, usize) {
        // Resolve paths to FileIds; skip unknown files (no cache entry → nothing to evict).
        let seed_ids: Vec<FileId> = {
            let id_map = self.file_id_map.lock();
            changed_files.iter().filter_map(|p| id_map.get(p)).collect()
        };
        if seed_ids.is_empty() {
            return (Vec::new(), 0);
        }

        // Phase 1: collect all dependents via BFS (lock held only here).
        let dependents: Vec<FileId> = {
            let graph = self.reverse_deps.lock();
            let mut visited: HashSet<FileId> = seed_ids.iter().copied().collect();
            let mut queue: std::collections::VecDeque<FileId> = seed_ids.iter().copied().collect();
            let mut result = Vec::new();
            while let Some(id) = queue.pop_front() {
                if let Some(next) = graph.dependents.get(&id) {
                    for &dep_id in next {
                        if visited.insert(dep_id) {
                            queue.push_back(dep_id);
                            result.push(dep_id);
                        }
                    }
                }
            }
            result
        };
        let seeds = if include_changed { &seed_ids[..] } else { &[] };
        let to_evict = || seeds.iter().chain(&dependents);

        // Phase 2: evict (reverse_deps lock released above).
        let mut entries = self.entries.lock();
        let evicted: Vec<(FileId, CacheEntry)> = to_evict()
            .filter_map(|id| entries.remove(id).map(|e| (*id, e)))
            .collect();
        let count = evicted.len();
        let mut unresolved = self.unresolved_entries.lock();
        for id in to_evict() {
            unresolved.remove(id);
        }
        drop((entries, unresolved));
        self.remember_evicted(evicted);
        if count > 0 {
            self.dirty.store(true, Ordering::Relaxed);
        }
        (dependents, count)
    }

    /// Files this cache holds a result for or records as a dependency
    /// target, for spotting ones deleted since the run that recorded them.
    pub(crate) fn known_files(&self) -> Vec<String> {
        let id_map = self.file_id_map.lock();
        let entries = self.entries.lock();
        let graph = self.reverse_deps.lock();
        let mut ids: HashSet<FileId> = entries.keys().copied().collect();
        ids.extend(graph.dependents.keys().copied());
        ids.into_iter()
            .filter_map(|id| id_map.path(id).map(str::to_string))
            .collect()
    }

    pub fn has_unresolved(&self) -> bool {
        !self.unresolved_entries.lock().is_empty()
    }

    /// Evict results with missing workspace names. Such files have no reverse
    /// dependency edge to a newly registered definition yet.
    pub fn evict_unresolved(&self) -> usize {
        let mut entries = self.entries.lock();
        let mut unresolved = self.unresolved_entries.lock();
        let count = unresolved.len();
        let evicted: Vec<(FileId, CacheEntry)> = unresolved
            .drain()
            .filter_map(|id| entries.remove(&id).map(|e| (id, e)))
            .collect();
        drop((entries, unresolved));
        self.remember_evicted(evicted);
        if count > 0 {
            self.dirty.store(true, Ordering::Relaxed);
        }
        count
    }

    /// Evict results with a missing name that `declared` may now provide.
    pub fn evict_unresolved_declaring(&self, declared: &DeclaredNames) -> Vec<String> {
        if declared.is_empty() {
            return Vec::new();
        }
        self.evict_unresolved_where(|issue| declared.may_resolve(issue))
    }

    /// Visit every issue of the results that report a missing name, without
    /// the cache's locks held.
    pub fn for_each_unresolved_issue(&self, mut visit: impl FnMut(&Issue)) {
        let issues: Vec<std::sync::Arc<[Issue]>> = {
            let entries = self.entries.lock();
            let unresolved = self.unresolved_entries.lock();
            unresolved
                .iter()
                .filter_map(|id| entries.get(id).map(|e| e.issues.clone()))
                .collect()
        };
        for issue in issues.iter().flat_map(|i| i.iter()) {
            visit(issue);
        }
    }

    /// Evict results with an issue matching `stale`; returns their paths.
    /// `stale` runs without the cache's locks held, so it may call back in.
    pub fn evict_unresolved_where(&self, stale: impl Fn(&Issue) -> bool) -> Vec<String> {
        let candidates: Vec<(FileId, Option<std::sync::Arc<[Issue]>>)> = {
            let entries = self.entries.lock();
            let unresolved = self.unresolved_entries.lock();
            unresolved
                .iter()
                .map(|id| (*id, entries.get(id).map(|e| e.issues.clone())))
                .collect()
        };
        let stale_ids: Vec<FileId> = candidates
            .into_iter()
            .filter(|(_, issues)| issues.as_ref().is_none_or(|i| i.iter().any(&stale)))
            .map(|(id, _)| id)
            .collect();
        let mut entries = self.entries.lock();
        let mut unresolved = self.unresolved_entries.lock();
        let evicted: Vec<(FileId, CacheEntry)> = stale_ids
            .iter()
            .filter_map(|id| {
                unresolved.remove(id);
                entries.remove(id).map(|e| (*id, e))
            })
            .collect();
        drop((entries, unresolved));
        let paths: Vec<String> = {
            let id_map = self.file_id_map.lock();
            evicted
                .iter()
                .filter_map(|(id, _)| id_map.path(*id).map(str::to_string))
                .collect()
        };
        if !evicted.is_empty() {
            self.dirty.store(true, Ordering::Relaxed);
        }
        self.remember_evicted(evicted);
        paths
    }

    /// Remove a single file's cache entry.
    pub fn evict(&self, file_path: &str) {
        let Some(id) = self.file_id_map.lock().get(file_path) else {
            return;
        };
        let mut entries = self.entries.lock();
        if let Some(entry) = entries.remove(&id) {
            self.unresolved_entries.lock().remove(&id);
            drop(entries);
            self.remember_evicted([(id, entry)]);
            self.dirty.store(true, Ordering::Relaxed);
        }
    }

    // -----------------------------------------------------------------------

    fn load(cache_dir: &Path, epoch: u64) -> CacheFile {
        let empty = || CacheFile {
            _version: epoch,
            strings: Vec::new(),
            entries: Vec::new(),
            reverse_deps: Vec::new(),
            dep_hashes: Vec::new(),
        };
        let Ok(bytes) = std::fs::read(cache_dir.join("cache.bin")) else {
            return empty();
        };
        // A file from another analyzer/stub set/PHP version may hold stale
        // results whose content hashes still match; skip it undecoded.
        if bytes.get(..8) != Some(&epoch.to_le_bytes()[..]) {
            return empty();
        }
        // Bound the read to the file's own size: a bit-flipped cache.bin can
        // desync bincode's length-prefixed decoding into a bogus multi-gigabyte
        // collection length, allocated before deserialize() can fail. `config()`
        // (not `options()`) matches the fixint encoding `serialize` writes;
        // `options()` defaults to varint and silently misreads it.
        #[allow(deprecated)]
        let cfg = bincode::config().limit(bytes.len() as u64).clone();
        cfg.deserialize::<CacheFile>(&bytes)
            .unwrap_or_else(|_| empty())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Fixed PHP version for cache tests — any value works as long as the
    /// open/write/reopen calls within one test agree on it.
    const TEST_PHP_V: u8 = 0;

    fn make_cache(dir: &TempDir) -> AnalysisCache {
        AnalysisCache::open(dir.path(), TEST_PHP_V, 0)
    }

    fn seed(cache: &AnalysisCache, file: &str) {
        cache.put(
            file,
            "hash".to_string(),
            String::new(),
            [].into(),
            [].into(),
        );
    }

    fn surface(src: &str) -> String {
        let parsed = php_rs_parser::parse(src);
        surface_fingerprint(src, &parsed.program)
    }

    #[test]
    fn surface_stable_across_declared_return_method_body_edit() {
        let a = "<?php\nclass C { public function f(): int { return 1; } }\n";
        let b = "<?php\nclass C { public function f(): int { $x = 2; return $x; } }\n";
        assert_eq!(
            surface(a),
            surface(b),
            "declared-return body edits must not change the surface"
        );
    }

    #[test]
    fn surface_stable_across_declared_return_body_line_shift() {
        // The common edit: adding lines inside a body shifts every later
        // declaration's position. The surface must ignore that.
        let a = "<?php\nfunction f(): int { return 1; }\nfunction g(): int { return 2; }\n";
        let b = "<?php\nfunction f(): int {\n    $a = 0;\n    $b = 1;\n    return $a + $b;\n}\nfunction g(): int { return 2; }\n";
        assert_eq!(surface(a), surface(b));
    }

    #[test]
    fn surface_stable_across_comment_and_whitespace_edits() {
        let base = surface("<?php\nclass C { public function f() { return 1; } }\n");
        for edited in [
            "<?php\nclass C { public function f() { return 1; } }\n// note\n",
            "<?php\n# note\nclass C { public function f() { return 1; } }\n",
            "<?php\n/* note */\nclass C { public function f() { return 1; } }\n",
            "<?php\nclass C { public function f() { /* note */ return 1; } }\n",
            "<?php\n\n\nclass C {\n    public function f() {\n        return 1;\n    }\n}\n",
        ] {
            assert_eq!(base, surface(edited), "{edited}");
        }
    }

    #[test]
    fn surface_changes_on_annotations_and_literal_text() {
        let pairs = [
            (
                "<?php\n/** @return int */\nfunction f() {}",
                "<?php\n/** @return string */\nfunction f() {}",
            ),
            (
                "<?php\n/** Old. */\nfunction f() {}",
                "<?php\n/** New. */\nfunction f() {}",
            ),
            (
                "<?php\n/* @var int */\nfunction f() {}",
                "<?php\n/* @var string */\nfunction f() {}",
            ),
            (
                "<?php\n#[A]\nfunction f() {}",
                "<?php\n#[B]\nfunction f() {}",
            ),
            ("<?php\nconst A = 'x // y';", "<?php\nconst A = 'x // z';"),
            ("<?php\nconst A = 'a  b';", "<?php\nconst A = 'a b';"),
            (
                "<?php\nconst A = <<<T\n a\nT;",
                "<?php\nconst A = <<<T\n b\nT;",
            ),
            (
                "<?php\nconst A = <<<'T'\n a // x\nT;",
                "<?php\nconst A = <<<'T'\n a // y\nT;",
            ),
            (
                "<?php\nfunction f() { return \"$a b\"; }",
                "<?php\nfunction f() { return \"$a c\"; }",
            ),
            ("a <?php function f() {}", "b <?php function f() {}"),
            ("<?php function f() {} ?> a", "<?php function f() {} ?> b"),
        ];
        for (a, b) in pairs {
            assert_ne!(surface(a), surface(b), "{a} / {b}");
        }
    }

    #[test]
    fn surface_of_an_unlexable_file_hashes_every_byte() {
        let a = "<?php\n/* unterminated\nfunction f() {}\n";
        let b = "<?php\n/* unterminated\nfunction g() {}\n";
        assert_ne!(surface(a), surface(b));
    }

    #[test]
    fn surface_changes_on_return_type_edit() {
        let a = "<?php\nclass C { public function f(): int { return 1; } }\n";
        let b = "<?php\nclass C { public function f(): string { return 1; } }\n";
        assert_ne!(surface(a), surface(b));
    }

    #[test]
    fn surface_changes_on_param_type_edit() {
        let a = "<?php\nclass C { public function f(int $x): int { return $x; } }\n";
        let b = "<?php\nclass C { public function f(string $x): int { return $x; } }\n";
        assert_ne!(surface(a), surface(b));
    }

    #[test]
    fn surface_changes_on_untyped_method_body_edit() {
        // No declared return type: the body feeds the inferred return type,
        // which is observable across files, so its bytes are kept.
        let a = "<?php\nclass C { public function f() { return 1; } }\n";
        let b = "<?php\nclass C { public function f() { return 'x'; } }\n";
        assert_ne!(surface(a), surface(b));
    }

    #[test]
    fn surface_changes_on_constructor_body_edit() {
        // Constructors have no declared return type — their body is kept.
        let a = "<?php\nclass C { public function __construct() { $this->x = 1; } }\n";
        let b = "<?php\nclass C { public function __construct() { $this->x = 2; } }\n";
        assert_ne!(surface(a), surface(b));
    }

    #[test]
    fn per_file_dependencies_replace_old_edges_and_persist() {
        let dir = TempDir::new().unwrap();
        {
            let cache = make_cache(&dir);
            for f in ["a", "b", "caller"] {
                seed(&cache, f);
            }
            cache.set_file_dependencies("caller", ["a"]);
            cache.set_file_dependencies("caller", ["b"]);
            cache.flush();
        }
        let cache = make_cache(&dir);
        assert!(cache
            .evict_reporting_dependents(&["a".to_string()], false)
            .is_empty());
        assert_eq!(
            cache.evict_reporting_dependents(&["b".to_string()], true),
            ["caller"]
        );
        assert!(cache.get("caller", "hash").is_none());
        assert!(cache.get("a", "hash").is_some());
    }

    #[test]
    fn evict_with_dependents_linear_chain() {
        // reverse_deps: A → [B], B → [C]
        // Changing A must evict B and C.
        let dir = TempDir::new().unwrap();
        let cache = make_cache(&dir);
        seed(&cache, "A");
        seed(&cache, "B");
        seed(&cache, "C");

        let mut deps: HashMap<String, HashSet<String>> = HashMap::default();
        deps.entry("A".into()).or_default().insert("B".into());
        deps.entry("B".into()).or_default().insert("C".into());
        cache.set_reverse_deps(deps);

        let evicted = cache.evict_with_dependents(&["A".to_string()]);

        assert_eq!(evicted, 2, "B and C should be evicted");
        assert!(cache.get("A", "hash").is_some(), "A itself is not evicted");
        assert!(cache.get("B", "hash").is_none(), "B should be evicted");
        assert!(cache.get("C", "hash").is_none(), "C should be evicted");
    }

    #[test]
    fn evict_with_dependents_diamond() {
        // reverse_deps: A → [B, C], B → [D], C → [D]
        // D should be evicted exactly once (visited set prevents double-eviction).
        let dir = TempDir::new().unwrap();
        let cache = make_cache(&dir);
        seed(&cache, "A");
        seed(&cache, "B");
        seed(&cache, "C");
        seed(&cache, "D");

        let mut deps: HashMap<String, HashSet<String>> = HashMap::default();
        deps.entry("A".into()).or_default().insert("B".into());
        deps.entry("A".into()).or_default().insert("C".into());
        deps.entry("B".into()).or_default().insert("D".into());
        deps.entry("C".into()).or_default().insert("D".into());
        cache.set_reverse_deps(deps);

        let evicted = cache.evict_with_dependents(&["A".to_string()]);

        assert_eq!(evicted, 3, "B, C, D each evicted once");
        assert!(cache.get("D", "hash").is_none());
    }

    #[test]
    fn evict_with_dependents_cycle_safety() {
        // reverse_deps: A → [B], B → [A]  (circular)
        // Must not loop forever; B should be evicted.
        let dir = TempDir::new().unwrap();
        let cache = make_cache(&dir);
        seed(&cache, "A");
        seed(&cache, "B");

        let mut deps: HashMap<String, HashSet<String>> = HashMap::default();
        deps.entry("A".into()).or_default().insert("B".into());
        deps.entry("B".into()).or_default().insert("A".into());
        cache.set_reverse_deps(deps);

        let evicted = cache.evict_with_dependents(&["A".to_string()]);

        // B is a dependent of A; A is the seed (not counted as "evicted dependent")
        assert_eq!(evicted, 1);
        assert!(cache.get("B", "hash").is_none());
    }

    #[test]
    fn evict_with_dependents_unrelated_file_untouched() {
        // Changing C should not evict B (which depends on A, not C).
        let dir = TempDir::new().unwrap();
        let cache = make_cache(&dir);
        seed(&cache, "A");
        seed(&cache, "B");
        seed(&cache, "C");

        let mut deps: HashMap<String, HashSet<String>> = HashMap::default();
        deps.entry("A".into()).or_default().insert("B".into());
        cache.set_reverse_deps(deps);

        let evicted = cache.evict_with_dependents(&["C".to_string()]);

        assert_eq!(evicted, 0);
        assert!(
            cache.get("B", "hash").is_some(),
            "B unrelated, should survive"
        );
    }

    #[test]
    fn flush_writes_atomically_leaving_no_tmp_file() {
        // flush() must go through tempfile-in-same-dir + rename, not a plain
        // in-place fs::write: a crash mid-write must leave the prior
        // cache.bin (or none) intact rather than a truncated file. This
        // can't directly simulate a crash, but it pins the observable
        // contract: after a successful flush, cache.bin exists with the
        // right content and no stray `.tmp.*` sibling is left behind.
        let dir = TempDir::new().unwrap();
        {
            let cache = make_cache(&dir);
            seed(&cache, "a.php");
            cache.flush();
        }
        let cache_file = dir.path().join("cache.bin");
        assert!(cache_file.is_file(), "cache.bin must exist after flush");

        let leftover: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".tmp."))
            .collect();
        assert!(
            leftover.is_empty(),
            "flush left a tmp file behind: {leftover:?}"
        );

        let reopened = AnalysisCache::open(dir.path(), TEST_PHP_V, 0);
        assert!(
            reopened.get("a.php", "hash").is_some(),
            "content must survive the atomic write"
        );
    }

    #[test]
    fn entries_survive_reopen_with_matching_epoch() {
        // Sanity: a flush + reopen within the same build (same epoch) keeps
        // entries. Guards against the epoch check being over-eager.
        let dir = TempDir::new().unwrap();
        {
            let cache = make_cache(&dir);
            cache.put(
                "a.php",
                "h1".to_string(),
                String::new(),
                [].into(),
                [].into(),
            );
            cache.flush();
        }
        let cache = AnalysisCache::open(dir.path(), TEST_PHP_V, 0);
        assert!(
            cache.get("a.php", "h1").is_some(),
            "entry written by the same build/stub set must survive a reopen"
        );
    }

    #[test]
    fn stale_epoch_discards_entire_cache() {
        // A cache.bin written under a different analyzer/stub epoch must be
        // discarded on load even though the per-file content hash still
        // matches: the guard for stale `UndefinedClass` after a stub-set change.
        let dir = TempDir::new().unwrap();
        {
            let cache = make_cache(&dir);
            seed(&cache, "a.php");
            cache.flush();
        }
        let path = dir.path().join("cache.bin");
        let mut bytes = std::fs::read(&path).unwrap();
        bytes[0] ^= 1;
        std::fs::write(&path, bytes).unwrap();

        let cache = AnalysisCache::open(dir.path(), TEST_PHP_V, 0);
        assert!(
            cache.get("a.php", "hash").is_none(),
            "entry from a mismatched epoch must not be served despite a matching content hash"
        );
    }

    #[test]
    fn switching_php_version_discards_cache() {
        // Results written targeting one PHP version must not be served when the
        // next run targets a different one: version-gated symbols resolve
        // differently though the file's content is identical. Regression guard
        // for the `--php-version` arm of the stale-cache family.
        let dir = TempDir::new().unwrap();
        {
            let cache = AnalysisCache::open(dir.path(), 74, 0); // analyzed as PHP 7.4
            cache.put(
                "a.php",
                "h1".to_string(),
                String::new(),
                [].into(),
                [].into(),
            );
            cache.flush();
        }
        let same = AnalysisCache::open(dir.path(), 74, 0);
        assert!(
            same.get("a.php", "h1").is_some(),
            "same PHP version must reuse the cache"
        );
        let other = AnalysisCache::open(dir.path(), 80, 0); // now PHP 8.0
        assert!(
            other.get("a.php", "h1").is_none(),
            "a different PHP version must discard the cache, not serve stale results"
        );
    }

    #[test]
    fn changing_user_stub_fingerprint_discards_cache() {
        // User stubs are resolvable like bundled ones, so editing/adding/removing
        // them changes analysis output for dependent files. The user-stub
        // fingerprint is folded into the epoch; a different one must discard.
        let dir = TempDir::new().unwrap();
        {
            let cache = AnalysisCache::open(dir.path(), TEST_PHP_V, 0xAAAA);
            cache.put(
                "a.php",
                "h1".to_string(),
                String::new(),
                [].into(),
                [].into(),
            );
            cache.flush();
        }
        let same = AnalysisCache::open(dir.path(), TEST_PHP_V, 0xAAAA);
        assert!(
            same.get("a.php", "h1").is_some(),
            "identical user-stub fingerprint must reuse the cache"
        );
        let changed = AnalysisCache::open(dir.path(), TEST_PHP_V, 0xBBBB);
        assert!(
            changed.get("a.php", "h1").is_none(),
            "a changed user-stub fingerprint must discard the cache"
        );
    }

    #[test]
    fn truncated_or_garbled_cache_bin_is_discarded_not_paniced() {
        let dir = TempDir::new().unwrap();
        {
            let cache = make_cache(&dir);
            seed(&cache, "a.php");
            cache.flush();
        }
        let path = dir.path().join("cache.bin");
        let bytes = std::fs::read(&path).unwrap();
        let mut garbled = bytes.clone();
        garbled[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
        for broken in [bytes[..bytes.len() - 1].to_vec(), garbled] {
            std::fs::write(&path, broken).unwrap();
            let cache = AnalysisCache::open(dir.path(), TEST_PHP_V, 0);
            assert!(cache.get("a.php", "hash").is_none());
        }
    }

    #[test]
    fn reopen_restores_reference_locations_issues_and_reverse_deps() {
        let dir = TempDir::new().unwrap();
        {
            let cache = make_cache(&dir);
            let sym: std::sync::Arc<str> = std::sync::Arc::from("cls:Foo");
            let issue = Issue::new(
                mir_issues::IssueKind::UndefinedClass {
                    name: "Foo".to_string(),
                },
                mir_types::Location {
                    file: std::sync::Arc::from("a.php"),
                    line: 3,
                    line_end: 3,
                    col_start: 1,
                    col_end: 4,
                },
            );
            cache.put(
                "a.php",
                "h1".to_string(),
                "s1".to_string(),
                vec![issue].into(),
                vec![(sym.clone(), 3, 1, 4), (sym, 9, 2, 5)].into(),
            );
            let mut deps: HashMap<String, HashSet<String>> = HashMap::default();
            deps.insert("b.php".into(), ["a.php".to_string()].into_iter().collect());
            cache.set_reverse_deps(deps);
            cache.observe_content("vendor/v.php", &"ab".repeat(32));
            cache.flush();
        }
        let cache = AnalysisCache::open(dir.path(), TEST_PHP_V, 0);
        let (issues, refs) = cache.get("a.php", "h1").expect("hit");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].location.line, 3);
        assert_eq!(refs.len(), 2);
        assert_eq!(
            (refs[1].0.as_ref(), refs[1].1, refs[1].2, refs[1].3),
            ("cls:Foo", 9, 2, 5)
        );
        assert!(
            std::sync::Arc::ptr_eq(&refs[0].0, &refs[1].0),
            "a symbol is decoded once and shared"
        );
        assert_eq!(cache.surface_hash("a.php").as_deref(), Some("s1"));
        assert!(cache.observe_content("vendor/v.php", &"ab".repeat(32)));
        assert_eq!(cache.evict_with_dependents(&["b.php".to_string()]), 1);
    }

    #[test]
    fn build_fingerprint_tracks_binary_identity() {
        // A different mir build (different executable bytes) must yield a
        // different fingerprint, so its cache epoch differs and an old build's
        // cache is never reused. Two distinct byte images stand in for two
        // builds; identical bytes must reproduce the same fingerprint.
        let build_a = compute_build_fingerprint(Some(b"mir-binary-image-A"));
        let build_a_again = compute_build_fingerprint(Some(b"mir-binary-image-A"));
        let build_b = compute_build_fingerprint(Some(b"mir-binary-image-B"));
        assert_eq!(
            build_a, build_a_again,
            "same binary bytes → same fingerprint"
        );
        assert_ne!(
            build_a, build_b,
            "a new mir build (different bytes) must change the fingerprint"
        );
        // The fallback path (executable unreadable) is also deterministic and
        // distinct from a real binary hash.
        assert_eq!(
            compute_build_fingerprint(None),
            compute_build_fingerprint(None)
        );
    }
}
