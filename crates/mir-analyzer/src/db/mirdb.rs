use std::collections::{HashMap, HashSet};

use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use std::sync::Arc;

use mir_codebase::storage::{
    ConstantStorage, FunctionStorage, Location, MethodStorage, PropertyStorage, TemplateParam,
    Visibility,
};
use mir_codebase::StubSlice;
use mir_types::Union;

use super::*;

// MirDb concrete database

/// Concrete in-process Salsa database.
///
/// `Clone` is required for parallel batch analysis: salsa's supported
/// pattern for sharing a db across threads is to give each worker its
/// own clone (each clone gets a fresh `ZalsaLocal`, sharing the
/// underlying memoization storage).  Sharing `&MirDb` across threads is
/// **not** supported because `salsa::Database: Send` (not `Sync`).
type MemberRegistry<V> = Arc<FxHashMap<Arc<str>, FxHashMap<Arc<str>, V>>>;
type ReferenceLocations = Arc<Mutex<FxHashMap<Arc<str>, Vec<(Arc<str>, u32, u16, u16)>>>>;
/// Forward index: file path → set of symbol keys that file references.
/// Kept in sync with `reference_locations` for O(degree) lookups.
type FileReferences = Arc<Mutex<FxHashMap<Arc<str>, HashSet<Arc<str>>>>>;
/// Reverse reference index: symbol key → set of files that reference it.
/// Transpose of `FileReferences`; maintained in lockstep so deletions can
/// find referencing files in O(1) even after the symbol's defining file entry
/// has been removed from `symbol_to_file`.
type SymbolReferencers = Arc<Mutex<FxHashMap<Arc<str>, HashSet<Arc<str>>>>>;
/// Forward index: file path → set of symbol FQNs it defines.
/// Maintained in lockstep with `symbol_to_file` so `remove_file_definitions`
/// can find a file's symbols in O(symbols_in_file) instead of O(total_symbols).
type FileDefinedSymbols = Arc<Mutex<FxHashMap<Arc<str>, HashSet<Arc<str>>>>>;

/// Per-clone staging buffer for reference locations recorded during a parallel
/// Pass 2 worker.  `record_reference_location` pushes here instead of directly
/// into the shared `Arc<Mutex<...>>` maps, eliminating cross-thread contention.
/// After the parallel phase the owner calls `take_pending_ref_locs` and commits
/// the batch serially via `commit_reference_locations_batch`.
///
/// The custom `Clone` impl returns a *new empty buffer* so that each `MirDb`
/// worker clone starts fresh — we do NOT propagate one clone's pending entries
/// to another worker.
#[derive(Default)]
struct PendingRefLocs(Mutex<Vec<super::reference_locations::RefLoc>>);

impl Clone for PendingRefLocs {
    fn clone(&self) -> Self {
        Self::default()
    }
}

#[salsa::db]
#[derive(Default, Clone)]
pub struct MirDb {
    storage: salsa::Storage<Self>,
    // Keep registries behind `Arc`s so `MirDb::clone()` stays cheap for
    // parallel analysis workers. The salsa storage is already shared by clone;
    // these maps only hold stable input handles, so copy-on-write insertion is
    // enough for the canonical mutable db paths.
    /// FQCN → ClassNode handle registry (not tracked by Salsa; see
    /// `lookup_class_node` for the rationale). Keys are canonical FQCNs;
    /// case-insensitive lookups go through `class_node_keys_lower`.
    class_nodes: Arc<FxHashMap<Arc<str>, ClassNode>>,
    /// Lowercased FQCN → canonical FQCN. Maintained in lockstep with
    /// `class_nodes` so callers can resolve PHP's case-insensitive class
    /// names (`new arrayobject()` → `ArrayObject`).
    class_node_keys_lower: Arc<FxHashMap<String, Arc<str>>>,
    /// FQN → FunctionNode handle registry. Keys are canonical FQNs;
    /// case-insensitive lookups go through `function_node_keys_lower`.
    function_nodes: Arc<FxHashMap<Arc<str>, FunctionNode>>,
    /// Lowercased FQN → canonical FQN. Maintained in lockstep with
    /// `function_nodes` so callers can resolve PHP's case-insensitive
    /// function names (`STRLEN($x)` → `strlen`).
    function_node_keys_lower: Arc<FxHashMap<String, Arc<str>>>,
    /// (owner FQCN) → (method_name_lower → MethodNode) handle registry.
    method_nodes: MemberRegistry<MethodNode>,
    /// (owner FQCN) → (prop_name → PropertyNode) handle registry.
    property_nodes: MemberRegistry<PropertyNode>,
    /// (owner FQCN) → (const_name → ClassConstantNode) handle registry.
    class_constant_nodes: MemberRegistry<ClassConstantNode>,
    /// FQN → GlobalConstantNode handle registry.
    global_constant_nodes: Arc<FxHashMap<Arc<str>, GlobalConstantNode>>,
    /// File path → first declared namespace.
    file_namespaces: Arc<FxHashMap<Arc<str>, Arc<str>>>,
    /// File path → use-alias imports.
    file_imports: Arc<FxHashMap<Arc<str>, HashMap<String, String>>>,
    /// Global variable name (without `$`) → collected type.
    global_vars: Arc<FxHashMap<Arc<str>, Union>>,
    /// Symbol FQN → defining file.
    symbol_to_file: Arc<FxHashMap<Arc<str>, Arc<str>>>,
    /// Forward index: file → set of symbol FQNs it defines.
    /// Maintained in lockstep with `symbol_to_file` so `remove_file_definitions`
    /// can find a file's symbols in O(symbols_in_file) instead of O(total_symbols).
    file_to_defined_symbols: FileDefinedSymbols,
    /// Public symbol key → reference locations.
    reference_locations: ReferenceLocations,
    /// Forward index: file → set of symbol keys it references. Kept in sync
    /// with `reference_locations` to provide O(degree) dep-graph lookups.
    file_references: FileReferences,
    /// Reverse reference index: symbol key → set of files that reference it.
    /// Maintained in lockstep with `file_references`.
    symbol_referencers: SymbolReferencers,
    /// Per-clone staging area for reference locations.  Workers push here
    /// during parallel analysis; the orchestrator drains and commits serially.
    pending_ref_locs: PendingRefLocs,
    /// File path → Salsa SourceFile input handle.
    source_files: Arc<FxHashMap<Arc<str>, SourceFile>>,
    /// Side-channel resolver state. The `ResolverConfig` salsa input is
    /// lazily created on first `set_resolver` call; its `revision` is
    /// bumped on every subsequent change so dependent tracked queries
    /// (e.g. `resolve_fqcn_to_path`) are invalidated. The `Arc<dyn
    /// ClassResolver>` lives off-salsa because trait objects don't
    /// participate in `salsa::Update`.
    resolver_state: Arc<parking_lot::RwLock<ResolverState>>,
    /// Lazily-created singleton [`InferredReturnTypes`] input. Stored
    /// alongside `resolver_state` (rather than as a bare salsa input
    /// handle on `Self`) because the handle is `Copy` and we need
    /// `MirDb: Clone`-friendly storage.
    inferred_return_types_input: Arc<parking_lot::RwLock<Option<InferredReturnTypes>>>,
    /// Lazily-created [`WorkspaceRevision`] singleton input; bumped on
    /// file add/remove so workspace-enumeration tracked queries
    /// (`workspace_classes`, `workspace_functions`) invalidate.
    workspace_revision_input: Arc<parking_lot::RwLock<Option<WorkspaceRevision>>>,
}

/// Resolver-related state held outside salsa storage. Wrapped in a
/// `parking_lot::RwLock` so `MirDb::clone()` (cheap for parallel readers)
/// shares one slot rather than copying.
#[derive(Default)]
struct ResolverState {
    /// Lazily created on first `set_resolver`. Once created, the handle
    /// is stable across clones and subsequent resolver swaps.
    config: Option<ResolverConfig>,
    /// Currently active resolver. `None` for sessions configured without
    /// PSR-4 / classmap support.
    resolver: Option<Arc<dyn crate::ClassResolver>>,
}

#[salsa::db]
impl salsa::Database for MirDb {}

#[salsa::db]
impl MirDatabase for MirDb {
    fn php_version_str(&self) -> Arc<str> {
        Arc::from("8.2")
    }

    fn lookup_class_node(&self, _fqcn: &str) -> Option<ClassNode> {
        None
    }

    fn lookup_function_node(&self, _fqn: &str) -> Option<FunctionNode> {
        None
    }

    fn lookup_method_node(&self, _fqcn: &str, _method_name_lower: &str) -> Option<MethodNode> {
        None
    }

    fn lookup_property_node(&self, _fqcn: &str, _prop_name: &str) -> Option<PropertyNode> {
        None
    }

    fn lookup_class_constant_node(
        &self,
        _fqcn: &str,
        _const_name: &str,
    ) -> Option<ClassConstantNode> {
        None
    }

    fn lookup_global_constant_node(&self, _fqn: &str) -> Option<GlobalConstantNode> {
        None
    }

    fn class_own_methods(&self, _fqcn: &str) -> Vec<MethodNode> {
        vec![]
    }

    fn class_own_properties(&self, _fqcn: &str) -> Vec<PropertyNode> {
        vec![]
    }

    fn class_own_constants(&self, _fqcn: &str) -> Vec<ClassConstantNode> {
        vec![]
    }

    fn active_class_node_fqcns(&self) -> Vec<Arc<str>> {
        vec![]
    }

    fn active_function_node_fqns(&self) -> Vec<Arc<str>> {
        vec![]
    }

    fn file_namespace(&self, file: &str) -> Option<Arc<str>> {
        self.file_namespaces.get(file).cloned()
    }

    fn file_imports(&self, file: &str) -> HashMap<String, String> {
        self.file_imports.get(file).cloned().unwrap_or_default()
    }

    fn global_var_type(&self, name: &str) -> Option<Union> {
        self.global_vars.get(name).cloned()
    }

    fn file_import_snapshots(&self) -> Vec<(Arc<str>, HashMap<String, String>)> {
        self.file_imports
            .iter()
            .map(|(file, imports)| (file.clone(), imports.clone()))
            .collect()
    }

    fn symbol_defining_file(&self, symbol: &str) -> Option<Arc<str>> {
        self.symbol_to_file.get(symbol).cloned()
    }

    fn symbols_defined_in_file(&self, file: &str) -> Vec<Arc<str>> {
        self.file_to_defined_symbols
            .lock()
            .get(file)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn file_defined_symbols(&self, file: &str) -> HashSet<Arc<str>> {
        self.file_to_defined_symbols
            .lock()
            .get(file)
            .cloned()
            .unwrap_or_default()
    }

    fn symbol_referencers_of(&self, symbol_key: &str) -> Vec<Arc<str>> {
        self.symbol_referencers
            .lock()
            .get(symbol_key)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn record_reference_location(&self, loc: RefLoc) {
        self.pending_ref_locs.0.lock().push(loc);
    }

    fn take_pending_ref_locs(&self) -> Vec<RefLoc> {
        std::mem::take(&mut *self.pending_ref_locs.0.lock())
    }

    fn replay_reference_locations(&self, file: Arc<str>, locs: &[(String, u32, u16, u16)]) {
        for (symbol, line, col_start, col_end) in locs {
            self.record_reference_location(RefLoc {
                symbol_key: Arc::from(symbol.as_str()),
                file: file.clone(),
                line: *line,
                col_start: *col_start,
                col_end: *col_end,
            });
        }
    }

    fn extract_file_reference_locations(&self, file: &str) -> Vec<(Arc<str>, u32, u16, u16)> {
        let refs = self.reference_locations.lock();
        let mut out = Vec::new();
        for (symbol, locs) in refs.iter() {
            for (loc_file, line, col_start, col_end) in locs {
                if loc_file.as_ref() == file {
                    out.push((symbol.clone(), *line, *col_start, *col_end));
                }
            }
        }
        out
    }

    fn reference_locations(&self, symbol: &str) -> Vec<(Arc<str>, u32, u16, u16)> {
        let refs = self.reference_locations.lock();
        refs.get(symbol).cloned().unwrap_or_default()
    }

    fn has_reference(&self, symbol: &str) -> bool {
        let refs = self.reference_locations.lock();
        refs.get(symbol).is_some_and(|locs| !locs.is_empty())
    }

    fn clear_file_references(&self, file: &str) {
        // Drain the forward index first to learn which symbols this file referenced,
        // then remove only those entries — O(degree) instead of O(S×R).
        let symbol_keys = {
            let mut file_refs = self.file_references.lock();
            file_refs.remove(file).unwrap_or_default()
        };
        let mut refs = self.reference_locations.lock();
        let mut sym_refs = self.symbol_referencers.lock();
        for key in &symbol_keys {
            if let Some(locs) = refs.get_mut(key) {
                locs.retain(|(loc_file, _, _, _)| loc_file.as_ref() != file);
            }
            let empty = if let Some(referencers) = sym_refs.get_mut(key) {
                referencers.remove(file);
                referencers.is_empty()
            } else {
                false
            };
            if empty {
                sym_refs.remove(key);
            }
        }
    }

    fn all_reference_location_pairs(&self) -> Vec<(Arc<str>, Arc<str>)> {
        let refs = self.reference_locations.lock();
        let mut pairs = Vec::new();
        for (symbol, locs) in refs.iter() {
            for (file, _, _, _) in locs {
                pairs.push((file.clone(), symbol.clone()));
            }
        }
        pairs
    }

    fn file_referenced_symbols(&self, file: &str) -> Vec<Arc<str>> {
        let file_refs = self.file_references.lock();
        file_refs
            .get(file)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn lookup_source_file(&self, path: &str) -> Option<SourceFile> {
        self.source_files.get(path).copied()
    }

    fn resolver_config(&self) -> Option<ResolverConfig> {
        self.resolver_state.read().config
    }

    fn current_resolver(&self) -> Option<Arc<dyn crate::ClassResolver>> {
        self.resolver_state.read().resolver.clone()
    }

    fn inferred_return_types(&self) -> Option<InferredReturnTypes> {
        *self.inferred_return_types_input.read()
    }

    fn workspace_revision(&self) -> Option<WorkspaceRevision> {
        *self.workspace_revision_input.read()
    }

    fn all_source_files(&self) -> Vec<SourceFile> {
        self.source_files.values().copied().collect()
    }
}

/// Field bag for [`MirDb::upsert_class_node`].  Construct with `..Default::default()`
/// to fill in the fields that don't apply to your kind (e.g. interfaces leave
/// `parent`, `traits`, `mixins`, `is_abstract`, etc. at their defaults).
///
/// Per-kind constructors (`for_class` / `for_interface` / `for_trait` /
/// `for_enum`) seed the kind discriminators so the caller only has to populate
/// kind-specific fields.
#[derive(Debug, Clone, Default)]
pub struct ClassNodeFields {
    pub fqcn: Arc<str>,
    pub is_interface: bool,
    pub is_trait: bool,
    pub is_enum: bool,
    pub is_abstract: bool,
    pub parent: Option<Arc<str>>,
    pub interfaces: Arc<[Arc<str>]>,
    pub traits: Arc<[Arc<str>]>,
    pub trait_use_locations: Arc<[(Arc<str>, Location)]>,
    pub extends: Arc<[Arc<str>]>,
    pub template_params: Arc<[TemplateParam]>,
    pub require_extends: Arc<[Arc<str>]>,
    pub require_implements: Arc<[Arc<str>]>,
    pub is_backed_enum: bool,
    pub mixins: Arc<[Arc<str>]>,
    pub deprecated: Option<Arc<str>>,
    pub enum_scalar_type: Option<Union>,
    pub is_final: bool,
    pub is_readonly: bool,
    pub location: Option<Location>,
    pub extends_type_args: Arc<[Union]>,
    pub implements_type_args: ImplementsTypeArgs,
}

impl ClassNodeFields {
    pub fn for_class(fqcn: Arc<str>) -> Self {
        Self {
            fqcn,
            ..Self::default()
        }
    }

    pub fn for_interface(fqcn: Arc<str>) -> Self {
        Self {
            fqcn,
            is_interface: true,
            ..Self::default()
        }
    }

    pub fn for_trait(fqcn: Arc<str>) -> Self {
        Self {
            fqcn,
            is_trait: true,
            ..Self::default()
        }
    }

    pub fn for_enum(fqcn: Arc<str>) -> Self {
        Self {
            fqcn,
            is_enum: true,
            ..Self::default()
        }
    }
}

impl MirDb {
    /// Commit a batch of reference locations into the shared maps in one lock
    /// acquisition per map.  Must be called serially after all parallel workers
    /// have dropped their db clones and returned their pending buffers.
    pub fn commit_reference_locations_batch(&self, locs: Vec<RefLoc>) {
        if locs.is_empty() {
            return;
        }
        let mut refs = self.reference_locations.lock();
        let mut file_refs = self.file_references.lock();
        let mut sym_refs = self.symbol_referencers.lock();
        for loc in locs {
            file_refs
                .entry(loc.file.clone())
                .or_default()
                .insert(loc.symbol_key.clone());
            sym_refs
                .entry(loc.symbol_key.clone())
                .or_default()
                .insert(loc.file.clone());
            let entry = refs.entry(loc.symbol_key).or_default();
            let tuple = (loc.file, loc.line, loc.col_start, loc.col_end);
            if !entry.iter().any(|e| e == &tuple) {
                entry.push(tuple);
            }
        }
    }

    /// Drain this db's pending buffer and commit it directly to the shared maps.
    ///
    /// Use on serial paths (e.g. `re_analyze_file`) where the db is not a
    /// worker clone: pending locations accumulate in the shared db itself and
    /// must be flushed before callers read the reference maps.
    pub fn commit_pending_to_maps(&self) {
        let locs = std::mem::take(&mut *self.pending_ref_locs.0.lock());
        self.commit_reference_locations_batch(locs);
    }

    /// Install or replace the active class resolver.
    ///
    /// First call lazily creates the singleton [`ResolverConfig`] salsa
    /// input (revision = 0); subsequent calls bump the revision so
    /// downstream tracked queries (notably
    /// [`crate::db::resolve_fqcn_to_path`]) are invalidated.
    ///
    /// `None` clears the resolver. The `ResolverConfig` input is *not*
    /// removed — it remains as a versioned anchor, with revision bumped to
    /// signal the change.
    pub fn set_resolver(&mut self, resolver: Option<Arc<dyn crate::ClassResolver>>) {
        use salsa::Setter as _;
        // The lock and salsa storage are independent; we briefly read /
        // briefly write the lock, but never hold it across a salsa setter
        // call (which needs `&mut self`).
        let existing = self.resolver_state.read().config;
        match existing {
            Some(c) => {
                let current = c.revision(self);
                c.set_revision(self).to(current.wrapping_add(1));
            }
            None => {
                let c = ResolverConfig::new(self, 0);
                self.resolver_state.write().config = Some(c);
            }
        }
        self.resolver_state.write().resolver = resolver;
    }

    /// Create a new or update an existing Salsa SourceFile input for `path`.
    /// Returns the stable handle that callers should retain for tracked queries.
    pub fn upsert_source_file(&mut self, path: Arc<str>, text: Arc<str>) -> SourceFile {
        use salsa::Setter as _;
        if let Some(&sf) = self.source_files.get(&path) {
            if sf.text(self) != text {
                sf.set_text(self).to(text);
            }
            return sf;
        }
        let sf = SourceFile::new(self, path.clone(), text);
        Arc::make_mut(&mut self.source_files).insert(path, sf);
        self.bump_workspace_revision();
        sf
    }

    /// Remove the Salsa SourceFile handle for `path` from the registry.
    pub fn remove_source_file(&mut self, path: &str) {
        if Arc::make_mut(&mut self.source_files).remove(path).is_some() {
            self.bump_workspace_revision();
        }
    }

    /// Bump the workspace revision so tracked `workspace_*` queries
    /// reading it invalidate. Lazily creates the singleton input on
    /// first call.
    fn bump_workspace_revision(&mut self) {
        use salsa::Setter as _;
        let existing = *self.workspace_revision_input.read();
        match existing {
            Some(rev) => {
                let cur = rev.revision(self);
                rev.set_revision(self).to(cur.wrapping_add(1));
            }
            None => {
                let rev = crate::db::WorkspaceRevision::new(self, 0);
                *self.workspace_revision_input.write() = Some(rev);
            }
        }
    }

    /// Number of source files currently registered.
    pub fn source_file_count(&self) -> usize {
        self.source_files.len()
    }

    /// All registered source file paths.
    pub fn source_file_paths(&self) -> Vec<Arc<str>> {
        self.source_files.keys().cloned().collect()
    }

    /// Insert `symbol` into both `symbol_to_file` and the `file_to_defined_symbols`
    /// forward index. All definition-registration sites must use this helper.
    fn register_symbol(&mut self, symbol: Arc<str>, file: Arc<str>) {
        Arc::make_mut(&mut self.symbol_to_file).insert(symbol.clone(), file.clone());
        self.file_to_defined_symbols
            .lock()
            .entry(file)
            .or_default()
            .insert(symbol);
    }

    pub fn remove_file_definitions(&mut self, file: &str) {
        // O(1) forward-index lookup instead of O(total_symbols) scan.
        let symbol_set: HashSet<Arc<str>> = self
            .file_to_defined_symbols
            .lock()
            .remove(file)
            .unwrap_or_default();
        for symbol in &symbol_set {
            self.deactivate_class_node(symbol);
            self.deactivate_function_node(symbol);
            self.deactivate_class_methods(symbol);
            self.deactivate_class_properties(symbol);
            self.deactivate_class_constants(symbol);
            self.deactivate_global_constant_node(symbol);
        }
        {
            let s2f = Arc::make_mut(&mut self.symbol_to_file);
            for sym in &symbol_set {
                s2f.remove(sym.as_ref());
            }
        }
        Arc::make_mut(&mut self.file_namespaces).retain(|path, _| path.as_ref() != file);
        Arc::make_mut(&mut self.file_imports).retain(|path, _| path.as_ref() != file);
        Arc::make_mut(&mut self.global_vars).retain(|name, _| !symbol_set.contains(name));
        self.clear_file_references(file);
    }

    pub fn type_count(&self) -> usize {
        self.class_nodes
            .values()
            .filter(|node| node.active(self))
            .count()
    }

    pub fn function_count(&self) -> usize {
        self.function_nodes
            .values()
            .filter(|node| node.active(self))
            .count()
    }

    pub fn constant_count(&self) -> usize {
        self.global_constant_nodes
            .values()
            .filter(|node| node.active(self))
            .count()
    }

    /// Walk one collected [`StubSlice`] and upsert the corresponding db nodes.
    ///
    /// This is the canonical post-Pass-1 ingestion path: each file's slice is
    /// fed in directly, so batch analysis does not need any intermediate
    /// mutable codebase store between Pass 1 and Pass 2.
    pub fn ingest_stub_slice(&mut self, _slice: &StubSlice) {
        // Phase 5: push-path indexes are no longer read. The body is retained
        // below behind `if false {}` so the symbol type-checks; future cleanup
        // removes it together with the FxHashMap registries it populates.
        // The function itself stays callable so existing call sites
        // (`SharedDb::collect_and_ingest_file`, project / stub bootstrap) keep
        // compiling without further edits.
        return;
        #[allow(unreachable_code)]
        let slice = _slice;
        #[allow(unreachable_code)]
        if false {
            use std::collections::HashSet;

            // Deduplicate param lists to save memory. Skip the clone+dedup when the
            // slice was already deduped in the parallel pass (is_deduped == true) or
            // for small slices where the overhead isn't worth it.
            let owned_slice;
            let slice: &StubSlice = if !slice.is_deduped {
                let total_methods: usize = slice
                    .classes
                    .iter()
                    .map(|c| c.own_methods.len())
                    .sum::<usize>()
                    + slice
                        .interfaces
                        .iter()
                        .map(|i| i.own_methods.len())
                        .sum::<usize>()
                    + slice
                        .traits
                        .iter()
                        .map(|t| t.own_methods.len())
                        .sum::<usize>()
                    + slice
                        .enums
                        .iter()
                        .map(|e| e.own_methods.len())
                        .sum::<usize>()
                    + slice.functions.len();

                if total_methods >= 8 {
                    let mut s = slice.clone();
                    mir_codebase::storage::deduplicate_params_in_slice(&mut s);
                    owned_slice = s;
                    &owned_slice
                } else {
                    slice
                }
            } else {
                slice
            };

            if let Some(file) = &slice.file {
                if let Some(namespace) = &slice.namespace {
                    Arc::make_mut(&mut self.file_namespaces)
                        .insert(file.clone(), namespace.clone());
                }
                if !slice.imports.is_empty() {
                    Arc::make_mut(&mut self.file_imports)
                        .insert(file.clone(), slice.imports.clone());
                }
                for (name, _) in &slice.global_vars {
                    let global_name = name.strip_prefix('$').unwrap_or(name.as_ref());
                    self.register_symbol(Arc::from(global_name), file.clone());
                }
            }
            for (name, ty) in &slice.global_vars {
                let global_name = name.strip_prefix('$').unwrap_or(name.as_ref());
                Arc::make_mut(&mut self.global_vars).insert(Arc::from(global_name), ty.clone());
            }

            let slice_file = slice.file.clone();
            for cls in &slice.classes {
                if let Some(file) = &slice_file {
                    self.register_symbol(cls.fqcn.clone(), file.clone());
                }
                self.upsert_class_node(ClassNodeFields {
                    is_abstract: cls.is_abstract,
                    parent: cls.parent.clone(),
                    interfaces: Arc::from(cls.interfaces.as_ref()),
                    traits: Arc::from(cls.traits.as_ref()),
                    trait_use_locations: Arc::from(cls.trait_use_locations.as_ref()),
                    template_params: Arc::from(cls.template_params.as_ref()),
                    mixins: Arc::from(cls.mixins.as_ref()),
                    deprecated: cls.deprecated.clone(),
                    is_final: cls.is_final,
                    is_readonly: cls.is_readonly,
                    location: cls.location.clone(),
                    extends_type_args: Arc::from(cls.extends_type_args.as_ref()),
                    implements_type_args: Arc::from(
                        cls.implements_type_args
                            .iter()
                            .map(|(iface, args)| (iface.clone(), Arc::from(args.as_ref())))
                            .collect::<Vec<_>>(),
                    ),
                    ..ClassNodeFields::for_class(cls.fqcn.clone())
                });
                if self.method_nodes.contains_key(cls.fqcn.as_ref()) {
                    let method_keep: HashSet<&str> =
                        cls.own_methods.keys().map(|m| m.as_ref()).collect();
                    self.prune_class_methods(&cls.fqcn, &method_keep);
                }
                for method in cls.own_methods.values() {
                    // Avoid cloning complex return type Unions during vendor ingestion
                    // by wrapping in Arc upfront. This is a per-method operation during
                    // vendor type collection (rare after initialization), so the Arc
                    // allocation is amortized.
                    self.upsert_method_node(method.as_ref());
                }
                if self.property_nodes.contains_key(cls.fqcn.as_ref()) {
                    let prop_keep: HashSet<&str> =
                        cls.own_properties.keys().map(|p| p.as_ref()).collect();
                    self.prune_class_properties(&cls.fqcn, &prop_keep);
                }
                for prop in cls.own_properties.values() {
                    self.upsert_property_node(&cls.fqcn, prop);
                }
                if self.class_constant_nodes.contains_key(cls.fqcn.as_ref()) {
                    let const_keep: HashSet<&str> =
                        cls.own_constants.keys().map(|c| c.as_ref()).collect();
                    self.prune_class_constants(&cls.fqcn, &const_keep);
                }
                for constant in cls.own_constants.values() {
                    self.upsert_class_constant_node(&cls.fqcn, constant);
                }
            }

            for iface in &slice.interfaces {
                if let Some(file) = &slice_file {
                    self.register_symbol(iface.fqcn.clone(), file.clone());
                }
                self.upsert_class_node(ClassNodeFields {
                    extends: Arc::from(iface.extends.as_ref()),
                    template_params: Arc::from(iface.template_params.as_ref()),
                    location: iface.location.clone(),
                    ..ClassNodeFields::for_interface(iface.fqcn.clone())
                });
                if self.method_nodes.contains_key(iface.fqcn.as_ref()) {
                    let method_keep: HashSet<&str> =
                        iface.own_methods.keys().map(|m| m.as_ref()).collect();
                    self.prune_class_methods(&iface.fqcn, &method_keep);
                }
                for method in iface.own_methods.values() {
                    self.upsert_method_node(method.as_ref());
                }
                if self.class_constant_nodes.contains_key(iface.fqcn.as_ref()) {
                    let const_keep: HashSet<&str> =
                        iface.own_constants.keys().map(|c| c.as_ref()).collect();
                    self.prune_class_constants(&iface.fqcn, &const_keep);
                }
                for constant in iface.own_constants.values() {
                    self.upsert_class_constant_node(&iface.fqcn, constant);
                }
            }

            for tr in &slice.traits {
                if let Some(file) = &slice_file {
                    self.register_symbol(tr.fqcn.clone(), file.clone());
                }
                self.upsert_class_node(ClassNodeFields {
                    traits: Arc::from(tr.traits.as_ref()),
                    template_params: Arc::from(tr.template_params.as_ref()),
                    require_extends: Arc::from(tr.require_extends.as_ref()),
                    require_implements: Arc::from(tr.require_implements.as_ref()),
                    location: tr.location.clone(),
                    ..ClassNodeFields::for_trait(tr.fqcn.clone())
                });
                if self.method_nodes.contains_key(tr.fqcn.as_ref()) {
                    let method_keep: HashSet<&str> =
                        tr.own_methods.keys().map(|m| m.as_ref()).collect();
                    self.prune_class_methods(&tr.fqcn, &method_keep);
                }
                for method in tr.own_methods.values() {
                    self.upsert_method_node(method.as_ref());
                }
                if self.property_nodes.contains_key(tr.fqcn.as_ref()) {
                    let prop_keep: HashSet<&str> =
                        tr.own_properties.keys().map(|p| p.as_ref()).collect();
                    self.prune_class_properties(&tr.fqcn, &prop_keep);
                }
                for prop in tr.own_properties.values() {
                    self.upsert_property_node(&tr.fqcn, prop);
                }
                if self.class_constant_nodes.contains_key(tr.fqcn.as_ref()) {
                    let const_keep: HashSet<&str> =
                        tr.own_constants.keys().map(|c| c.as_ref()).collect();
                    self.prune_class_constants(&tr.fqcn, &const_keep);
                }
                for constant in tr.own_constants.values() {
                    self.upsert_class_constant_node(&tr.fqcn, constant);
                }
            }

            for en in &slice.enums {
                if let Some(file) = &slice_file {
                    self.register_symbol(en.fqcn.clone(), file.clone());
                }
                self.upsert_class_node(ClassNodeFields {
                    interfaces: Arc::from(en.interfaces.as_ref()),
                    is_backed_enum: en.scalar_type.is_some(),
                    enum_scalar_type: en.scalar_type.clone(),
                    location: en.location.clone(),
                    ..ClassNodeFields::for_enum(en.fqcn.clone())
                });
                if self.method_nodes.contains_key(en.fqcn.as_ref()) {
                    let mut method_keep: HashSet<&str> =
                        en.own_methods.keys().map(|m| m.as_ref()).collect();
                    method_keep.insert("cases");
                    if en.scalar_type.is_some() {
                        method_keep.insert("from");
                        method_keep.insert("tryfrom");
                    }
                    self.prune_class_methods(&en.fqcn, &method_keep);
                }
                for method in en.own_methods.values() {
                    self.upsert_method_node(method.as_ref());
                }
                let synth_method = |name: &str| mir_codebase::storage::MethodStorage {
                    fqcn: en.fqcn.clone(),
                    name: Arc::from(name),
                    params: Arc::from([].as_ref()),
                    return_type: Some(Arc::new(Union::mixed())),
                    inferred_return_type: None,
                    visibility: Visibility::Public,
                    is_static: true,
                    is_abstract: false,
                    is_constructor: false,
                    template_params: vec![],
                    assertions: vec![],
                    throws: vec![],
                    is_final: false,
                    is_virtual: false,
                    is_internal: false,
                    is_pure: false,
                    deprecated: None,
                    location: None,
                    docstring: None,
                };
                let already = |name: &str| {
                    en.own_methods
                        .keys()
                        .any(|k| k.as_ref().eq_ignore_ascii_case(name))
                };
                if !already("cases") {
                    self.upsert_method_node(&synth_method("cases"));
                }
                if en.scalar_type.is_some() {
                    if !already("from") {
                        self.upsert_method_node(&synth_method("from"));
                    }
                    if !already("tryFrom") {
                        self.upsert_method_node(&synth_method("tryFrom"));
                    }
                }
                if self.class_constant_nodes.contains_key(en.fqcn.as_ref()) {
                    let mut const_keep: HashSet<&str> =
                        en.own_constants.keys().map(|c| c.as_ref()).collect();
                    for case in en.cases.values() {
                        const_keep.insert(case.name.as_ref());
                    }
                    self.prune_class_constants(&en.fqcn, &const_keep);
                }
                for constant in en.own_constants.values() {
                    self.upsert_class_constant_node(&en.fqcn, constant);
                }
                for case in en.cases.values() {
                    let case_const = ConstantStorage {
                        name: case.name.clone(),
                        ty: mir_types::Union::mixed(),
                        visibility: None,
                        is_final: false,
                        location: case.location.clone(),
                    };
                    self.upsert_class_constant_node(&en.fqcn, &case_const);
                }
            }

            for func in &slice.functions {
                if let Some(file) = &slice_file {
                    self.register_symbol(func.fqn.clone(), file.clone());
                }
                self.upsert_function_node(func);
            }
            for (fqn, ty) in &slice.constants {
                self.upsert_global_constant_node(fqn.clone(), ty.clone());
            }
        }
    }

    /// Bulk-ingest many stub slices in one call.
    ///
    /// Why this exists: when an external `Arc<MirDb>` snapshot is alive (e.g.
    /// an LSP server holds one for query serving), each `Arc::make_mut` inside
    /// [`Self::ingest_stub_slice`] forces a copy-on-write clone of the
    /// underlying `HashMap`. Calling `ingest_stub_slice` N times in sequence
    /// with the snapshot alive between calls pays one clone *per call* —
    /// asymptotically O(N × map_size), which becomes pathological at vendor
    /// scale (~2k+ slices).
    ///
    /// Inside this bulk path the snapshot doesn't get refreshed between
    /// slices, so the first slice's clone establishes a fresh inner `Arc` with
    /// `strong_count == 1` and every subsequent insert in the batch is O(1).
    /// Net cost: O(N + map_size) instead of O(N × map_size).
    ///
    /// Use this whenever you're about to ingest more than one slice in a row,
    /// such as in:
    /// - LSP warm-up over a `composer.lock` worth of vendor files
    /// - Project-wide reindex
    /// - Cache hydration on session restart
    pub fn ingest_stub_slices<'a, I>(&mut self, slices: I)
    where
        I: IntoIterator<Item = &'a StubSlice>,
    {
        for slice in slices {
            self.ingest_stub_slice(slice);
        }
    }

    /// Create or update the `ClassNode` for `fqcn`.
    ///
    /// If a handle already exists, its fields are updated in-place so Salsa
    /// can track the change.  A new handle is created only on first registration.
    #[allow(clippy::too_many_arguments)]
    pub fn upsert_class_node(&mut self, fields: ClassNodeFields) -> ClassNode {
        use salsa::Setter as _;
        let ClassNodeFields {
            fqcn,
            is_interface,
            is_trait,
            is_enum,
            is_abstract,
            parent,
            interfaces,
            traits,
            trait_use_locations,
            extends,
            template_params,
            require_extends,
            require_implements,
            is_backed_enum,
            mixins,
            deprecated,
            enum_scalar_type,
            is_final,
            is_readonly,
            location,
            extends_type_args,
            implements_type_args,
        } = fields;
        if let Some(&node) = self.class_nodes.get(&fqcn) {
            // Fast-skip: an already-active node whose Salsa-tracked fields
            // match the upsert input.  Bulk re-ingest paths
            // (`ingest_stub_slice` / `lazy_load_*`) call this for every class
            // on every iteration; without the skip each call fires 13
            // setters, each acquiring the Salsa write lock.  Schema doesn't
            // mutate after Pass 1 (Pass 2 only writes `inferred_return_type`),
            // so an active node with matching fields is by construction up
            // to date.
            //
            // Mutation paths (LSP re-analyze) call `deactivate_class_node`
            // first; that flips `active=false`, defeating this guard so the
            // setters run as before.
            if node.active(self)
                && node.is_interface(self) == is_interface
                && node.is_trait(self) == is_trait
                && node.is_enum(self) == is_enum
                && node.is_abstract(self) == is_abstract
                && node.is_backed_enum(self) == is_backed_enum
                && node.parent(self) == parent
                && *node.interfaces(self) == *interfaces
                && *node.traits(self) == *traits
                && *node.trait_use_locations(self) == *trait_use_locations
                && *node.extends(self) == *extends
                && *node.template_params(self) == *template_params
                && *node.require_extends(self) == *require_extends
                && *node.require_implements(self) == *require_implements
                && *node.mixins(self) == *mixins
                && node.deprecated(self) == deprecated
                && node.enum_scalar_type(self) == enum_scalar_type
                && node.is_final(self) == is_final
                && node.is_readonly(self) == is_readonly
                && node.location(self) == location
                && *node.extends_type_args(self) == *extends_type_args
                && *node.implements_type_args(self) == *implements_type_args
            {
                return node;
            }
            node.set_active(self).to(true);
            node.set_is_interface(self).to(is_interface);
            node.set_is_trait(self).to(is_trait);
            node.set_is_enum(self).to(is_enum);
            node.set_is_abstract(self).to(is_abstract);
            node.set_parent(self).to(parent);
            node.set_interfaces(self).to(interfaces);
            node.set_traits(self).to(traits);
            node.set_trait_use_locations(self).to(trait_use_locations);
            node.set_extends(self).to(extends);
            node.set_template_params(self).to(template_params);
            node.set_require_extends(self).to(require_extends);
            node.set_require_implements(self).to(require_implements);
            node.set_is_backed_enum(self).to(is_backed_enum);
            node.set_mixins(self).to(mixins);
            node.set_deprecated(self).to(deprecated);
            node.set_enum_scalar_type(self).to(enum_scalar_type);
            node.set_is_final(self).to(is_final);
            node.set_is_readonly(self).to(is_readonly);
            node.set_location(self).to(location);
            node.set_extends_type_args(self).to(extends_type_args);
            node.set_implements_type_args(self).to(implements_type_args);
            node
        } else {
            let node = ClassNode::new(
                self,
                fqcn.clone(),
                true,
                is_interface,
                is_trait,
                is_enum,
                is_abstract,
                parent,
                interfaces,
                traits,
                trait_use_locations,
                extends,
                template_params,
                require_extends,
                require_implements,
                is_backed_enum,
                mixins,
                deprecated,
                enum_scalar_type,
                is_final,
                is_readonly,
                location,
                extends_type_args,
                implements_type_args,
            );
            Arc::make_mut(&mut self.class_node_keys_lower)
                .insert(fqcn.to_ascii_lowercase(), fqcn.clone());
            Arc::make_mut(&mut self.class_nodes).insert(fqcn, node);
            node
        }
    }

    /// Mark the `ClassNode` for `fqcn` as inactive.
    ///
    /// Dependent `class_ancestors` queries will observe the change and re-run,
    /// returning an empty list.
    pub fn deactivate_class_node(&mut self, fqcn: &str) {
        use salsa::Setter as _;
        if let Some(&node) = self.class_nodes.get(fqcn) {
            node.set_active(self).to(false);
        }
    }

    /// Create or update the `FunctionNode` for the given `FunctionStorage`.
    pub fn upsert_function_node(&mut self, storage: &FunctionStorage) -> FunctionNode {
        use salsa::Setter as _;
        let fqn = &storage.fqn;
        if let Some(&node) = self.function_nodes.get(fqn.as_ref()) {
            // Fast-skip identical re-ingest — see `upsert_class_node` for rationale.
            // `inferred_return_type` is intentionally NOT compared / written:
            // it is owned by the priming sweep's serial commit phase
            // (`commit_inferred_return_types`) and Pass-1 re-ingest must not
            // clobber a previously-inferred value.
            if node.active(self)
                && node.short_name(self) == storage.short_name
                && node.is_pure(self) == storage.is_pure
                && node.deprecated(self) == storage.deprecated
                && node.return_type(self).as_deref() == storage.return_type.as_deref()
                && node.location(self) == storage.location
                && *node.params(self) == *storage.params.as_ref()
                && *node.template_params(self) == *storage.template_params
                && *node.assertions(self) == *storage.assertions
                && *node.throws(self) == *storage.throws
            {
                return node;
            }
            node.set_active(self).to(true);
            node.set_short_name(self).to(storage.short_name.clone());
            node.set_params(self).to(storage.params.clone());
            node.set_return_type(self).to(storage.return_type.clone());
            node.set_template_params(self)
                .to(Arc::from(storage.template_params.as_slice()));
            node.set_assertions(self)
                .to(Arc::from(storage.assertions.as_slice()));
            node.set_throws(self)
                .to(Arc::from(storage.throws.as_slice()));
            node.set_deprecated(self).to(storage.deprecated.clone());
            node.set_docstring(self).to(storage.docstring.clone());
            node.set_is_pure(self).to(storage.is_pure);
            node.set_location(self).to(storage.location.clone());
            node
        } else {
            let node = FunctionNode::new(
                self,
                fqn.clone(),
                storage.short_name.clone(),
                true,
                storage.params.clone(),
                storage.return_type.clone(),
                storage
                    .inferred_return_type
                    .as_ref()
                    .map(|t| Arc::new(t.clone())),
                Arc::from(storage.template_params.as_slice()),
                Arc::from(storage.assertions.as_slice()),
                Arc::from(storage.throws.as_slice()),
                storage.deprecated.clone(),
                storage.docstring.clone(),
                storage.is_pure,
                storage.location.clone(),
            );
            Arc::make_mut(&mut self.function_node_keys_lower)
                .insert(fqn.to_ascii_lowercase(), fqn.clone());
            Arc::make_mut(&mut self.function_nodes).insert(fqn.clone(), node);
            node
        }
    }

    /// Commit a parallel-sweep-collected [`InferredReturnTypes`] buffer
    /// into the Salsa db.  **Must be called serially**, after all rayon
    /// workers from the priming sweep have dropped their db clones, so
    /// that `Storage::cancel_others` sees strong-count==1 inside the
    /// setter.  Calling this from inside a `for_each_with` / `map_with`
    /// closure will deadlock.
    ///
    /// Skips writes whose value already matches the current Salsa-tracked
    /// value (preserves PR21's fast-skip semantics).  Skips inactive
    /// nodes — there's no point committing an inferred return for a node
    /// that has been deactivated by a re-analyze.
    /// Commit inferred return types collected during the priming sweep.
    /// Takes ownership of the function and method inferred type vectors.
    pub fn commit_inferred_return_types(
        &mut self,
        functions: Vec<(Arc<str>, mir_types::Union)>,
        methods: Vec<(Arc<str>, Arc<str>, mir_types::Union)>,
    ) {
        use salsa::Setter as _;

        // Aggregate maps for the new salsa-input path. Built in lockstep
        // with the legacy node-setter writes so the two paths stay
        // consistent during the Phase-4 migration. Pass-2 readers will
        // switch to the input one site at a time; once all are migrated,
        // the legacy node writes go away (Phase 5).
        let merged_functions = self.inferred_function_map_clone();
        let merged_methods = self.inferred_method_map_clone();
        let mut new_functions = (*merged_functions).clone();
        let mut new_methods = (*merged_methods).clone();

        for (fqn, inferred) in functions {
            let arc_inferred = Arc::new(inferred);
            new_functions.insert(fqn.clone(), arc_inferred.clone());
            if let Some(&node) = self.function_nodes.get(fqn.as_ref()) {
                if !node.active(self) {
                    continue;
                }
                let new = Some(arc_inferred);
                if node.inferred_return_type(self) == new {
                    continue;
                }
                node.set_inferred_return_type(self).to(new);
            }
        }
        for (fqcn, name, inferred) in methods {
            let name_lower: Arc<str> = if name.chars().all(|c| !c.is_uppercase()) {
                name.clone()
            } else {
                Arc::from(name.to_lowercase().as_str())
            };
            let arc_inferred = Arc::new(inferred);
            new_methods.insert((fqcn.clone(), name_lower.clone()), arc_inferred.clone());
            let node = self
                .method_nodes
                .get(fqcn.as_ref())
                .and_then(|m| m.get(&name_lower))
                .copied();
            if let Some(node) = node {
                if !node.active(self) {
                    continue;
                }
                let new = Some(arc_inferred);
                if node.inferred_return_type(self) == new {
                    continue;
                }
                node.set_inferred_return_type(self).to(new);
            }
        }

        self.set_inferred_return_types_input(Arc::new(new_functions), Arc::new(new_methods));
    }

    /// Snapshot the function inferred map from the singleton input, or an
    /// empty map if the input hasn't been created yet.
    fn inferred_function_map_clone(&self) -> Arc<crate::db::FunctionInferredMap> {
        match *self.inferred_return_types_input.read() {
            Some(input) => input.functions(self),
            None => Arc::new(rustc_hash::FxHashMap::default()),
        }
    }

    /// Snapshot the method inferred map from the singleton input, or an
    /// empty map if the input hasn't been created yet.
    fn inferred_method_map_clone(&self) -> Arc<crate::db::MethodInferredMap> {
        match *self.inferred_return_types_input.read() {
            Some(input) => input.methods(self),
            None => Arc::new(rustc_hash::FxHashMap::default()),
        }
    }

    /// Set / create the singleton [`crate::db::InferredReturnTypes`] input.
    /// Lazily creates the input on first call (handle stored on
    /// `inferred_return_types_input`); subsequent calls update the
    /// existing handle via setters.
    fn set_inferred_return_types_input(
        &mut self,
        functions: Arc<crate::db::FunctionInferredMap>,
        methods: Arc<crate::db::MethodInferredMap>,
    ) {
        use salsa::Setter as _;
        let existing = *self.inferred_return_types_input.read();
        match existing {
            Some(handle) => {
                handle.set_functions(self).to(functions);
                handle.set_methods(self).to(methods);
            }
            None => {
                let handle = crate::db::InferredReturnTypes::new(self, functions, methods);
                *self.inferred_return_types_input.write() = Some(handle);
            }
        }
    }

    /// Mark the `FunctionNode` for `fqn` as inactive.
    pub fn deactivate_function_node(&mut self, fqn: &str) {
        use salsa::Setter as _;
        if let Some(&node) = self.function_nodes.get(fqn) {
            node.set_active(self).to(false);
        }
    }

    /// Create or update the `MethodNode` for `(storage.fqcn, storage.name.to_lowercase())`.
    pub fn upsert_method_node(&mut self, storage: &MethodStorage) -> MethodNode {
        use salsa::Setter as _;
        let fqcn = &storage.fqcn;
        let name_lower: Arc<str> = Arc::from(storage.name.to_lowercase().as_str());
        // Copy the existing handle out to release the immutable borrow before
        // calling node.set_*(self), which needs &mut self.
        let existing = self
            .method_nodes
            .get(fqcn.as_ref())
            .and_then(|m| m.get(&name_lower))
            .copied();
        if let Some(node) = existing {
            // Fast-skip identical re-ingest — see `upsert_class_node` for rationale.
            // `inferred_return_type` intentionally not compared / written here;
            // ownership is in the priming-sweep commit phase.
            if node.active(self)
                && node.visibility(self) == storage.visibility
                && node.is_static(self) == storage.is_static
                && node.is_abstract(self) == storage.is_abstract
                && node.is_final(self) == storage.is_final
                && node.is_constructor(self) == storage.is_constructor
                && node.is_pure(self) == storage.is_pure
                && node.is_internal(self) == storage.is_internal
                && node.is_virtual(self) == storage.is_virtual
                && node.deprecated(self) == storage.deprecated
                && node.return_type(self).as_deref() == storage.return_type.as_deref()
                && node.location(self) == storage.location
                && *node.params(self) == *storage.params.as_ref()
                && *node.template_params(self) == *storage.template_params
                && *node.assertions(self) == *storage.assertions
                && *node.throws(self) == *storage.throws
            {
                return node;
            }
            node.set_active(self).to(true);
            node.set_params(self).to(storage.params.clone());
            node.set_return_type(self).to(storage.return_type.clone());
            node.set_template_params(self)
                .to(Arc::from(storage.template_params.as_slice()));
            node.set_assertions(self)
                .to(Arc::from(storage.assertions.as_slice()));
            node.set_throws(self)
                .to(Arc::from(storage.throws.as_slice()));
            node.set_deprecated(self).to(storage.deprecated.clone());
            node.set_docstring(self).to(storage.docstring.clone());
            node.set_is_internal(self).to(storage.is_internal);
            node.set_visibility(self).to(storage.visibility);
            node.set_is_static(self).to(storage.is_static);
            node.set_is_abstract(self).to(storage.is_abstract);
            node.set_is_final(self).to(storage.is_final);
            node.set_is_constructor(self).to(storage.is_constructor);
            node.set_is_pure(self).to(storage.is_pure);
            node.set_is_virtual(self).to(storage.is_virtual);
            node.set_location(self).to(storage.location.clone());
            node
        } else {
            // MethodNode::new takes &mut self; insert after it returns.
            let node = MethodNode::new(
                self,
                fqcn.clone(),
                storage.name.clone(),
                true,
                storage.params.clone(),
                storage.return_type.clone(),
                storage
                    .inferred_return_type
                    .as_ref()
                    .map(|t| Arc::new(t.clone())),
                Arc::from(storage.template_params.as_slice()),
                Arc::from(storage.assertions.as_slice()),
                Arc::from(storage.throws.as_slice()),
                storage.deprecated.clone(),
                storage.docstring.clone(),
                storage.is_internal,
                storage.visibility,
                storage.is_static,
                storage.is_abstract,
                storage.is_final,
                storage.is_constructor,
                storage.is_pure,
                storage.is_virtual,
                storage.location.clone(),
            );
            Arc::make_mut(&mut self.method_nodes)
                .entry(fqcn.clone())
                .or_default()
                .insert(name_lower, node);
            node
        }
    }

    /// Mark all `MethodNode`s owned by `fqcn` as inactive.
    pub fn deactivate_class_methods(&mut self, fqcn: &str) {
        use salsa::Setter as _;
        let nodes: Vec<MethodNode> = match self.method_nodes.get(fqcn) {
            Some(methods) => methods.values().copied().collect(),
            None => return,
        };
        for node in nodes {
            node.set_active(self).to(false);
        }
    }

    /// Deactivate `MethodNode`s for `fqcn` whose lowercased name is not in
    /// `keep_lower`.  Used by `ingest_stub_slice` to prune stale stub methods
    /// when a user file shadows a bundled-stub class with a different method
    /// set.  Active-only check preserves PR21's fast-skip — already-inactive
    /// nodes don't fire a setter.
    pub fn prune_class_methods<T>(&mut self, fqcn: &str, keep_lower: &std::collections::HashSet<T>)
    where
        T: Eq + std::hash::Hash + std::borrow::Borrow<str>,
    {
        use salsa::Setter as _;
        let candidates: Vec<MethodNode> = self
            .method_nodes
            .get(fqcn)
            .map(|m| {
                m.iter()
                    .filter(|(k, _)| !keep_lower.contains(k.as_ref()))
                    .map(|(_, n)| *n)
                    .collect()
            })
            .unwrap_or_default();
        for node in candidates {
            if node.active(self) {
                node.set_active(self).to(false);
            }
        }
    }

    /// Deactivate `PropertyNode`s for `fqcn` whose name is not in `keep`.
    pub fn prune_class_properties<T>(&mut self, fqcn: &str, keep: &std::collections::HashSet<T>)
    where
        T: Eq + std::hash::Hash + std::borrow::Borrow<str>,
    {
        use salsa::Setter as _;
        let candidates: Vec<PropertyNode> = self
            .property_nodes
            .get(fqcn)
            .map(|m| {
                m.iter()
                    .filter(|(k, _)| !keep.contains(k.as_ref()))
                    .map(|(_, n)| *n)
                    .collect()
            })
            .unwrap_or_default();
        for node in candidates {
            if node.active(self) {
                node.set_active(self).to(false);
            }
        }
    }

    /// Deactivate `ClassConstantNode`s for `fqcn` whose name is not in `keep`.
    pub fn prune_class_constants<T>(&mut self, fqcn: &str, keep: &std::collections::HashSet<T>)
    where
        T: Eq + std::hash::Hash + std::borrow::Borrow<str>,
    {
        use salsa::Setter as _;
        let candidates: Vec<ClassConstantNode> = self
            .class_constant_nodes
            .get(fqcn)
            .map(|m| {
                m.iter()
                    .filter(|(k, _)| !keep.contains(k.as_ref()))
                    .map(|(_, n)| *n)
                    .collect()
            })
            .unwrap_or_default();
        for node in candidates {
            if node.active(self) {
                node.set_active(self).to(false);
            }
        }
    }

    /// Create or update the `PropertyNode` for `(storage.fqcn, storage.name)`.
    pub fn upsert_property_node(&mut self, fqcn: &Arc<str>, storage: &PropertyStorage) {
        use salsa::Setter as _;
        let existing = self
            .property_nodes
            .get(fqcn.as_ref())
            .and_then(|m| m.get(storage.name.as_ref()))
            .copied();
        if let Some(node) = existing {
            // Fast-skip identical re-ingest — see `upsert_class_node` for rationale.
            if node.active(self)
                && node.visibility(self) == storage.visibility
                && node.is_static(self) == storage.is_static
                && node.is_readonly(self) == storage.is_readonly
                && node.ty(self) == storage.ty
                && node.location(self) == storage.location
            {
                return;
            }
            node.set_active(self).to(true);
            node.set_ty(self).to(storage.ty.clone());
            node.set_visibility(self).to(storage.visibility);
            node.set_is_static(self).to(storage.is_static);
            node.set_is_readonly(self).to(storage.is_readonly);
            node.set_location(self).to(storage.location.clone());
        } else {
            let node = PropertyNode::new(
                self,
                fqcn.clone(),
                storage.name.clone(),
                true,
                storage.ty.clone(),
                storage.visibility,
                storage.is_static,
                storage.is_readonly,
                storage.location.clone(),
            );
            Arc::make_mut(&mut self.property_nodes)
                .entry(fqcn.clone())
                .or_default()
                .insert(storage.name.clone(), node);
        }
    }

    /// Mark all `PropertyNode`s owned by `fqcn` as inactive.
    pub fn deactivate_class_properties(&mut self, fqcn: &str) {
        use salsa::Setter as _;
        let nodes: Vec<PropertyNode> = match self.property_nodes.get(fqcn) {
            Some(props) => props.values().copied().collect(),
            None => return,
        };
        for node in nodes {
            node.set_active(self).to(false);
        }
    }

    /// Create or update the `ClassConstantNode` for `(fqcn, storage.name)`.
    pub fn upsert_class_constant_node(&mut self, fqcn: &Arc<str>, storage: &ConstantStorage) {
        use salsa::Setter as _;
        let existing = self
            .class_constant_nodes
            .get(fqcn.as_ref())
            .and_then(|m| m.get(storage.name.as_ref()))
            .copied();
        if let Some(node) = existing {
            // Fast-skip identical re-ingest — see `upsert_class_node` for rationale.
            if node.active(self)
                && node.visibility(self) == storage.visibility
                && node.is_final(self) == storage.is_final
                && node.ty(self) == storage.ty
                && node.location(self) == storage.location
            {
                return;
            }
            node.set_active(self).to(true);
            node.set_ty(self).to(storage.ty.clone());
            node.set_visibility(self).to(storage.visibility);
            node.set_is_final(self).to(storage.is_final);
            node.set_location(self).to(storage.location.clone());
        } else {
            let node = ClassConstantNode::new(
                self,
                fqcn.clone(),
                storage.name.clone(),
                true,
                storage.ty.clone(),
                storage.visibility,
                storage.is_final,
                storage.location.clone(),
            );
            Arc::make_mut(&mut self.class_constant_nodes)
                .entry(fqcn.clone())
                .or_default()
                .insert(storage.name.clone(), node);
        }
    }

    /// Create or update the `GlobalConstantNode` for `fqn`.
    pub fn upsert_global_constant_node(&mut self, fqn: Arc<str>, ty: Union) -> GlobalConstantNode {
        use salsa::Setter as _;
        if let Some(&node) = self.global_constant_nodes.get(&fqn) {
            // Fast-skip identical re-ingest — see `upsert_class_node` for rationale.
            if node.active(self) && node.ty(self) == ty {
                return node;
            }
            node.set_active(self).to(true);
            node.set_ty(self).to(ty);
            node
        } else {
            let node = GlobalConstantNode::new(self, fqn.clone(), true, ty);
            Arc::make_mut(&mut self.global_constant_nodes).insert(fqn, node);
            node
        }
    }

    /// Mark the `GlobalConstantNode` for `fqn` as inactive.
    pub fn deactivate_global_constant_node(&mut self, fqn: &str) {
        use salsa::Setter as _;
        if let Some(&node) = self.global_constant_nodes.get(fqn) {
            node.set_active(self).to(false);
        }
    }

    /// Mark all `ClassConstantNode`s owned by `fqcn` as inactive.
    pub fn deactivate_class_constants(&mut self, fqcn: &str) {
        use salsa::Setter as _;
        let nodes: Vec<ClassConstantNode> = match self.class_constant_nodes.get(fqcn) {
            Some(consts) => consts.values().copied().collect(),
            None => return,
        };
        for node in nodes {
            node.set_active(self).to(false);
        }
    }
}
