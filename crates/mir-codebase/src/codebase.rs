use std::sync::Arc;

use dashmap::{DashMap, DashSet};

use crate::interner::Interner;

/// Maps symbol ID → flat list of `(file_id, line, col_start, col_end)`.
///
/// Entries are appended during Pass 2. Duplicates (e.g. from union receivers like
/// `Foo|Foo->method()`) are filtered at insert time. IDs come from
/// `Codebase::symbol_interner` / `Codebase::file_interner`.
///
/// Each entry is 12 bytes (`u32` + `u32` + `u16` + `u16`) with no per-entry
/// allocator overhead beyond the `Vec` backing store.
type ReferenceLocations = DashMap<u32, Vec<(u32, u32, u16, u16)>>;

use crate::storage::{
    ClassStorage, EnumStorage, FunctionStorage, InterfaceStorage, MethodStorage, TraitStorage,
};
use mir_types::Union;

// ---------------------------------------------------------------------------
// Private helper — shared insert logic for reference tracking
// ---------------------------------------------------------------------------

/// Case-insensitive method lookup within a single `own_methods` map.
///
/// Tries an exact key match first (O(1)), then falls back to a linear
/// case-insensitive scan for stubs that store keys in original case.
#[inline]
fn lookup_method<'a>(
    map: &'a indexmap::IndexMap<Arc<str>, Arc<MethodStorage>>,
    name: &str,
) -> Option<&'a Arc<MethodStorage>> {
    map.get(name).or_else(|| {
        map.iter()
            .find(|(k, _)| k.as_ref().eq_ignore_ascii_case(name))
            .map(|(_, v)| v)
    })
}

/// Append `(sym_id, file_id, line, col_start, col_end)` to the reference index,
/// skipping exact duplicates so union receivers like `Foo|Foo->method()` don't
/// inflate the span list.
///
/// Both maps are updated atomically under their respective DashMap shard locks.
#[inline]
fn record_ref(
    sym_locs: &ReferenceLocations,
    file_refs: &DashMap<u32, Vec<u32>>,
    sym_id: u32,
    file_id: u32,
    line: u32,
    col_start: u16,
    col_end: u16,
) {
    {
        let mut entries = sym_locs.entry(sym_id).or_default();
        let span = (file_id, line, col_start, col_end);
        if !entries.contains(&span) {
            entries.push(span);
        }
    }
    {
        let mut refs = file_refs.entry(file_id).or_default();
        if !refs.contains(&sym_id) {
            refs.push(sym_id);
        }
    }
}

// ---------------------------------------------------------------------------
// Compact CSR reference index (post-Pass-2 read-optimised form)
// ---------------------------------------------------------------------------

/// Read-optimised Compressed Sparse Row representation of the reference index.
///
/// Built once by [`Codebase::compact_reference_index`] after Pass 2 finishes.
/// After compaction the build-phase [`DashMap`]s are cleared, freeing the
/// per-entry allocator overhead (~72 bytes per (symbol, file) pair).
///
/// Two CSR views are maintained over the same flat `entries` array:
/// - by symbol: `entries[sym_offsets[id]..sym_offsets[id+1]]`
/// - by file: `by_file[file_offsets[id]..file_offsets[id+1]]` (indirect indices)
#[derive(Debug, Default)]
struct CompactRefIndex {
    /// All spans sorted by `(sym_id, file_id, line, col_start, col_end)`, deduplicated.
    /// Each entry is 16 bytes; total size = `n_refs × 16` with no hash overhead.
    entries: Vec<(u32, u32, u32, u16, u16)>,
    /// CSR offsets keyed by sym_id (length = max_sym_id + 2).
    sym_offsets: Vec<u32>,
    /// Indices into `entries` sorted by `(file_id, sym_id, line, col_start, col_end)`.
    /// Allows O(log n) file-keyed lookups without duplicating the payload.
    by_file: Vec<u32>,
    /// CSR offsets keyed by file_id into `by_file` (length = max_file_id + 2).
    file_offsets: Vec<u32>,
}

// ---------------------------------------------------------------------------
// Codebase — thread-safe global symbol registry
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct Codebase {
    pub classes: DashMap<Arc<str>, ClassStorage>,
    pub interfaces: DashMap<Arc<str>, InterfaceStorage>,
    pub traits: DashMap<Arc<str>, TraitStorage>,
    pub enums: DashMap<Arc<str>, EnumStorage>,
    pub functions: DashMap<Arc<str>, FunctionStorage>,
    pub constants: DashMap<Arc<str>, Union>,

    /// Types of `@var`-annotated global variables, collected in Pass 1.
    /// Key: variable name without the `$` prefix.
    pub global_vars: DashMap<Arc<str>, Union>,
    /// Maps file path → variable names declared with `@var` in that file.
    /// Used by `remove_file_definitions` to purge stale entries on re-analysis.
    file_global_vars: DashMap<Arc<str>, Vec<Arc<str>>>,

    /// Methods referenced during Pass 2 — stored as interned symbol IDs.
    /// Used by the dead-code detector (M18).
    referenced_methods: DashSet<u32>,
    /// Properties referenced during Pass 2 — stored as interned symbol IDs.
    referenced_properties: DashSet<u32>,
    /// Free functions referenced during Pass 2 — stored as interned symbol IDs.
    referenced_functions: DashSet<u32>,

    /// Interner for symbol keys (`"ClassName::method"`, `"ClassName::prop"`, FQN).
    /// Replaces repeated `Arc<str>` copies (16 bytes) with compact `u32` IDs (4 bytes).
    pub symbol_interner: Interner,
    /// Interner for file paths. Same memory rationale as `symbol_interner`.
    pub file_interner: Interner,

    /// Maps symbol ID → flat list of `(file_id, line, col_start, col_end)`.
    /// IDs come from `symbol_interner` / `file_interner`.
    symbol_reference_locations: ReferenceLocations,
    /// Reverse index: file ID → symbol IDs referenced in that file.
    /// Used by `remove_file_definitions` to avoid a full scan of all symbols.
    /// A `Vec` rather than `HashSet`: duplicate sym_ids are guarded at insert time
    /// (same as `symbol_reference_locations`) for the same structural simplicity.
    file_symbol_references: DashMap<u32, Vec<u32>>,

    /// Compact CSR view of the reference index, built by `compact_reference_index()`.
    /// When `Some`, the build-phase DashMaps above are empty and this is the
    /// authoritative source for all reference queries.
    compact_ref_index: std::sync::RwLock<Option<CompactRefIndex>>,
    /// `true` iff `compact_ref_index` is `Some`. Checked atomically before
    /// acquiring any lock, so the fast path during Pass 2 is a single load.
    is_compacted: std::sync::atomic::AtomicBool,

    /// Maps every FQCN (class, interface, trait, enum, function) to the absolute
    /// path of the file that defines it. Populated during Pass 1.
    pub symbol_to_file: DashMap<Arc<str>, Arc<str>>,

    /// Lightweight FQCN index populated by `SymbolTable` before Pass 1.
    /// Enables O(1) "does this symbol exist?" checks before full definitions
    /// are available.
    pub known_symbols: DashSet<Arc<str>>,

    /// Per-file `use` alias maps: alias → FQCN.  Populated during Pass 1.
    ///
    /// Key: absolute file path (as `Arc<str>`).
    /// Value: map of `alias → fully-qualified class name`.
    ///
    /// Exposed as `pub` so that external consumers (e.g. `php-lsp`) can read
    /// import data that mir already collects, instead of reimplementing it.
    pub file_imports: DashMap<Arc<str>, std::collections::HashMap<String, String>>,
    /// Per-file current namespace (if any).  Populated during Pass 1.
    ///
    /// Key: absolute file path (as `Arc<str>`).
    /// Value: the declared namespace string (e.g. `"App\\Controller"`).
    ///
    /// Exposed as `pub` so that external consumers (e.g. `php-lsp`) can read
    /// namespace data that mir already collects, instead of reimplementing it.
    pub file_namespaces: DashMap<Arc<str>, String>,

    /// Whether finalize() has been called.
    finalized: std::sync::atomic::AtomicBool,
}

impl Codebase {
    pub fn new() -> Self {
        Self::default()
    }

    // -----------------------------------------------------------------------
    // Stub injection
    // -----------------------------------------------------------------------

    /// Insert all definitions from `slice` into this codebase.
    ///
    /// Called by generated stub modules (`src/generated/stubs_*.rs`) to register
    /// their pre-compiled definitions. Later insertions overwrite earlier ones,
    /// so custom stubs loaded after PHPStorm stubs act as overrides.
    /// Merge a [`StubSlice`] into the codebase.
    ///
    /// When `slice.file` is `Some`, this method also writes file-keyed metadata:
    /// `symbol_to_file`, `global_vars`, `file_namespaces`, and `file_imports`.
    /// This includes slices produced from PHPStorm stub files — so after this
    /// call, `file_namespaces` and `file_imports` will contain entries keyed by
    /// stub file paths as well as user-code file paths.  That is intentional:
    /// the lazy-load scan iterates `file_imports` but is gated by `type_exists`,
    /// so stub-sourced entries are harmlessly short-circuited there.
    pub fn inject_stub_slice(&self, slice: crate::storage::StubSlice) {
        let file = slice.file.clone();
        for cls in slice.classes {
            if let Some(f) = &file {
                self.symbol_to_file.insert(cls.fqcn.clone(), f.clone());
            }
            self.classes.insert(cls.fqcn.clone(), cls);
        }
        for iface in slice.interfaces {
            if let Some(f) = &file {
                self.symbol_to_file.insert(iface.fqcn.clone(), f.clone());
            }
            self.interfaces.insert(iface.fqcn.clone(), iface);
        }
        for tr in slice.traits {
            if let Some(f) = &file {
                self.symbol_to_file.insert(tr.fqcn.clone(), f.clone());
            }
            self.traits.insert(tr.fqcn.clone(), tr);
        }
        for en in slice.enums {
            if let Some(f) = &file {
                self.symbol_to_file.insert(en.fqcn.clone(), f.clone());
            }
            self.enums.insert(en.fqcn.clone(), en);
        }
        for func in slice.functions {
            if let Some(f) = &file {
                self.symbol_to_file.insert(func.fqn.clone(), f.clone());
            }
            self.functions.insert(func.fqn.clone(), func);
        }
        for (name, ty) in slice.constants {
            self.constants.insert(name, ty);
        }
        if let Some(f) = &file {
            for (name, ty) in slice.global_vars {
                self.register_global_var(f, name, ty);
            }
            if let Some(ns) = slice.namespace {
                self.file_namespaces.insert(f.clone(), ns.to_string());
            }
            if !slice.imports.is_empty() {
                self.file_imports.insert(f.clone(), slice.imports);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Compact reference index
    // -----------------------------------------------------------------------

    /// Convert the build-phase `DashMap` reference index into a compact CSR form.
    ///
    /// Call this once after Pass 2 completes on all files. The method:
    /// 1. Drains the two build-phase `DashMap`s into a single flat `Vec`.
    /// 2. Sorts and deduplicates entries.
    /// 3. Builds two CSR offset arrays (by symbol and by file).
    /// 4. Clears the `DashMap`s (freeing their allocations).
    ///
    /// After this call all reference queries use the compact index. Incremental
    /// re-analysis via [`Self::re_analyze_file`] will automatically decompress the
    /// index back into `DashMap`s on the first write, then recompact can be called
    /// again at the end of that analysis pass.
    pub fn compact_reference_index(&self) {
        // Collect all entries from the build-phase DashMap.
        let mut entries: Vec<(u32, u32, u32, u16, u16)> = self
            .symbol_reference_locations
            .iter()
            .flat_map(|entry| {
                let sym_id = *entry.key();
                entry
                    .value()
                    .iter()
                    .map(move |&(file_id, line, col_start, col_end)| {
                        (sym_id, file_id, line, col_start, col_end)
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        if entries.is_empty() {
            return;
        }

        // Sort by (sym_id, file_id, line, col_start, col_end) and drop exact duplicates.
        entries.sort_unstable();
        entries.dedup();

        let n = entries.len();

        // ---- Build symbol-keyed CSR offsets --------------------------------
        let max_sym = entries.iter().map(|&(s, ..)| s).max().unwrap_or(0) as usize;
        let mut sym_offsets = vec![0u32; max_sym + 2];
        for &(sym_id, ..) in &entries {
            sym_offsets[sym_id as usize + 1] += 1;
        }
        for i in 1..sym_offsets.len() {
            sym_offsets[i] += sym_offsets[i - 1];
        }

        // ---- Build file-keyed indirect index --------------------------------
        // `by_file[i]` is an index into `entries`; the slice is sorted by
        // `(file_id, sym_id, line, col_start, col_end)` so CSR offsets can be computed cheaply.
        let max_file = entries.iter().map(|&(_, f, ..)| f).max().unwrap_or(0) as usize;
        let mut by_file: Vec<u32> = (0..n as u32).collect();
        by_file.sort_unstable_by_key(|&i| {
            let (sym_id, file_id, line, col_start, col_end) = entries[i as usize];
            (file_id, sym_id, line, col_start, col_end)
        });

        let mut file_offsets = vec![0u32; max_file + 2];
        for &idx in &by_file {
            let file_id = entries[idx as usize].1;
            file_offsets[file_id as usize + 1] += 1;
        }
        for i in 1..file_offsets.len() {
            file_offsets[i] += file_offsets[i - 1];
        }

        *self.compact_ref_index.write().unwrap() = Some(CompactRefIndex {
            entries,
            sym_offsets,
            by_file,
            file_offsets,
        });
        self.is_compacted
            .store(true, std::sync::atomic::Ordering::Release);

        // Free build-phase allocations.
        self.symbol_reference_locations.clear();
        self.file_symbol_references.clear();
    }

    /// Decompress the compact index back into the build-phase `DashMap`s.
    ///
    /// Called automatically by write methods when the compact index is live.
    /// This makes incremental re-analysis transparent: callers never need to
    /// know whether the index is compacted or not.
    fn ensure_expanded(&self) {
        // Fast path: not compacted — one atomic load, no lock.
        if !self.is_compacted.load(std::sync::atomic::Ordering::Acquire) {
            return;
        }
        // Slow path: acquire write lock and decompress.
        let mut guard = self.compact_ref_index.write().unwrap();
        if let Some(ci) = guard.take() {
            for &(sym_id, file_id, line, col_start, col_end) in &ci.entries {
                record_ref(
                    &self.symbol_reference_locations,
                    &self.file_symbol_references,
                    sym_id,
                    file_id,
                    line,
                    col_start,
                    col_end,
                );
            }
            self.is_compacted
                .store(false, std::sync::atomic::Ordering::Release);
        }
        // If another thread already decompressed (guard is now None), we're done.
    }

    /// Reset the finalization flag so that `finalize()` will run again.
    ///
    /// Use this when new class definitions have been added after an initial
    /// `finalize()` call (e.g., lazily loaded via PSR-4) and the inheritance
    /// graph needs to be rebuilt.
    pub fn invalidate_finalization(&self) {
        self.finalized
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    // -----------------------------------------------------------------------
    // Incremental: remove all definitions from a single file
    // -----------------------------------------------------------------------

    /// Remove all definitions and outgoing reference locations contributed by the given file.
    /// This clears classes, interfaces, traits, enums, functions, and constants
    /// whose defining file matches `file_path`, the file's import and namespace entries,
    /// and all entries in symbol_reference_locations that originated from this file.
    /// After calling this, `invalidate_finalization()` is called so the next `finalize()`
    /// rebuilds inheritance.
    pub fn remove_file_definitions(&self, file_path: &str) {
        // Collect all symbols defined in this file
        let symbols: Vec<Arc<str>> = self
            .symbol_to_file
            .iter()
            .filter(|entry| entry.value().as_ref() == file_path)
            .map(|entry| entry.key().clone())
            .collect();

        // Remove each symbol from its respective map and from symbol_to_file.
        for sym in &symbols {
            self.classes.remove(sym.as_ref());
            self.interfaces.remove(sym.as_ref());
            self.traits.remove(sym.as_ref());
            self.enums.remove(sym.as_ref());
            self.functions.remove(sym.as_ref());
            self.constants.remove(sym.as_ref());
            self.symbol_to_file.remove(sym.as_ref());
            self.known_symbols.remove(sym.as_ref());
        }

        // Remove file-level metadata
        self.file_imports.remove(file_path);
        self.file_namespaces.remove(file_path);

        // Remove @var-annotated global variables declared in this file
        if let Some((_, var_names)) = self.file_global_vars.remove(file_path) {
            for name in var_names {
                self.global_vars.remove(name.as_ref());
            }
        }

        // Ensure the reference index is in DashMap form so the removal below works.
        self.ensure_expanded();

        // Remove reference locations contributed by this file.
        // Use the reverse index to avoid a full scan of all symbols.
        if let Some(file_id) = self.file_interner.get_id(file_path) {
            if let Some((_, sym_ids)) = self.file_symbol_references.remove(&file_id) {
                for sym_id in sym_ids {
                    if let Some(mut entries) = self.symbol_reference_locations.get_mut(&sym_id) {
                        entries.retain(|&(fid, ..)| fid != file_id);
                    }
                }
            }
        }

        self.invalidate_finalization();
    }

    // -----------------------------------------------------------------------
    // Structural snapshot — skip finalize() on body-only changes
    // -----------------------------------------------------------------------

    /// Restore the pre-computed ancestor list for a single class or interface.
    ///
    /// Called by `re_analyze_file` when the Salsa `class_ancestors` query
    /// confirms that the inheritance structure of a file is unchanged, so
    /// we don't have to walk the hierarchy in `finalize()` again.
    pub fn restore_ancestors(&self, fqcn: &str, ancestors: Arc<[Arc<str>]>) {
        if let Some(mut cls) = self.classes.get_mut(fqcn) {
            cls.all_parents = ancestors.to_vec();
        } else if let Some(mut iface) = self.interfaces.get_mut(fqcn) {
            iface.all_parents = ancestors.to_vec();
        }
    }

    /// Mark the codebase as finalized without running the full `finalize()` sweep.
    ///
    /// Call this after `restore_ancestors` has been called for all symbols in
    /// the re-analyzed file to signal that the inheritance graph is up to date.
    pub fn mark_finalized(&self) {
        self.finalized
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    // -----------------------------------------------------------------------
    // Global variable registry
    // -----------------------------------------------------------------------

    /// Record an `@var`-annotated global variable type discovered in Pass 1.
    /// If the same variable is annotated in multiple files, the last write wins.
    pub fn register_global_var(&self, file: &Arc<str>, name: Arc<str>, ty: Union) {
        self.file_global_vars
            .entry(file.clone())
            .or_default()
            .push(name.clone());
        self.global_vars.insert(name, ty);
    }

    // -----------------------------------------------------------------------
    // Lookups
    // -----------------------------------------------------------------------

    /// Direct lookup of a method's `inferred_return_type` on the owner
    /// class/trait/interface/enum.  Does not walk the inheritance chain —
    /// callers are expected to know the owning FQCN already (e.g. from
    /// `db::lookup_method_in_chain`).
    pub fn method_inferred_return_type(
        &self,
        owner_fqcn: &str,
        method_name: &str,
    ) -> Option<Union> {
        if let Some(cls) = self.classes.get(owner_fqcn) {
            if let Some(m) = lookup_method(&cls.own_methods, method_name) {
                return m.inferred_return_type.clone();
            }
        }
        if let Some(tr) = self.traits.get(owner_fqcn) {
            if let Some(m) = lookup_method(&tr.own_methods, method_name) {
                return m.inferred_return_type.clone();
            }
        }
        if let Some(iface) = self.interfaces.get(owner_fqcn) {
            if let Some(m) = lookup_method(&iface.own_methods, method_name) {
                return m.inferred_return_type.clone();
            }
        }
        if let Some(en) = self.enums.get(owner_fqcn) {
            if let Some(m) = lookup_method(&en.own_methods, method_name) {
                return m.inferred_return_type.clone();
            }
        }
        None
    }

    /// Whether a class/interface/trait/enum with this FQCN exists.
    pub fn type_exists(&self, fqcn: &str) -> bool {
        self.classes.contains_key(fqcn)
            || self.interfaces.contains_key(fqcn)
            || self.traits.contains_key(fqcn)
            || self.enums.contains_key(fqcn)
    }

    pub fn function_exists(&self, fqn: &str) -> bool {
        self.functions.contains_key(fqn)
    }

    /// Returns true if the class is declared abstract.
    /// Used to suppress `UndefinedMethod` on abstract class receivers: the concrete
    /// subclass is expected to implement the method, matching Psalm errorLevel=3 behaviour.
    pub fn is_abstract_class(&self, fqcn: &str) -> bool {
        self.classes.get(fqcn).is_some_and(|c| c.is_abstract)
    }

    /// Return the declared template params for `fqcn` (class or interface), or
    /// an empty vec if the type is not found or has no templates.
    pub fn get_class_template_params(&self, fqcn: &str) -> Vec<crate::storage::TemplateParam> {
        if let Some(cls) = self.classes.get(fqcn) {
            return cls.template_params.clone();
        }
        if let Some(iface) = self.interfaces.get(fqcn) {
            return iface.template_params.clone();
        }
        if let Some(tr) = self.traits.get(fqcn) {
            return tr.template_params.clone();
        }
        vec![]
    }

    /// Walk the parent chain collecting template bindings from `@extends` type args.
    ///
    /// For `class UserRepo extends BaseRepo` with `@extends BaseRepo<User>`, this returns
    /// `{ T → User }` where `T` is `BaseRepo`'s declared template parameter.
    pub fn get_inherited_template_bindings(
        &self,
        fqcn: &str,
    ) -> std::collections::HashMap<Arc<str>, Union> {
        let mut bindings = std::collections::HashMap::new();
        let mut current = fqcn.to_string();

        loop {
            let (parent_fqcn, extends_type_args) = {
                let cls = match self.classes.get(current.as_str()) {
                    Some(c) => c,
                    None => break,
                };
                let parent = match &cls.parent {
                    Some(p) => p.clone(),
                    None => break,
                };
                let args = cls.extends_type_args.clone();
                (parent, args)
            };

            if !extends_type_args.is_empty() {
                let parent_tps = self.get_class_template_params(&parent_fqcn);
                for (tp, ty) in parent_tps.iter().zip(extends_type_args.iter()) {
                    bindings
                        .entry(tp.name.clone())
                        .or_insert_with(|| ty.clone());
                }
            }

            current = parent_fqcn.to_string();
        }

        bindings
    }

    /// Returns true if the class (or any ancestor/trait) defines a `__get` magic method.
    /// Such classes allow arbitrary property access, suppressing UndefinedProperty.
    pub fn has_magic_get(&self, fqcn: &str) -> bool {
        if let Some(cls) = self.classes.get(fqcn) {
            if lookup_method(&cls.own_methods, "__get").is_some() {
                return true;
            }
            let traits = cls.traits.clone();
            let parents = cls.all_parents.clone();
            drop(cls);
            for tr_fqcn in &traits {
                if let Some(tr) = self.traits.get(tr_fqcn.as_ref()) {
                    if lookup_method(&tr.own_methods, "__get").is_some() {
                        return true;
                    }
                }
            }
            for anc in &parents {
                if let Some(anc_cls) = self.classes.get(anc.as_ref()) {
                    if lookup_method(&anc_cls.own_methods, "__get").is_some() {
                        return true;
                    }
                    let anc_traits = anc_cls.traits.clone();
                    drop(anc_cls);
                    for tr_fqcn in &anc_traits {
                        if let Some(tr) = self.traits.get(tr_fqcn.as_ref()) {
                            if lookup_method(&tr.own_methods, "__get").is_some() {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Returns true if the class (or any of its ancestors) has a parent/interface/trait
    /// that is NOT present in the codebase.  Used to suppress `UndefinedMethod` false
    /// positives: if a method might be inherited from an unscanned external class we
    /// cannot confirm or deny its existence.
    ///
    /// We use the pre-computed `all_parents` list (built during finalization) rather
    /// than recursive DashMap lookups to avoid potential deadlocks.
    pub fn has_unknown_ancestor(&self, fqcn: &str) -> bool {
        // For interfaces: check whether any parent interface is unknown.
        if let Some(iface) = self.interfaces.get(fqcn) {
            let parents = iface.all_parents.clone();
            drop(iface);
            for p in &parents {
                if !self.type_exists(p.as_ref()) {
                    return true;
                }
            }
            return false;
        }

        // Clone the data we need so the DashMap ref is dropped before any further lookups.
        let (parent, interfaces, traits, all_parents) = {
            let Some(cls) = self.classes.get(fqcn) else {
                return false;
            };
            (
                cls.parent.clone(),
                cls.interfaces.clone(),
                cls.traits.clone(),
                cls.all_parents.clone(),
            )
        };

        // Fast path: check direct parent/interfaces/traits
        if let Some(ref p) = parent {
            if !self.type_exists(p.as_ref()) {
                return true;
            }
        }
        for iface in &interfaces {
            if !self.type_exists(iface.as_ref()) {
                return true;
            }
        }
        for tr in &traits {
            if !self.type_exists(tr.as_ref()) {
                return true;
            }
        }

        // Also check the full ancestor chain (pre-computed during finalization)
        for ancestor in &all_parents {
            if !self.type_exists(ancestor.as_ref()) {
                return true;
            }
        }

        false
    }

    /// Resolve a short class/function name to its FQCN using the import table
    /// and namespace recorded for `file` during Pass 1.
    ///
    /// - Names already containing `\` (after stripping a leading `\`) are
    ///   returned as-is (already fully qualified).
    /// - `self`, `parent`, `static` are returned unchanged (caller handles them).
    pub fn resolve_class_name(&self, file: &str, name: &str) -> String {
        let name = name.trim_start_matches('\\');
        if name.is_empty() {
            return name.to_string();
        }
        // Fully qualified absolute paths start with '\' (already stripped above).
        // Names containing '\' but not starting with it may be:
        //   - Already-resolved FQCNs (e.g. Frontify\Util\Foo) — check type_exists
        //   - Qualified relative names (e.g. Option\Some from within Frontify\Utility) — need namespace prefix
        if name.contains('\\') {
            // Check if the leading segment matches a use-import alias
            let first_segment = name.split('\\').next().unwrap_or(name);
            if let Some(imports) = self.file_imports.get(file) {
                if let Some(resolved_prefix) = imports.get(first_segment) {
                    let rest = &name[first_segment.len()..]; // includes leading '\'
                    return format!("{resolved_prefix}{rest}");
                }
            }
            // If already known in codebase as-is, it's FQCN — trust it
            if self.type_exists(name) {
                return name.to_string();
            }
            // Otherwise it's a relative qualified name — prepend the file namespace
            if let Some(ns) = self.file_namespaces.get(file) {
                let qualified = format!("{}\\{}", *ns, name);
                if self.type_exists(&qualified) {
                    return qualified;
                }
            }
            return name.to_string();
        }
        // Built-in pseudo-types / keywords handled by the caller
        match name {
            "self" | "parent" | "static" | "this" => return name.to_string(),
            _ => {}
        }
        // Check use aliases for this file (PHP class names are case-insensitive)
        if let Some(imports) = self.file_imports.get(file) {
            if let Some(resolved) = imports.get(name) {
                return resolved.clone();
            }
            // Fall back to case-insensitive alias lookup
            let name_lower = name.to_lowercase();
            for (alias, resolved) in imports.iter() {
                if alias.to_lowercase() == name_lower {
                    return resolved.clone();
                }
            }
        }
        // Qualify with the file's namespace if one exists
        if let Some(ns) = self.file_namespaces.get(file) {
            let qualified = format!("{}\\{}", *ns, name);
            // If the namespaced version exists in the codebase, use it.
            // Otherwise fall back to the global (unqualified) name if that exists.
            // This handles `DateTimeInterface`, `Exception`, etc. used without import
            // while not overriding user-defined classes in namespaces.
            if self.type_exists(&qualified) {
                return qualified;
            }
            if self.type_exists(name) {
                return name.to_string();
            }
            return qualified;
        }
        name.to_string()
    }

    // -----------------------------------------------------------------------
    // Definition location lookups
    // -----------------------------------------------------------------------

    /// Look up the definition location of any symbol (class, interface, trait, enum, function).
    /// Returns the file path and byte offsets.
    pub fn get_symbol_location(&self, fqcn: &str) -> Option<crate::storage::Location> {
        if let Some(cls) = self.classes.get(fqcn) {
            return cls.location.clone();
        }
        if let Some(iface) = self.interfaces.get(fqcn) {
            return iface.location.clone();
        }
        if let Some(tr) = self.traits.get(fqcn) {
            return tr.location.clone();
        }
        if let Some(en) = self.enums.get(fqcn) {
            return en.location.clone();
        }
        if let Some(func) = self.functions.get(fqcn) {
            return func.location.clone();
        }
        None
    }

    /// Look up the definition location of a class member (method, property, constant).
    pub fn get_member_location(
        &self,
        fqcn: &str,
        member_name: &str,
    ) -> Option<crate::storage::Location> {
        let method_lower = member_name.to_lowercase();
        // Methods: own → traits → ancestors (own + traits).
        if let Some(loc) = self.find_method_location_in_chain(fqcn, &method_lower) {
            return loc;
        }
        // Properties: own → ancestors.
        if let Some(loc) = self.find_property_location_in_chain(fqcn, member_name) {
            return loc;
        }
        // Check class constants
        if let Some(cls) = self.classes.get(fqcn) {
            if let Some(c) = cls.own_constants.get(member_name) {
                return c.location.clone();
            }
        }
        // Check interface constants
        if let Some(iface) = self.interfaces.get(fqcn) {
            if let Some(c) = iface.own_constants.get(member_name) {
                return c.location.clone();
            }
        }
        // Check trait constants
        if let Some(tr) = self.traits.get(fqcn) {
            if let Some(c) = tr.own_constants.get(member_name) {
                return c.location.clone();
            }
        }
        // Check enum constants and cases
        if let Some(en) = self.enums.get(fqcn) {
            if let Some(c) = en.own_constants.get(member_name) {
                return c.location.clone();
            }
            if let Some(case) = en.cases.get(member_name) {
                return case.location.clone();
            }
        }
        None
    }

    // -----------------------------------------------------------------------
    // Reference tracking (M18 dead-code detection)
    // -----------------------------------------------------------------------

    /// Mark a method as referenced from user code.
    pub fn mark_method_referenced(&self, fqcn: &str, method_name: &str) {
        let key = format!("{}::{}", fqcn, method_name.to_lowercase());
        let id = self.symbol_interner.intern_str(&key);
        self.referenced_methods.insert(id);
    }

    /// Mark a property as referenced from user code.
    pub fn mark_property_referenced(&self, fqcn: &str, prop_name: &str) {
        let key = format!("{fqcn}::{prop_name}");
        let id = self.symbol_interner.intern_str(&key);
        self.referenced_properties.insert(id);
    }

    /// Mark a free function as referenced from user code.
    pub fn mark_function_referenced(&self, fqn: &str) {
        let id = self.symbol_interner.intern_str(fqn);
        self.referenced_functions.insert(id);
    }

    pub fn is_method_referenced(&self, fqcn: &str, method_name: &str) -> bool {
        let key = format!("{}::{}", fqcn, method_name.to_lowercase());
        match self.symbol_interner.get_id(&key) {
            Some(id) => self.referenced_methods.contains(&id),
            None => false,
        }
    }

    pub fn is_property_referenced(&self, fqcn: &str, prop_name: &str) -> bool {
        let key = format!("{fqcn}::{prop_name}");
        match self.symbol_interner.get_id(&key) {
            Some(id) => self.referenced_properties.contains(&id),
            None => false,
        }
    }

    pub fn is_function_referenced(&self, fqn: &str) -> bool {
        match self.symbol_interner.get_id(fqn) {
            Some(id) => self.referenced_functions.contains(&id),
            None => false,
        }
    }

    /// Record a method reference with its source location.
    /// Also updates the referenced_methods DashSet for dead-code detection.
    pub fn mark_method_referenced_at(
        &self,
        fqcn: &str,
        method_name: &str,
        file: Arc<str>,
        line: u32,
        col_start: u16,
        col_end: u16,
    ) {
        let key = format!("{}::{}", fqcn, method_name.to_lowercase());
        self.ensure_expanded();
        let sym_id = self.symbol_interner.intern_str(&key);
        let file_id = self.file_interner.intern(file);
        self.referenced_methods.insert(sym_id);
        record_ref(
            &self.symbol_reference_locations,
            &self.file_symbol_references,
            sym_id,
            file_id,
            line,
            col_start,
            col_end,
        );
    }

    /// Record a property reference with its source location.
    /// Also updates the referenced_properties DashSet for dead-code detection.
    pub fn mark_property_referenced_at(
        &self,
        fqcn: &str,
        prop_name: &str,
        file: Arc<str>,
        line: u32,
        col_start: u16,
        col_end: u16,
    ) {
        let key = format!("{fqcn}::{prop_name}");
        self.ensure_expanded();
        let sym_id = self.symbol_interner.intern_str(&key);
        let file_id = self.file_interner.intern(file);
        self.referenced_properties.insert(sym_id);
        record_ref(
            &self.symbol_reference_locations,
            &self.file_symbol_references,
            sym_id,
            file_id,
            line,
            col_start,
            col_end,
        );
    }

    /// Record a function reference with its source location.
    /// Also updates the referenced_functions DashSet for dead-code detection.
    pub fn mark_function_referenced_at(
        &self,
        fqn: &str,
        file: Arc<str>,
        line: u32,
        col_start: u16,
        col_end: u16,
    ) {
        self.ensure_expanded();
        let sym_id = self.symbol_interner.intern_str(fqn);
        let file_id = self.file_interner.intern(file);
        self.referenced_functions.insert(sym_id);
        record_ref(
            &self.symbol_reference_locations,
            &self.file_symbol_references,
            sym_id,
            file_id,
            line,
            col_start,
            col_end,
        );
    }

    /// Record a class reference (e.g. `new Foo()`) with its source location.
    /// Does not update any dead-code DashSet — class instantiation tracking is
    /// separate from method/property/function dead-code detection.
    pub fn mark_class_referenced_at(
        &self,
        fqcn: &str,
        file: Arc<str>,
        line: u32,
        col_start: u16,
        col_end: u16,
    ) {
        self.ensure_expanded();
        let sym_id = self.symbol_interner.intern_str(fqcn);
        let file_id = self.file_interner.intern(file);
        record_ref(
            &self.symbol_reference_locations,
            &self.file_symbol_references,
            sym_id,
            file_id,
            line,
            col_start,
            col_end,
        );
    }

    /// Replay cached reference locations for a file into the reference index.
    /// Called on cache hits to avoid re-running Pass 2 just to rebuild the index.
    /// `locs` is a slice of `(symbol_key, line, col_start, col_end)` as stored in the cache.
    pub fn replay_reference_locations(&self, file: Arc<str>, locs: &[(String, u32, u16, u16)]) {
        if locs.is_empty() {
            return;
        }
        self.ensure_expanded();
        let file_id = self.file_interner.intern(file);
        for (symbol_key, line, col_start, col_end) in locs {
            let sym_id = self.symbol_interner.intern_str(symbol_key);
            record_ref(
                &self.symbol_reference_locations,
                &self.file_symbol_references,
                sym_id,
                file_id,
                *line,
                *col_start,
                *col_end,
            );
        }
    }

    /// Return all reference locations for `symbol` as `Vec<(file, line, col_start, col_end)>`.
    /// Returns an empty Vec if the symbol has no recorded references.
    pub fn get_reference_locations(&self, symbol: &str) -> Vec<(Arc<str>, u32, u16, u16)> {
        let Some(sym_id) = self.symbol_interner.get_id(symbol) else {
            return Vec::new();
        };
        // Fast path: compact CSR index.
        if let Some(ref ci) = *self.compact_ref_index.read().unwrap() {
            let id = sym_id as usize;
            if id + 1 >= ci.sym_offsets.len() {
                return Vec::new();
            }
            let start = ci.sym_offsets[id] as usize;
            let end = ci.sym_offsets[id + 1] as usize;
            return ci.entries[start..end]
                .iter()
                .map(|&(_, file_id, line, col_start, col_end)| {
                    (self.file_interner.get(file_id), line, col_start, col_end)
                })
                .collect();
        }
        // Slow path: build-phase DashMap.
        let Some(entries) = self.symbol_reference_locations.get(&sym_id) else {
            return Vec::new();
        };
        entries
            .iter()
            .map(|&(file_id, line, col_start, col_end)| {
                (self.file_interner.get(file_id), line, col_start, col_end)
            })
            .collect()
    }

    /// Extract all reference locations recorded for `file` as
    /// `(symbol_key, line, col_start, col_end)` tuples.
    /// Used by the cache layer to persist per-file reference data between runs.
    pub fn extract_file_reference_locations(&self, file: &str) -> Vec<(Arc<str>, u32, u16, u16)> {
        let Some(file_id) = self.file_interner.get_id(file) else {
            return Vec::new();
        };
        // Fast path: compact CSR index.
        if let Some(ref ci) = *self.compact_ref_index.read().unwrap() {
            let id = file_id as usize;
            if id + 1 >= ci.file_offsets.len() {
                return Vec::new();
            }
            let start = ci.file_offsets[id] as usize;
            let end = ci.file_offsets[id + 1] as usize;
            return ci.by_file[start..end]
                .iter()
                .map(|&entry_idx| {
                    let (sym_id, _, line, col_start, col_end) = ci.entries[entry_idx as usize];
                    (self.symbol_interner.get(sym_id), line, col_start, col_end)
                })
                .collect();
        }
        // Slow path: build-phase DashMaps.
        let Some(sym_ids) = self.file_symbol_references.get(&file_id) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for &sym_id in sym_ids.iter() {
            let Some(entries) = self.symbol_reference_locations.get(&sym_id) else {
                continue;
            };
            let sym_key = self.symbol_interner.get(sym_id);
            for &(entry_file_id, line, col_start, col_end) in entries.iter() {
                if entry_file_id == file_id {
                    out.push((sym_key.clone(), line, col_start, col_end));
                }
            }
        }
        out
    }

    /// Returns true if the given file has any recorded symbol references.
    pub fn file_has_symbol_references(&self, file: &str) -> bool {
        let Some(file_id) = self.file_interner.get_id(file) else {
            return false;
        };
        // Check compact index first.
        if let Some(ref ci) = *self.compact_ref_index.read().unwrap() {
            let id = file_id as usize;
            return id + 1 < ci.file_offsets.len() && ci.file_offsets[id] < ci.file_offsets[id + 1];
        }
        self.file_symbol_references.contains_key(&file_id)
    }

    // -----------------------------------------------------------------------
    // Finalization
    // -----------------------------------------------------------------------

    /// Must be called after all files have been parsed (pass 1 complete).
    /// Computes `all_parents` for every class and interface and resolves
    /// `@psalm-import-type` declarations.
    pub fn finalize(&self) {
        if self.finalized.load(std::sync::atomic::Ordering::SeqCst) {
            return;
        }

        // 1. Compute all_parents for classes.
        let class_keys: Vec<Arc<str>> = self.classes.iter().map(|e| e.key().clone()).collect();
        for fqcn in &class_keys {
            let parents = self.compute_class_ancestors(fqcn);
            if let Some(mut cls) = self.classes.get_mut(fqcn.as_ref()) {
                cls.all_parents = parents;
            }
        }

        // 2. Compute all_parents for interfaces.
        let iface_keys: Vec<Arc<str>> = self.interfaces.iter().map(|e| e.key().clone()).collect();
        for fqcn in &iface_keys {
            let parents = self.compute_interface_ancestors(fqcn);
            if let Some(mut iface) = self.interfaces.get_mut(fqcn.as_ref()) {
                iface.all_parents = parents;
            }
        }

        // 3. Resolve @psalm-import-type declarations
        // Collect imports first to avoid holding two locks simultaneously.
        type PendingImports = Vec<(Arc<str>, Vec<(Arc<str>, Arc<str>, Arc<str>)>)>;
        let pending: PendingImports = self
            .classes
            .iter()
            .filter(|e| !e.pending_import_types.is_empty())
            .map(|e| (e.key().clone(), e.pending_import_types.clone()))
            .collect();
        for (dst_fqcn, imports) in pending {
            let mut resolved: std::collections::HashMap<Arc<str>, mir_types::Union> =
                std::collections::HashMap::new();
            for (local, original, from_class) in &imports {
                if let Some(src_cls) = self.classes.get(from_class.as_ref()) {
                    if let Some(ty) = src_cls.type_aliases.get(original.as_ref()) {
                        resolved.insert(local.clone(), ty.clone());
                    }
                }
            }
            if !resolved.is_empty() {
                if let Some(mut dst_cls) = self.classes.get_mut(dst_fqcn.as_ref()) {
                    for (k, v) in resolved {
                        dst_cls.type_aliases.insert(k, v);
                    }
                }
            }
        }

        self.finalized
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Walk own → traits → ancestors looking up a method's location.  The
    /// outer `Option` indicates whether the method was found; the inner
    /// `Option` is its (possibly absent) location.
    fn find_method_location_in_chain(
        &self,
        fqcn: &str,
        method_lower: &str,
    ) -> Option<Option<crate::storage::Location>> {
        if let Some(cls) = self.classes.get(fqcn) {
            if let Some(m) = lookup_method(&cls.own_methods, method_lower) {
                return Some(m.location.clone());
            }
            let traits = cls.traits.clone();
            let parents = cls.all_parents.clone();
            drop(cls);
            for tr_fqcn in &traits {
                if let Some(tr) = self.traits.get(tr_fqcn.as_ref()) {
                    if let Some(m) = lookup_method(&tr.own_methods, method_lower) {
                        return Some(m.location.clone());
                    }
                }
            }
            for anc in &parents {
                if let Some(anc_cls) = self.classes.get(anc.as_ref()) {
                    if let Some(m) = lookup_method(&anc_cls.own_methods, method_lower) {
                        return Some(m.location.clone());
                    }
                    let anc_traits = anc_cls.traits.clone();
                    drop(anc_cls);
                    for tr_fqcn in &anc_traits {
                        if let Some(tr) = self.traits.get(tr_fqcn.as_ref()) {
                            if let Some(m) = lookup_method(&tr.own_methods, method_lower) {
                                return Some(m.location.clone());
                            }
                        }
                    }
                } else if let Some(iface) = self.interfaces.get(anc.as_ref()) {
                    if let Some(m) = lookup_method(&iface.own_methods, method_lower) {
                        return Some(m.location.clone());
                    }
                }
            }
            return None;
        }
        if let Some(iface) = self.interfaces.get(fqcn) {
            if let Some(m) = lookup_method(&iface.own_methods, method_lower) {
                return Some(m.location.clone());
            }
            let parents = iface.all_parents.clone();
            drop(iface);
            for p in &parents {
                if let Some(parent_iface) = self.interfaces.get(p.as_ref()) {
                    if let Some(m) = lookup_method(&parent_iface.own_methods, method_lower) {
                        return Some(m.location.clone());
                    }
                }
            }
            return None;
        }
        if let Some(tr) = self.traits.get(fqcn) {
            if let Some(m) = lookup_method(&tr.own_methods, method_lower) {
                return Some(m.location.clone());
            }
        }
        if let Some(en) = self.enums.get(fqcn) {
            if let Some(m) = lookup_method(&en.own_methods, method_lower) {
                return Some(m.location.clone());
            }
        }
        None
    }

    /// Walk own → traits → ancestors looking up a property's location.
    fn find_property_location_in_chain(
        &self,
        fqcn: &str,
        prop_name: &str,
    ) -> Option<Option<crate::storage::Location>> {
        if let Some(cls) = self.classes.get(fqcn) {
            if let Some(p) = cls.own_properties.get(prop_name) {
                return Some(p.location.clone());
            }
            let traits = cls.traits.clone();
            let parents = cls.all_parents.clone();
            drop(cls);
            for tr_fqcn in &traits {
                if let Some(tr) = self.traits.get(tr_fqcn.as_ref()) {
                    if let Some(p) = tr.own_properties.get(prop_name) {
                        return Some(p.location.clone());
                    }
                }
            }
            for anc in &parents {
                if let Some(anc_cls) = self.classes.get(anc.as_ref()) {
                    if let Some(p) = anc_cls.own_properties.get(prop_name) {
                        return Some(p.location.clone());
                    }
                }
            }
            return None;
        }
        if let Some(tr) = self.traits.get(fqcn) {
            if let Some(p) = tr.own_properties.get(prop_name) {
                return Some(p.location.clone());
            }
        }
        None
    }

    /// Compute the ancestor list (`all_parents`) for a class.  Ordering matches
    /// the historical `ensure_finalized` walk: parent + parent's ancestors,
    /// then each implemented interface + its ancestors, then directly-used
    /// traits.  A local `seen` set deduplicates diamond ancestors.
    fn compute_class_ancestors(&self, fqcn: &str) -> Vec<Arc<str>> {
        let mut result: Vec<Arc<str>> = Vec::new();
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        self.collect_class_ancestors(fqcn, &mut result, &mut seen);
        result
    }

    fn collect_class_ancestors(
        &self,
        fqcn: &str,
        result: &mut Vec<Arc<str>>,
        seen: &mut std::collections::HashSet<String>,
    ) {
        let (parent, interfaces, traits) = match self.classes.get(fqcn) {
            Some(cls) => (
                cls.parent.clone(),
                cls.interfaces.clone(),
                cls.traits.clone(),
            ),
            None => return,
        };

        if let Some(p) = parent {
            if seen.insert(p.to_string()) {
                result.push(Arc::clone(&p));
                self.collect_class_ancestors(&p, result, seen);
            }
        }
        for iface in &interfaces {
            if seen.insert(iface.to_string()) {
                result.push(Arc::clone(iface));
                self.collect_interface_ancestors(iface, result, seen);
            }
        }
        for t in traits {
            if seen.insert(t.to_string()) {
                result.push(t);
            }
        }
    }

    /// Compute the ancestor list for an interface by walking `extends` chains.
    fn compute_interface_ancestors(&self, fqcn: &str) -> Vec<Arc<str>> {
        let mut result: Vec<Arc<str>> = Vec::new();
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        self.collect_interface_ancestors(fqcn, &mut result, &mut seen);
        result
    }

    fn collect_interface_ancestors(
        &self,
        fqcn: &str,
        result: &mut Vec<Arc<str>>,
        seen: &mut std::collections::HashSet<String>,
    ) {
        let extends = match self.interfaces.get(fqcn) {
            Some(iface) => iface.extends.clone(),
            None => return,
        };
        for e in &extends {
            if seen.insert(e.to_string()) {
                result.push(Arc::clone(e));
                self.collect_interface_ancestors(e, result, seen);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CodebaseBuilder — compose a finalized Codebase from per-file StubSlices
// ---------------------------------------------------------------------------

/// Incremental builder that accumulates [`crate::storage::StubSlice`] values
/// into a fresh [`Codebase`] and finalizes it on demand.
///
/// Designed for callers (e.g. salsa queries in downstream consumers) that want
/// to treat Pass-1 definition collection as a pure function from source to
/// `StubSlice`, then compose the slices into a full codebase outside the
/// collector.
pub struct CodebaseBuilder {
    cb: Codebase,
}

impl CodebaseBuilder {
    pub fn new() -> Self {
        Self {
            cb: Codebase::new(),
        }
    }

    /// Inject a single slice. Later injections overwrite earlier definitions
    /// with the same FQN, matching [`Codebase::inject_stub_slice`] semantics.
    pub fn add(&mut self, slice: crate::storage::StubSlice) {
        self.cb.inject_stub_slice(slice);
    }

    /// Finalize inheritance graphs and return the built `Codebase`.
    pub fn finalize(self) -> Codebase {
        self.cb.finalize();
        self.cb
    }

    /// Access the in-progress codebase without consuming the builder.
    pub fn codebase(&self) -> &Codebase {
        &self.cb
    }
}

impl Default for CodebaseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// One-shot: build a finalized [`Codebase`] from a set of per-file slices.
pub fn codebase_from_parts(parts: Vec<crate::storage::StubSlice>) -> Codebase {
    let mut b = CodebaseBuilder::new();
    for p in parts {
        b.add(p);
    }
    b.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arc(s: &str) -> Arc<str> {
        Arc::from(s)
    }

    #[test]
    fn method_referenced_at_groups_spans_by_file() {
        let cb = Codebase::new();
        cb.mark_method_referenced_at("Foo", "bar", arc("a.php"), 1, 0, 5);
        cb.mark_method_referenced_at("Foo", "bar", arc("a.php"), 1, 10, 15);
        cb.mark_method_referenced_at("Foo", "bar", arc("b.php"), 2, 0, 5);

        let locs = cb.get_reference_locations("Foo::bar");
        let files: std::collections::HashSet<&str> =
            locs.iter().map(|(f, ..)| f.as_ref()).collect();
        assert_eq!(files.len(), 2, "two files, not three spans");
        assert!(locs.contains(&(arc("a.php"), 1, 0, 5)));
        assert!(locs.contains(&(arc("a.php"), 1, 10, 15)));
        assert_eq!(
            locs.iter().filter(|(f, ..)| f.as_ref() == "a.php").count(),
            2
        );
        assert!(locs.contains(&(arc("b.php"), 2, 0, 5)));
        assert!(
            cb.is_method_referenced("Foo", "bar"),
            "DashSet also updated"
        );
    }

    #[test]
    fn duplicate_spans_are_deduplicated() {
        let cb = Codebase::new();
        // Same call site recorded twice (e.g. union receiver Foo|Foo)
        cb.mark_method_referenced_at("Foo", "bar", arc("a.php"), 1, 0, 5);
        cb.mark_method_referenced_at("Foo", "bar", arc("a.php"), 1, 0, 5);

        let count = cb
            .get_reference_locations("Foo::bar")
            .iter()
            .filter(|(f, ..)| f.as_ref() == "a.php")
            .count();
        assert_eq!(count, 1, "duplicate span deduplicated");
    }

    #[test]
    fn method_key_is_lowercased() {
        let cb = Codebase::new();
        cb.mark_method_referenced_at("Cls", "MyMethod", arc("f.php"), 1, 0, 3);
        assert!(!cb.get_reference_locations("Cls::mymethod").is_empty());
    }

    #[test]
    fn property_referenced_at_records_location() {
        let cb = Codebase::new();
        cb.mark_property_referenced_at("Bar", "count", arc("x.php"), 1, 5, 10);

        assert!(cb
            .get_reference_locations("Bar::count")
            .contains(&(arc("x.php"), 1, 5, 10)));
        assert!(cb.is_property_referenced("Bar", "count"));
    }

    #[test]
    fn function_referenced_at_records_location() {
        let cb = Codebase::new();
        cb.mark_function_referenced_at("my_fn", arc("a.php"), 1, 10, 15);

        assert!(cb
            .get_reference_locations("my_fn")
            .contains(&(arc("a.php"), 1, 10, 15)));
        assert!(cb.is_function_referenced("my_fn"));
    }

    #[test]
    fn class_referenced_at_records_location() {
        let cb = Codebase::new();
        cb.mark_class_referenced_at("Foo", arc("a.php"), 1, 5, 8);

        assert!(cb
            .get_reference_locations("Foo")
            .contains(&(arc("a.php"), 1, 5, 8)));
    }

    #[test]
    fn get_reference_locations_flattens_all_files() {
        let cb = Codebase::new();
        cb.mark_function_referenced_at("fn1", arc("a.php"), 1, 0, 5);
        cb.mark_function_referenced_at("fn1", arc("b.php"), 2, 0, 5);

        let mut locs = cb.get_reference_locations("fn1");
        locs.sort_by_key(|&(_, line, col, _)| (line, col));
        assert_eq!(locs.len(), 2);
        assert_eq!(locs[0], (arc("a.php"), 1, 0, 5));
        assert_eq!(locs[1], (arc("b.php"), 2, 0, 5));
    }

    #[test]
    fn replay_reference_locations_restores_index() {
        let cb = Codebase::new();
        let locs = vec![
            ("Foo::bar".to_string(), 1u32, 0u16, 5u16),
            ("Foo::bar".to_string(), 1, 10, 15),
            ("greet".to_string(), 2, 0, 5),
        ];
        cb.replay_reference_locations(arc("a.php"), &locs);

        let bar_locs = cb.get_reference_locations("Foo::bar");
        assert!(bar_locs.contains(&(arc("a.php"), 1, 0, 5)));
        assert!(bar_locs.contains(&(arc("a.php"), 1, 10, 15)));

        assert!(cb
            .get_reference_locations("greet")
            .contains(&(arc("a.php"), 2, 0, 5)));

        assert!(cb.file_has_symbol_references("a.php"));
    }

    #[test]
    fn remove_file_clears_its_spans_only() {
        let cb = Codebase::new();
        cb.mark_function_referenced_at("fn1", arc("a.php"), 1, 0, 5);
        cb.mark_function_referenced_at("fn1", arc("b.php"), 1, 10, 15);

        cb.remove_file_definitions("a.php");

        let locs = cb.get_reference_locations("fn1");
        assert!(
            !locs.iter().any(|(f, ..)| f.as_ref() == "a.php"),
            "a.php spans removed"
        );
        assert!(
            locs.contains(&(arc("b.php"), 1, 10, 15)),
            "b.php spans untouched"
        );
        assert!(!cb.file_has_symbol_references("a.php"));
    }

    #[test]
    fn remove_file_does_not_affect_other_files() {
        let cb = Codebase::new();
        cb.mark_property_referenced_at("Cls", "prop", arc("x.php"), 1, 1, 4);
        cb.mark_property_referenced_at("Cls", "prop", arc("y.php"), 1, 7, 10);

        cb.remove_file_definitions("x.php");

        let locs = cb.get_reference_locations("Cls::prop");
        assert!(!locs.iter().any(|(f, ..)| f.as_ref() == "x.php"));
        assert!(locs.contains(&(arc("y.php"), 1, 7, 10)));
    }

    #[test]
    fn remove_file_definitions_on_never_analyzed_file_is_noop() {
        let cb = Codebase::new();
        cb.mark_function_referenced_at("fn1", arc("a.php"), 1, 0, 5);

        // "ghost.php" was never analyzed — removing it must not panic or corrupt state.
        cb.remove_file_definitions("ghost.php");

        // Existing data must be untouched.
        assert!(cb
            .get_reference_locations("fn1")
            .contains(&(arc("a.php"), 1, 0, 5)));
        assert!(!cb.file_has_symbol_references("ghost.php"));
    }

    #[test]
    fn replay_reference_locations_with_empty_list_is_noop() {
        let cb = Codebase::new();
        cb.mark_function_referenced_at("fn1", arc("a.php"), 1, 0, 5);

        // Replaying an empty list must not touch existing entries.
        cb.replay_reference_locations(arc("b.php"), &[]);

        assert!(
            !cb.file_has_symbol_references("b.php"),
            "empty replay must not create a file entry"
        );
        assert!(
            cb.get_reference_locations("fn1")
                .contains(&(arc("a.php"), 1, 0, 5)),
            "existing spans untouched"
        );
    }

    #[test]
    fn replay_reference_locations_twice_does_not_duplicate_spans() {
        let cb = Codebase::new();
        let locs = vec![("fn1".to_string(), 1u32, 0u16, 5u16)];

        cb.replay_reference_locations(arc("a.php"), &locs);
        cb.replay_reference_locations(arc("a.php"), &locs);

        let count = cb
            .get_reference_locations("fn1")
            .iter()
            .filter(|(f, ..)| f.as_ref() == "a.php")
            .count();
        assert_eq!(
            count, 1,
            "replaying the same location twice must not create duplicate spans"
        );
    }

    // -----------------------------------------------------------------------
    // inject_stub_slice — correctness-critical tests
    // -----------------------------------------------------------------------

    fn make_fn(fqn: &str, short_name: &str) -> crate::storage::FunctionStorage {
        crate::storage::FunctionStorage {
            fqn: Arc::from(fqn),
            short_name: Arc::from(short_name),
            params: vec![],
            return_type: None,
            inferred_return_type: None,
            template_params: vec![],
            assertions: vec![],
            throws: vec![],
            deprecated: None,
            is_pure: false,
            location: None,
        }
    }

    #[test]
    fn inject_stub_slice_later_injection_overwrites_earlier() {
        let cb = Codebase::new();

        cb.inject_stub_slice(crate::storage::StubSlice {
            functions: vec![make_fn("strlen", "phpstorm_version")],
            file: Some(Arc::from("phpstorm/standard.php")),
            ..Default::default()
        });
        assert_eq!(
            cb.functions.get("strlen").unwrap().short_name.as_ref(),
            "phpstorm_version"
        );

        cb.inject_stub_slice(crate::storage::StubSlice {
            functions: vec![make_fn("strlen", "custom_version")],
            file: Some(Arc::from("stubs/standard/basic.php")),
            ..Default::default()
        });

        assert_eq!(
            cb.functions.get("strlen").unwrap().short_name.as_ref(),
            "custom_version",
            "custom stub must overwrite phpstorm stub"
        );
        assert_eq!(
            cb.symbol_to_file.get("strlen").unwrap().as_ref(),
            "stubs/standard/basic.php",
            "symbol_to_file must point to the overriding file"
        );
    }

    #[test]
    fn inject_stub_slice_constants_not_added_to_symbol_to_file() {
        let cb = Codebase::new();

        cb.inject_stub_slice(crate::storage::StubSlice {
            constants: vec![(Arc::from("PHP_EOL"), mir_types::Union::empty())],
            file: Some(Arc::from("stubs/core/constants.php")),
            ..Default::default()
        });

        assert!(
            cb.constants.contains_key("PHP_EOL"),
            "constant must be registered in constants map"
        );
        assert!(
            !cb.symbol_to_file.contains_key("PHP_EOL"),
            "constants must not appear in symbol_to_file — go-to-definition is not supported for them"
        );
    }

    #[test]
    fn remove_file_definitions_purges_injected_global_vars() {
        let cb = Codebase::new();

        cb.inject_stub_slice(crate::storage::StubSlice {
            global_vars: vec![(Arc::from("db_connection"), mir_types::Union::empty())],
            file: Some(Arc::from("src/bootstrap.php")),
            ..Default::default()
        });
        assert!(
            cb.global_vars.contains_key("db_connection"),
            "global var must be registered after injection"
        );

        cb.remove_file_definitions("src/bootstrap.php");

        assert!(
            !cb.global_vars.contains_key("db_connection"),
            "global var must be removed when its defining file is removed"
        );
    }

    #[test]
    fn inject_stub_slice_without_file_discards_global_vars() {
        let cb = Codebase::new();

        cb.inject_stub_slice(crate::storage::StubSlice {
            global_vars: vec![(Arc::from("orphan_var"), mir_types::Union::empty())],
            file: None,
            ..Default::default()
        });

        assert!(
            !cb.global_vars.contains_key("orphan_var"),
            "global_vars must not be registered when slice.file is None"
        );
    }

    // These three tests guard the StubSlice → file_namespaces / file_imports contract.
    //
    // Background: inject_stub_slice is the only write path used by both
    // collect() (the normal project-analysis path) and collect_slice +
    // inject_stub_slice (the salsa/LSP incremental path and re_analyze_file).
    // Prior to the fix, inject_stub_slice never wrote file_namespaces or
    // file_imports, so any consumer that skipped the separate project.rs AST
    // walk ended up with empty maps and produced false UndefinedClass
    // diagnostics for use-aliased classes.

    #[test]
    fn inject_stub_slice_populates_file_namespace() {
        // A slice with a namespace must cause file_namespaces to be populated
        // for that file so that StatementsAnalyzer can resolve unqualified names
        // against the correct namespace during Pass 2.
        let cb = Codebase::new();
        cb.inject_stub_slice(crate::storage::StubSlice {
            file: Some(Arc::from("src/Service.php")),
            namespace: Some(Arc::from("App\\Service")),
            ..Default::default()
        });
        assert_eq!(
            cb.file_namespaces
                .get("src/Service.php")
                .as_deref()
                .map(|s| s.as_str()),
            Some("App\\Service"),
            "file_namespaces must be populated when slice carries a namespace"
        );

        // file=Some but namespace=None must not create a spurious entry.
        let cb2 = Codebase::new();
        cb2.inject_stub_slice(crate::storage::StubSlice {
            file: Some(Arc::from("src/global.php")),
            namespace: None,
            ..Default::default()
        });
        assert!(
            cb2.file_namespaces.is_empty(),
            "file_namespaces must not be written when slice.namespace is None"
        );
    }

    #[test]
    fn inject_stub_slice_populates_file_imports() {
        // A slice with use-alias imports must cause file_imports to be
        // populated so that StatementsAnalyzer can resolve aliased short names
        // (e.g. `new Entity()` where `use App\Model\Entity` is in scope).
        let cb = Codebase::new();
        let mut imports = std::collections::HashMap::new();
        imports.insert("Entity".to_string(), "App\\Model\\Entity".to_string());
        imports.insert(
            "Repo".to_string(),
            "App\\Repository\\EntityRepo".to_string(),
        );
        cb.inject_stub_slice(crate::storage::StubSlice {
            file: Some(Arc::from("src/Handler.php")),
            imports,
            ..Default::default()
        });
        let stored = cb.file_imports.get("src/Handler.php").unwrap();
        assert_eq!(
            stored.get("Entity").map(|s| s.as_str()),
            Some("App\\Model\\Entity")
        );
        assert_eq!(
            stored.get("Repo").map(|s| s.as_str()),
            Some("App\\Repository\\EntityRepo")
        );

        // file=Some but empty imports must not create a spurious entry.
        let cb2 = Codebase::new();
        cb2.inject_stub_slice(crate::storage::StubSlice {
            file: Some(Arc::from("src/no_imports.php")),
            imports: std::collections::HashMap::new(),
            ..Default::default()
        });
        assert!(
            cb2.file_imports.is_empty(),
            "file_imports must not be written when slice.imports is empty"
        );
    }

    #[test]
    fn inject_stub_slice_skips_namespace_and_imports_when_no_file() {
        // Bundled stub slices (file = None) must never pollute file_namespaces
        // or file_imports — those maps are keyed by on-disk path and only make
        // sense for slices that represent a specific source file.
        let cb = Codebase::new();
        let mut imports = std::collections::HashMap::new();
        imports.insert("Foo".to_string(), "Bar\\Foo".to_string());
        cb.inject_stub_slice(crate::storage::StubSlice {
            file: None,
            namespace: Some(Arc::from("Bar")),
            imports,
            ..Default::default()
        });
        assert!(
            cb.file_namespaces.is_empty(),
            "file_namespaces must not be written when slice.file is None"
        );
        assert!(
            cb.file_imports.is_empty(),
            "file_imports must not be written when slice.file is None"
        );
    }

    #[test]
    fn remove_file_definitions_purges_file_namespaces_and_imports() {
        // remove_file_definitions and inject_stub_slice form a round-trip:
        // remove clears, inject refills. This test guards the remove half for
        // file_namespaces and file_imports — symmetric to
        // remove_file_definitions_purges_injected_global_vars which guards
        // the same round-trip for global_vars.
        let cb = Codebase::new();
        let mut imports = std::collections::HashMap::new();
        imports.insert("Entity".to_string(), "App\\Model\\Entity".to_string());
        cb.inject_stub_slice(crate::storage::StubSlice {
            file: Some(Arc::from("src/Handler.php")),
            namespace: Some(Arc::from("App\\Service")),
            imports,
            ..Default::default()
        });
        assert!(
            cb.file_namespaces.contains_key("src/Handler.php"),
            "setup: namespace must be present"
        );
        assert!(
            cb.file_imports.contains_key("src/Handler.php"),
            "setup: imports must be present"
        );

        cb.remove_file_definitions("src/Handler.php");

        assert!(
            !cb.file_namespaces.contains_key("src/Handler.php"),
            "file_namespaces entry must be removed when its defining file is removed"
        );
        assert!(
            !cb.file_imports.contains_key("src/Handler.php"),
            "file_imports entry must be removed when its defining file is removed"
        );
    }
}
