use std::sync::Arc;

use dashmap::{DashMap, DashSet};

use crate::interner::Interner;

/// Maps symbol ID → flat list of `(file_id, start_byte, end_byte)`.
///
/// Entries are appended during Pass 2. Duplicates (e.g. from union receivers like
/// `Foo|Foo->method()`) are filtered at insert time. IDs come from
/// `Codebase::symbol_interner` / `Codebase::file_interner`.
///
/// Compared with the previous `DashMap<u32, HashMap<u32, HashSet<(u32, u32)>>>`,
/// this eliminates two levels of hash-map overhead (a `HashMap` per symbol and a
/// `HashSet` per file). Each entry is now 12 bytes (`u32` × 3) with no per-entry
/// allocator overhead beyond the `Vec` backing store.
type ReferenceLocations = DashMap<u32, Vec<(u32, u32, u32)>>;

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

/// Append `(sym_id, file_id, start, end)` to the reference index, skipping
/// exact duplicates so union receivers like `Foo|Foo->method()` don't inflate
/// the span list.
///
/// Both maps are updated atomically under their respective DashMap shard locks.
#[inline]
fn record_ref(
    sym_locs: &ReferenceLocations,
    file_refs: &DashMap<u32, Vec<u32>>,
    sym_id: u32,
    file_id: u32,
    start: u32,
    end: u32,
) {
    {
        let mut entries = sym_locs.entry(sym_id).or_default();
        let span = (file_id, start, end);
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
    /// All spans sorted by `(sym_id, file_id, start, end)`, deduplicated.
    /// Each entry is 16 bytes; total size = `n_refs × 16` with no hash overhead.
    entries: Vec<(u32, u32, u32, u32)>,
    /// CSR offsets keyed by sym_id (length = max_sym_id + 2).
    sym_offsets: Vec<u32>,
    /// Indices into `entries` sorted by `(file_id, sym_id, start, end)`.
    /// Allows O(log n) file-keyed lookups without duplicating the payload.
    by_file: Vec<u32>,
    /// CSR offsets keyed by file_id into `by_file` (length = max_file_id + 2).
    file_offsets: Vec<u32>,
}

// ---------------------------------------------------------------------------
// StructuralSnapshot — inheritance data captured before file removal
// ---------------------------------------------------------------------------

struct ClassInheritance {
    parent: Option<Arc<str>>,
    interfaces: Vec<Arc<str>>, // sorted for order-insensitive comparison
    traits: Vec<Arc<str>>,     // sorted
    all_parents: Vec<Arc<str>>,
}

struct InterfaceInheritance {
    extends: Vec<Arc<str>>, // sorted
    all_parents: Vec<Arc<str>>,
}

/// Snapshot of the inheritance structure of all symbols defined in a file.
///
/// Produced by [`Codebase::file_structural_snapshot`] before
/// [`Codebase::remove_file_definitions`], and consumed by
/// [`Codebase::structural_unchanged_after_pass1`] /
/// [`Codebase::restore_all_parents`] to skip an expensive `finalize()` call
/// when only method bodies (not class hierarchies) changed.
pub struct StructuralSnapshot {
    classes: std::collections::HashMap<Arc<str>, ClassInheritance>,
    interfaces: std::collections::HashMap<Arc<str>, InterfaceInheritance>,
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

    /// Maps symbol ID → { file ID → {(start_byte, end_byte)} }.
    /// IDs come from `symbol_interner` / `file_interner`.
    /// The inner HashMap groups spans by file for O(1) per-file cleanup.
    /// HashSet deduplicates spans from union receivers (e.g. Foo|Foo->method()).
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
        let mut entries: Vec<(u32, u32, u32, u32)> = self
            .symbol_reference_locations
            .iter()
            .flat_map(|entry| {
                let sym_id = *entry.key();
                entry
                    .value()
                    .iter()
                    .map(move |&(file_id, start, end)| (sym_id, file_id, start, end))
                    .collect::<Vec<_>>()
            })
            .collect();

        if entries.is_empty() {
            return;
        }

        // Sort by (sym_id, file_id, start, end) and drop exact duplicates.
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
        // `(file_id, sym_id, start, end)` so CSR offsets can be computed cheaply.
        let max_file = entries.iter().map(|&(_, f, ..)| f).max().unwrap_or(0) as usize;
        let mut by_file: Vec<u32> = (0..n as u32).collect();
        by_file.sort_unstable_by_key(|&i| {
            let (sym_id, file_id, start, end) = entries[i as usize];
            (file_id, sym_id, start, end)
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
            for &(sym_id, file_id, start, end) in &ci.entries {
                record_ref(
                    &self.symbol_reference_locations,
                    &self.file_symbol_references,
                    sym_id,
                    file_id,
                    start,
                    end,
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

        // Remove each symbol from its respective map and from symbol_to_file
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
                        entries.retain(|&(fid, _, _)| fid != file_id);
                    }
                }
            }
        }

        self.invalidate_finalization();
    }

    // -----------------------------------------------------------------------
    // Structural snapshot — skip finalize() on body-only changes
    // -----------------------------------------------------------------------

    /// Capture the inheritance structure of all symbols defined in `file_path`.
    ///
    /// Call this *before* `remove_file_definitions` to preserve the data that
    /// `finalize()` would otherwise have to recompute.  The snapshot records, for
    /// each class/interface in the file, the fields that feed into
    /// `all_parents` (parent class, implemented interfaces, used traits, extended
    /// interfaces) as well as the already-computed `all_parents` list itself.
    pub fn file_structural_snapshot(&self, file_path: &str) -> StructuralSnapshot {
        let symbols: Vec<Arc<str>> = self
            .symbol_to_file
            .iter()
            .filter(|e| e.value().as_ref() == file_path)
            .map(|e| e.key().clone())
            .collect();

        let mut classes = std::collections::HashMap::new();
        let mut interfaces = std::collections::HashMap::new();

        for sym in symbols {
            if let Some(cls) = self.classes.get(sym.as_ref()) {
                let mut ifaces = cls.interfaces.clone();
                ifaces.sort_unstable_by(|a, b| a.as_ref().cmp(b.as_ref()));
                let mut traits = cls.traits.clone();
                traits.sort_unstable_by(|a, b| a.as_ref().cmp(b.as_ref()));
                classes.insert(
                    sym,
                    ClassInheritance {
                        parent: cls.parent.clone(),
                        interfaces: ifaces,
                        traits,
                        all_parents: cls.all_parents.clone(),
                    },
                );
            } else if let Some(iface) = self.interfaces.get(sym.as_ref()) {
                let mut extends = iface.extends.clone();
                extends.sort_unstable_by(|a, b| a.as_ref().cmp(b.as_ref()));
                interfaces.insert(
                    sym,
                    InterfaceInheritance {
                        extends,
                        all_parents: iface.all_parents.clone(),
                    },
                );
            }
        }

        StructuralSnapshot {
            classes,
            interfaces,
        }
    }

    /// After Pass 1 completes, check whether the inheritance structure in
    /// `file_path` matches the snapshot taken before `remove_file_definitions`.
    ///
    /// Returns `true` if `finalize()` can be skipped — i.e. only method bodies,
    /// properties, or annotations changed, not any class/interface hierarchy.
    pub fn structural_unchanged_after_pass1(
        &self,
        file_path: &str,
        old: &StructuralSnapshot,
    ) -> bool {
        let symbols: Vec<Arc<str>> = self
            .symbol_to_file
            .iter()
            .filter(|e| e.value().as_ref() == file_path)
            .map(|e| e.key().clone())
            .collect();

        let mut seen_classes = 0usize;
        let mut seen_interfaces = 0usize;

        for sym in &symbols {
            if let Some(cls) = self.classes.get(sym.as_ref()) {
                seen_classes += 1;
                let Some(old_cls) = old.classes.get(sym.as_ref()) else {
                    return false; // new class added
                };
                if old_cls.parent != cls.parent {
                    return false;
                }
                let mut new_ifaces = cls.interfaces.clone();
                new_ifaces.sort_unstable_by(|a, b| a.as_ref().cmp(b.as_ref()));
                if old_cls.interfaces != new_ifaces {
                    return false;
                }
                let mut new_traits = cls.traits.clone();
                new_traits.sort_unstable_by(|a, b| a.as_ref().cmp(b.as_ref()));
                if old_cls.traits != new_traits {
                    return false;
                }
            } else if let Some(iface) = self.interfaces.get(sym.as_ref()) {
                seen_interfaces += 1;
                let Some(old_iface) = old.interfaces.get(sym.as_ref()) else {
                    return false; // new interface added
                };
                let mut new_extends = iface.extends.clone();
                new_extends.sort_unstable_by(|a, b| a.as_ref().cmp(b.as_ref()));
                if old_iface.extends != new_extends {
                    return false;
                }
            }
            // Traits, enums, functions, constants: not finalization-relevant, skip.
        }

        // Check for removed classes or interfaces.
        seen_classes == old.classes.len() && seen_interfaces == old.interfaces.len()
    }

    /// Restore `all_parents` from a snapshot and mark the codebase as finalized.
    ///
    /// Call this instead of `finalize()` when `structural_unchanged_after_pass1`
    /// returns `true`.  The newly re-registered symbols (written by Pass 1) have
    /// `all_parents = []`; this method repopulates them from the snapshot so that
    /// all downstream lookups that depend on `all_parents` keep working correctly.
    pub fn restore_all_parents(&self, file_path: &str, snapshot: &StructuralSnapshot) {
        let symbols: Vec<Arc<str>> = self
            .symbol_to_file
            .iter()
            .filter(|e| e.value().as_ref() == file_path)
            .map(|e| e.key().clone())
            .collect();

        for sym in &symbols {
            if let Some(old_cls) = snapshot.classes.get(sym.as_ref()) {
                if let Some(mut cls) = self.classes.get_mut(sym.as_ref()) {
                    cls.all_parents = old_cls.all_parents.clone();
                }
            } else if let Some(old_iface) = snapshot.interfaces.get(sym.as_ref()) {
                if let Some(mut iface) = self.interfaces.get_mut(sym.as_ref()) {
                    iface.all_parents = old_iface.all_parents.clone();
                }
            }
        }

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

    /// Resolve a property, walking up the inheritance chain (parent classes and traits).
    pub fn get_property(
        &self,
        fqcn: &str,
        prop_name: &str,
    ) -> Option<crate::storage::PropertyStorage> {
        // Check direct class own_properties
        if let Some(cls) = self.classes.get(fqcn) {
            if let Some(p) = cls.own_properties.get(prop_name) {
                return Some(p.clone());
            }
            let mixins = cls.mixins.clone();
            drop(cls);
            for mixin in &mixins {
                if let Some(p) = self.get_property(mixin.as_ref(), prop_name) {
                    return Some(p);
                }
            }
        }

        // Walk all ancestors (collected during finalize)
        let all_parents = {
            if let Some(cls) = self.classes.get(fqcn) {
                cls.all_parents.clone()
            } else {
                return None;
            }
        };

        for ancestor_fqcn in &all_parents {
            if let Some(ancestor_cls) = self.classes.get(ancestor_fqcn.as_ref()) {
                if let Some(p) = ancestor_cls.own_properties.get(prop_name) {
                    return Some(p.clone());
                }
            }
        }

        // Check traits
        let trait_list = {
            if let Some(cls) = self.classes.get(fqcn) {
                cls.traits.clone()
            } else {
                vec![]
            }
        };
        for trait_fqcn in &trait_list {
            if let Some(tr) = self.traits.get(trait_fqcn.as_ref()) {
                if let Some(p) = tr.own_properties.get(prop_name) {
                    return Some(p.clone());
                }
            }
        }

        None
    }

    /// Resolve a class constant by name, walking up the inheritance chain.
    pub fn get_class_constant(
        &self,
        fqcn: &str,
        const_name: &str,
    ) -> Option<crate::storage::ConstantStorage> {
        // Class: own → traits → ancestors → interfaces
        if let Some(cls) = self.classes.get(fqcn) {
            if let Some(c) = cls.own_constants.get(const_name) {
                return Some(c.clone());
            }
            let all_parents = cls.all_parents.clone();
            let interfaces = cls.interfaces.clone();
            let traits = cls.traits.clone();
            drop(cls);

            for tr_fqcn in &traits {
                if let Some(tr) = self.traits.get(tr_fqcn.as_ref()) {
                    if let Some(c) = tr.own_constants.get(const_name) {
                        return Some(c.clone());
                    }
                }
            }

            for ancestor_fqcn in &all_parents {
                if let Some(ancestor) = self.classes.get(ancestor_fqcn.as_ref()) {
                    if let Some(c) = ancestor.own_constants.get(const_name) {
                        return Some(c.clone());
                    }
                }
                if let Some(iface) = self.interfaces.get(ancestor_fqcn.as_ref()) {
                    if let Some(c) = iface.own_constants.get(const_name) {
                        return Some(c.clone());
                    }
                }
            }

            for iface_fqcn in &interfaces {
                if let Some(iface) = self.interfaces.get(iface_fqcn.as_ref()) {
                    if let Some(c) = iface.own_constants.get(const_name) {
                        return Some(c.clone());
                    }
                }
            }

            return None;
        }

        // Interface: own → parent interfaces
        if let Some(iface) = self.interfaces.get(fqcn) {
            if let Some(c) = iface.own_constants.get(const_name) {
                return Some(c.clone());
            }
            let parents = iface.all_parents.clone();
            drop(iface);
            for p in &parents {
                if let Some(parent_iface) = self.interfaces.get(p.as_ref()) {
                    if let Some(c) = parent_iface.own_constants.get(const_name) {
                        return Some(c.clone());
                    }
                }
            }
            return None;
        }

        // Enum: own constants + cases
        if let Some(en) = self.enums.get(fqcn) {
            if let Some(c) = en.own_constants.get(const_name) {
                return Some(c.clone());
            }
            if en.cases.contains_key(const_name) {
                return Some(crate::storage::ConstantStorage {
                    name: Arc::from(const_name),
                    ty: mir_types::Union::mixed(),
                    visibility: None,
                    location: None,
                });
            }
            return None;
        }

        // Trait: own constants only
        if let Some(tr) = self.traits.get(fqcn) {
            if let Some(c) = tr.own_constants.get(const_name) {
                return Some(c.clone());
            }
            return None;
        }

        None
    }

    /// Resolve a method, walking up the full inheritance chain (own → traits → ancestors).
    pub fn get_method(&self, fqcn: &str, method_name: &str) -> Option<Arc<MethodStorage>> {
        // PHP method names are case-insensitive — normalize to lowercase for all lookups.
        let method_lower = method_name.to_lowercase();
        let method_name = method_lower.as_str();

        // --- Class: own methods → own traits → ancestor classes/traits/interfaces ---
        if let Some(cls) = self.classes.get(fqcn) {
            // 1. Own methods (highest priority)
            if let Some(m) = lookup_method(&cls.own_methods, method_name) {
                return Some(Arc::clone(m));
            }
            // Collect chain info before dropping the DashMap guard.
            let own_traits = cls.traits.clone();
            let ancestors = cls.all_parents.clone();
            let mixins = cls.mixins.clone();
            drop(cls);

            // 2. Docblock mixins (delegated magic lookup)
            for mixin_fqcn in &mixins {
                if let Some(m) = self.get_method(mixin_fqcn, method_name) {
                    return Some(m);
                }
            }

            // 3. Own trait methods (recursive into trait-of-trait)
            for tr_fqcn in &own_traits {
                if let Some(m) = self.get_method_in_trait(tr_fqcn, method_name) {
                    return Some(m);
                }
            }

            // 4. Ancestor chain (all_parents is closest-first: parent, grandparent, …)
            for ancestor_fqcn in &ancestors {
                if let Some(anc) = self.classes.get(ancestor_fqcn.as_ref()) {
                    if let Some(m) = lookup_method(&anc.own_methods, method_name) {
                        return Some(Arc::clone(m));
                    }
                    let anc_traits = anc.traits.clone();
                    drop(anc);
                    for tr_fqcn in &anc_traits {
                        if let Some(m) = self.get_method_in_trait(tr_fqcn, method_name) {
                            return Some(m);
                        }
                    }
                } else if let Some(iface) = self.interfaces.get(ancestor_fqcn.as_ref()) {
                    if let Some(m) = lookup_method(&iface.own_methods, method_name) {
                        let mut ms = (**m).clone();
                        ms.is_abstract = true;
                        return Some(Arc::new(ms));
                    }
                }
                // Traits listed in all_parents are already covered via their owning class above.
            }
            return None;
        }

        // --- Interface: own methods + parent interfaces ---
        if let Some(iface) = self.interfaces.get(fqcn) {
            if let Some(m) = lookup_method(&iface.own_methods, method_name) {
                return Some(Arc::clone(m));
            }
            let parents = iface.all_parents.clone();
            drop(iface);
            for parent_fqcn in &parents {
                if let Some(parent_iface) = self.interfaces.get(parent_fqcn.as_ref()) {
                    if let Some(m) = lookup_method(&parent_iface.own_methods, method_name) {
                        return Some(Arc::clone(m));
                    }
                }
            }
            return None;
        }

        // --- Trait (variable annotated with a trait type) ---
        if let Some(tr) = self.traits.get(fqcn) {
            if let Some(m) = lookup_method(&tr.own_methods, method_name) {
                return Some(Arc::clone(m));
            }
            return None;
        }

        // --- Enum ---
        if let Some(e) = self.enums.get(fqcn) {
            if let Some(m) = lookup_method(&e.own_methods, method_name) {
                return Some(Arc::clone(m));
            }
            // PHP 8.1 built-in enum methods: cases(), from(), tryFrom()
            if matches!(method_name, "cases" | "from" | "tryfrom") {
                return Some(Arc::new(crate::storage::MethodStorage {
                    fqcn: Arc::from(fqcn),
                    name: Arc::from(method_name),
                    params: vec![],
                    return_type: Some(mir_types::Union::mixed()),
                    inferred_return_type: None,
                    visibility: crate::storage::Visibility::Public,
                    is_static: true,
                    is_abstract: false,
                    is_constructor: false,
                    template_params: vec![],
                    assertions: vec![],
                    throws: vec![],
                    is_final: false,
                    is_internal: false,
                    is_pure: false,
                    deprecated: None,
                    location: None,
                }));
            }
        }

        None
    }

    /// Returns true if `child` extends or implements `ancestor` (transitively).
    pub fn extends_or_implements(&self, child: &str, ancestor: &str) -> bool {
        if child == ancestor {
            return true;
        }
        if let Some(cls) = self.classes.get(child) {
            return cls.implements_or_extends(ancestor);
        }
        if let Some(iface) = self.interfaces.get(child) {
            return iface.all_parents.iter().any(|p| p.as_ref() == ancestor);
        }
        // Enum: backed enums implicitly implement BackedEnum (and UnitEnum);
        // pure enums implicitly implement UnitEnum.
        if let Some(en) = self.enums.get(child) {
            // Check explicitly declared interfaces (e.g. implements SomeInterface)
            if en.interfaces.iter().any(|i| i.as_ref() == ancestor) {
                return true;
            }
            // PHP built-in: every enum implements UnitEnum
            if ancestor == "UnitEnum" || ancestor == "\\UnitEnum" {
                return true;
            }
            // Backed enums implement BackedEnum
            if (ancestor == "BackedEnum" || ancestor == "\\BackedEnum") && en.scalar_type.is_some()
            {
                return true;
            }
        }
        false
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
        self.get_method(fqcn, "__get").is_some()
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
        // Check methods
        if let Some(method) = self.get_method(fqcn, member_name) {
            return method.location.clone();
        }
        // Check properties
        if let Some(prop) = self.get_property(fqcn, member_name) {
            return prop.location.clone();
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
        start: u32,
        end: u32,
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
            start,
            end,
        );
    }

    /// Record a property reference with its source location.
    /// Also updates the referenced_properties DashSet for dead-code detection.
    pub fn mark_property_referenced_at(
        &self,
        fqcn: &str,
        prop_name: &str,
        file: Arc<str>,
        start: u32,
        end: u32,
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
            start,
            end,
        );
    }

    /// Record a function reference with its source location.
    /// Also updates the referenced_functions DashSet for dead-code detection.
    pub fn mark_function_referenced_at(&self, fqn: &str, file: Arc<str>, start: u32, end: u32) {
        self.ensure_expanded();
        let sym_id = self.symbol_interner.intern_str(fqn);
        let file_id = self.file_interner.intern(file);
        self.referenced_functions.insert(sym_id);
        record_ref(
            &self.symbol_reference_locations,
            &self.file_symbol_references,
            sym_id,
            file_id,
            start,
            end,
        );
    }

    /// Record a class reference (e.g. `new Foo()`) with its source location.
    /// Does not update any dead-code DashSet — class instantiation tracking is
    /// separate from method/property/function dead-code detection.
    pub fn mark_class_referenced_at(&self, fqcn: &str, file: Arc<str>, start: u32, end: u32) {
        self.ensure_expanded();
        let sym_id = self.symbol_interner.intern_str(fqcn);
        let file_id = self.file_interner.intern(file);
        record_ref(
            &self.symbol_reference_locations,
            &self.file_symbol_references,
            sym_id,
            file_id,
            start,
            end,
        );
    }

    /// Replay cached reference locations for a file into the reference index.
    /// Called on cache hits to avoid re-running Pass 2 just to rebuild the index.
    /// `locs` is a slice of `(symbol_key, start_byte, end_byte)` as stored in the cache.
    pub fn replay_reference_locations(&self, file: Arc<str>, locs: &[(String, u32, u32)]) {
        if locs.is_empty() {
            return;
        }
        self.ensure_expanded();
        let file_id = self.file_interner.intern(file);
        for (symbol_key, start, end) in locs {
            let sym_id = self.symbol_interner.intern_str(symbol_key);
            record_ref(
                &self.symbol_reference_locations,
                &self.file_symbol_references,
                sym_id,
                file_id,
                *start,
                *end,
            );
        }
    }

    /// Return all reference locations for `symbol` as a flat `Vec<(file, start, end)>`.
    /// Returns an empty Vec if the symbol has no recorded references.
    pub fn get_reference_locations(&self, symbol: &str) -> Vec<(Arc<str>, u32, u32)> {
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
                .map(|&(_, file_id, s, e)| (self.file_interner.get(file_id), s, e))
                .collect();
        }
        // Slow path: build-phase DashMap.
        let Some(entries) = self.symbol_reference_locations.get(&sym_id) else {
            return Vec::new();
        };
        entries
            .iter()
            .map(|&(file_id, start, end)| (self.file_interner.get(file_id), start, end))
            .collect()
    }

    /// Extract all reference locations recorded for `file` as `(symbol_key, start, end)` triples.
    /// Used by the cache layer to persist per-file reference data between runs.
    pub fn extract_file_reference_locations(&self, file: &str) -> Vec<(Arc<str>, u32, u32)> {
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
                    let (sym_id, _, s, e) = ci.entries[entry_idx as usize];
                    (self.symbol_interner.get(sym_id), s, e)
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
            for &(entry_file_id, start, end) in entries.iter() {
                if entry_file_id == file_id {
                    out.push((sym_key.clone(), start, end));
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
    /// Resolves inheritance chains and builds method dispatch tables.
    pub fn finalize(&self) {
        if self.finalized.load(std::sync::atomic::Ordering::SeqCst) {
            return;
        }

        // 1. Resolve all_parents for classes
        let class_keys: Vec<Arc<str>> = self.classes.iter().map(|e| e.key().clone()).collect();
        for fqcn in &class_keys {
            let parents = self.collect_class_ancestors(fqcn);
            if let Some(mut cls) = self.classes.get_mut(fqcn.as_ref()) {
                cls.all_parents = parents;
            }
        }

        // 2. Resolve all_parents for interfaces
        let iface_keys: Vec<Arc<str>> = self.interfaces.iter().map(|e| e.key().clone()).collect();
        for fqcn in &iface_keys {
            let parents = self.collect_interface_ancestors(fqcn);
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

    /// Look up `method_name` in a trait's own methods, then recursively in any
    /// traits that the trait itself uses (`use OtherTrait;` inside a trait body).
    /// A visited set prevents infinite loops on pathological mutual trait use.
    fn get_method_in_trait(
        &self,
        tr_fqcn: &Arc<str>,
        method_name: &str,
    ) -> Option<Arc<MethodStorage>> {
        let mut visited = std::collections::HashSet::new();
        self.get_method_in_trait_inner(tr_fqcn, method_name, &mut visited)
    }

    fn get_method_in_trait_inner(
        &self,
        tr_fqcn: &Arc<str>,
        method_name: &str,
        visited: &mut std::collections::HashSet<String>,
    ) -> Option<Arc<MethodStorage>> {
        if !visited.insert(tr_fqcn.to_string()) {
            return None; // cycle guard
        }
        let tr = self.traits.get(tr_fqcn.as_ref())?;
        if let Some(m) = lookup_method(&tr.own_methods, method_name) {
            return Some(Arc::clone(m));
        }
        let used_traits = tr.traits.clone();
        drop(tr);
        for used_fqcn in &used_traits {
            if let Some(m) = self.get_method_in_trait_inner(used_fqcn, method_name, visited) {
                return Some(m);
            }
        }
        None
    }

    fn collect_class_ancestors(&self, fqcn: &str) -> Vec<Arc<str>> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.collect_class_ancestors_inner(fqcn, &mut result, &mut visited);
        result
    }

    fn collect_class_ancestors_inner(
        &self,
        fqcn: &str,
        out: &mut Vec<Arc<str>>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        if !visited.insert(fqcn.to_string()) {
            return; // cycle guard
        }
        let (parent, interfaces, traits) = {
            if let Some(cls) = self.classes.get(fqcn) {
                (
                    cls.parent.clone(),
                    cls.interfaces.clone(),
                    cls.traits.clone(),
                )
            } else {
                return;
            }
        };

        if let Some(p) = parent {
            out.push(p.clone());
            self.collect_class_ancestors_inner(&p, out, visited);
        }
        for iface in interfaces {
            out.push(iface.clone());
            self.collect_interface_ancestors_inner(&iface, out, visited);
        }
        for t in traits {
            out.push(t);
        }
    }

    fn collect_interface_ancestors(&self, fqcn: &str) -> Vec<Arc<str>> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.collect_interface_ancestors_inner(fqcn, &mut result, &mut visited);
        result
    }

    fn collect_interface_ancestors_inner(
        &self,
        fqcn: &str,
        out: &mut Vec<Arc<str>>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        if !visited.insert(fqcn.to_string()) {
            return;
        }
        let extends = {
            if let Some(iface) = self.interfaces.get(fqcn) {
                iface.extends.clone()
            } else {
                return;
            }
        };
        for e in extends {
            out.push(e.clone());
            self.collect_interface_ancestors_inner(&e, out, visited);
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
        cb.mark_method_referenced_at("Foo", "bar", arc("a.php"), 0, 5);
        cb.mark_method_referenced_at("Foo", "bar", arc("a.php"), 10, 15);
        cb.mark_method_referenced_at("Foo", "bar", arc("b.php"), 20, 25);

        let locs = cb.get_reference_locations("Foo::bar");
        let files: std::collections::HashSet<&str> =
            locs.iter().map(|(f, _, _)| f.as_ref()).collect();
        assert_eq!(files.len(), 2, "two files, not three spans");
        assert!(locs.contains(&(arc("a.php"), 0, 5)));
        assert!(locs.contains(&(arc("a.php"), 10, 15)));
        assert_eq!(
            locs.iter()
                .filter(|(f, _, _)| f.as_ref() == "a.php")
                .count(),
            2
        );
        assert!(locs.contains(&(arc("b.php"), 20, 25)));
        assert!(
            cb.is_method_referenced("Foo", "bar"),
            "DashSet also updated"
        );
    }

    #[test]
    fn duplicate_spans_are_deduplicated() {
        let cb = Codebase::new();
        // Same call site recorded twice (e.g. union receiver Foo|Foo)
        cb.mark_method_referenced_at("Foo", "bar", arc("a.php"), 0, 5);
        cb.mark_method_referenced_at("Foo", "bar", arc("a.php"), 0, 5);

        let count = cb
            .get_reference_locations("Foo::bar")
            .iter()
            .filter(|(f, _, _)| f.as_ref() == "a.php")
            .count();
        assert_eq!(count, 1, "duplicate span deduplicated");
    }

    #[test]
    fn method_key_is_lowercased() {
        let cb = Codebase::new();
        cb.mark_method_referenced_at("Cls", "MyMethod", arc("f.php"), 0, 3);
        assert!(!cb.get_reference_locations("Cls::mymethod").is_empty());
    }

    #[test]
    fn property_referenced_at_records_location() {
        let cb = Codebase::new();
        cb.mark_property_referenced_at("Bar", "count", arc("x.php"), 5, 10);

        assert!(cb
            .get_reference_locations("Bar::count")
            .contains(&(arc("x.php"), 5, 10)));
        assert!(cb.is_property_referenced("Bar", "count"));
    }

    #[test]
    fn function_referenced_at_records_location() {
        let cb = Codebase::new();
        cb.mark_function_referenced_at("my_fn", arc("a.php"), 10, 15);

        assert!(cb
            .get_reference_locations("my_fn")
            .contains(&(arc("a.php"), 10, 15)));
        assert!(cb.is_function_referenced("my_fn"));
    }

    #[test]
    fn class_referenced_at_records_location() {
        let cb = Codebase::new();
        cb.mark_class_referenced_at("Foo", arc("a.php"), 5, 8);

        assert!(cb
            .get_reference_locations("Foo")
            .contains(&(arc("a.php"), 5, 8)));
    }

    #[test]
    fn get_reference_locations_flattens_all_files() {
        let cb = Codebase::new();
        cb.mark_function_referenced_at("fn1", arc("a.php"), 0, 5);
        cb.mark_function_referenced_at("fn1", arc("b.php"), 10, 15);

        let mut locs = cb.get_reference_locations("fn1");
        locs.sort_by_key(|(_, s, _)| *s);
        assert_eq!(locs.len(), 2);
        assert_eq!(locs[0], (arc("a.php"), 0, 5));
        assert_eq!(locs[1], (arc("b.php"), 10, 15));
    }

    #[test]
    fn replay_reference_locations_restores_index() {
        let cb = Codebase::new();
        let locs = vec![
            ("Foo::bar".to_string(), 0u32, 5u32),
            ("Foo::bar".to_string(), 10, 15),
            ("greet".to_string(), 20, 25),
        ];
        cb.replay_reference_locations(arc("a.php"), &locs);

        let bar_locs = cb.get_reference_locations("Foo::bar");
        assert!(bar_locs.contains(&(arc("a.php"), 0, 5)));
        assert!(bar_locs.contains(&(arc("a.php"), 10, 15)));

        assert!(cb
            .get_reference_locations("greet")
            .contains(&(arc("a.php"), 20, 25)));

        assert!(cb.file_has_symbol_references("a.php"));
    }

    #[test]
    fn remove_file_clears_its_spans_only() {
        let cb = Codebase::new();
        cb.mark_function_referenced_at("fn1", arc("a.php"), 0, 5);
        cb.mark_function_referenced_at("fn1", arc("b.php"), 10, 15);

        cb.remove_file_definitions("a.php");

        let locs = cb.get_reference_locations("fn1");
        assert!(
            !locs.iter().any(|(f, _, _)| f.as_ref() == "a.php"),
            "a.php spans removed"
        );
        assert!(
            locs.contains(&(arc("b.php"), 10, 15)),
            "b.php spans untouched"
        );
        assert!(!cb.file_has_symbol_references("a.php"));
    }

    #[test]
    fn remove_file_does_not_affect_other_files() {
        let cb = Codebase::new();
        cb.mark_property_referenced_at("Cls", "prop", arc("x.php"), 1, 4);
        cb.mark_property_referenced_at("Cls", "prop", arc("y.php"), 7, 10);

        cb.remove_file_definitions("x.php");

        let locs = cb.get_reference_locations("Cls::prop");
        assert!(!locs.iter().any(|(f, _, _)| f.as_ref() == "x.php"));
        assert!(locs.contains(&(arc("y.php"), 7, 10)));
    }

    #[test]
    fn remove_file_definitions_on_never_analyzed_file_is_noop() {
        let cb = Codebase::new();
        cb.mark_function_referenced_at("fn1", arc("a.php"), 0, 5);

        // "ghost.php" was never analyzed — removing it must not panic or corrupt state.
        cb.remove_file_definitions("ghost.php");

        // Existing data must be untouched.
        assert!(cb
            .get_reference_locations("fn1")
            .contains(&(arc("a.php"), 0, 5)));
        assert!(!cb.file_has_symbol_references("ghost.php"));
    }

    #[test]
    fn replay_reference_locations_with_empty_list_is_noop() {
        let cb = Codebase::new();
        cb.mark_function_referenced_at("fn1", arc("a.php"), 0, 5);

        // Replaying an empty list must not touch existing entries.
        cb.replay_reference_locations(arc("b.php"), &[]);

        assert!(
            !cb.file_has_symbol_references("b.php"),
            "empty replay must not create a file entry"
        );
        assert!(
            cb.get_reference_locations("fn1")
                .contains(&(arc("a.php"), 0, 5)),
            "existing spans untouched"
        );
    }

    #[test]
    fn replay_reference_locations_twice_does_not_duplicate_spans() {
        let cb = Codebase::new();
        let locs = vec![("fn1".to_string(), 0u32, 5u32)];

        cb.replay_reference_locations(arc("a.php"), &locs);
        cb.replay_reference_locations(arc("a.php"), &locs);

        let count = cb
            .get_reference_locations("fn1")
            .iter()
            .filter(|(f, _, _)| f.as_ref() == "a.php")
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
}
