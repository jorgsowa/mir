use super::*;

impl AnalysisSession {
    /// Resolve a top-level symbol (class or function) to its declaration
    /// location. Powers go-to-definition.
    ///
    /// **Side effects:** if the symbol isn't yet known, this may invoke the
    /// configured [`crate::SourceProvider`] to fault in additional files and
    /// mutate the salsa input set. Use [`Self::definition_of_cached`] for a
    /// pure variant that only consults already-loaded state.
    ///
    /// Returns:
    /// - `Ok(Location)` — symbol found with a source location
    /// - `Err(NotFound)` — no such symbol in the codebase
    /// - `Err(NoSourceLocation)` — symbol exists but has no recorded span
    ///   (e.g. some stub-only declarations)
    pub fn definition_of(
        &self,
        symbol: &crate::Name,
    ) -> Result<mir_types::Location, crate::SymbolLookupError> {
        // Trigger any necessary lazy-load mutations before snapshotting.
        match symbol {
            crate::Name::Class(fqcn) => {
                let _ = self.load_class(fqcn.as_ref());
            }
            crate::Name::Function(fqn) => {
                let _ = self.load_class(fqn.as_ref());
            }
            crate::Name::Method { class, .. }
            | crate::Name::Property { class, .. }
            | crate::Name::ClassConstant { class, .. } => {
                let _ = self.load_class(class.as_ref());
            }
            _ => {}
        }
        self.definition_of_cached(symbol)
    }

    /// Pure variant of [`Self::definition_of`]. Never invokes the
    /// [`crate::SourceProvider`] and never mutates salsa inputs; resolves
    /// only against state already loaded by `set_file_text` / `ingest_file`.
    /// Returns `Err(NotFound)` when the symbol isn't in the loaded set, even
    /// if a resolver could in principle map it.
    pub fn definition_of_cached(
        &self,
        symbol: &crate::Name,
    ) -> Result<mir_types::Location, crate::SymbolLookupError> {
        let db = self.snapshot_db();
        match symbol {
            crate::Name::Class(fqcn) => {
                let here = crate::db::Fqcn::from_str(&db, fqcn.as_ref());
                let class = crate::db::find_class_like(&db, here)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                class
                    .location()
                    .cloned()
                    .ok_or(crate::SymbolLookupError::NoSourceLocation)
            }
            crate::Name::Function(fqn) => {
                let here = crate::db::Fqcn::from_str(&db, fqn.as_ref());
                let f = crate::db::find_function(&db, here)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                f.location
                    .clone()
                    .ok_or(crate::SymbolLookupError::NoSourceLocation)
            }
            crate::Name::Method { class, name }
            | crate::Name::Property { class, name }
            | crate::Name::ClassConstant { class, name } => {
                crate::db::member_location(&db, class, name)
                    .ok_or(crate::SymbolLookupError::NotFound)
            }
            crate::Name::GlobalConstant(_) => Err(crate::SymbolLookupError::NoSourceLocation),
        }
    }

    /// Hover information for a symbol: type, docstring, and definition location.
    ///
    /// Use [`crate::FileAnalysis::symbol_at`] to find the symbol at a cursor
    /// position, then build a [`crate::Name`] from its `kind`. This method
    /// assembles the displayable hover data.
    ///
    /// **Side effects:** when `symbol`'s owning class isn't yet loaded, this
    /// may invoke the configured [`crate::SourceProvider`] to fault in
    /// dependencies. Use [`Self::hover_cached`] for a pure variant.
    ///
    /// Returns `Err(NotFound)` if the symbol doesn't exist. May still return
    /// `Ok` with `docstring: None` or `definition: None` if those specific
    /// pieces aren't available.
    pub fn hover(
        &self,
        symbol: &crate::Name,
    ) -> Result<crate::HoverInfo, crate::SymbolLookupError> {
        // Trigger lazy loading for class-rooted symbols before snapshotting.
        // No-op when the class is already known; ensures inherited member
        // lookups have the chain present.
        match symbol {
            crate::Name::Class(fqcn) => {
                self.load_class(fqcn.as_ref());
            }
            crate::Name::Method { class, .. }
            | crate::Name::Property { class, .. }
            | crate::Name::ClassConstant { class, .. } => {
                // Fault in the owning class for navigation if the background
                // indexer hasn't reached it yet. Its inheritance ancestors
                // resolve through the (eagerly-built) workspace symbol index.
                self.load_class(class.as_ref());
            }
            _ => {}
        }
        self.hover_cached(symbol)
    }

    /// Pure variant of [`Self::hover`]. Never invokes the
    /// [`crate::SourceProvider`]; consults only the already-loaded db.
    pub fn hover_cached(
        &self,
        symbol: &crate::Name,
    ) -> Result<crate::HoverInfo, crate::SymbolLookupError> {
        use mir_types::{Atomic, Type};
        let db = self.snapshot_db();
        match symbol {
            crate::Name::Function(fqn) => {
                let here = crate::db::Fqcn::from_str(&db, fqn.as_ref());
                let f = crate::db::find_function(&db, here)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = f
                    .return_type
                    .as_deref()
                    .cloned()
                    .unwrap_or_else(Type::mixed);
                let docstring = f.docstring.as_ref().map(|s| s.to_string());
                Ok(crate::HoverInfo {
                    ty,
                    docstring,
                    definition: f.location.clone(),
                })
            }
            crate::Name::Method { class, name } => {
                let here = crate::db::Fqcn::from_str(&db, class.as_ref());
                let (_, m) = crate::db::find_method_in_chain(&db, here, name)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = m
                    .return_type
                    .as_deref()
                    .cloned()
                    .unwrap_or_else(Type::mixed);
                let docstring = m.docstring.as_ref().map(|s| s.to_string());
                Ok(crate::HoverInfo {
                    ty,
                    docstring,
                    definition: m.location.clone(),
                })
            }
            crate::Name::Class(fqcn) => {
                let here = crate::db::Fqcn::from_str(&db, fqcn.as_ref());
                let class = crate::db::find_class_like(&db, here)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = Type::single(Atomic::TNamedObject {
                    fqcn: mir_types::Name::from(fqcn.as_ref()),
                    type_params: mir_types::union::empty_type_params(),
                });
                Ok(crate::HoverInfo {
                    ty,
                    docstring: None,
                    definition: class.location().cloned(),
                })
            }
            crate::Name::Property { class, name } => {
                let here = crate::db::Fqcn::from_str(&db, class.as_ref());
                let (_, p) = crate::db::find_property_in_chain(&db, here, name)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = p.ty.as_deref().cloned().unwrap_or_else(Type::mixed);
                Ok(crate::HoverInfo {
                    ty,
                    docstring: None,
                    definition: p.location.clone(),
                })
            }
            crate::Name::ClassConstant { class, name } => {
                let here = crate::db::Fqcn::from_str(&db, class.as_ref());
                let (_, c) = crate::db::find_class_constant_in_chain(&db, here, name)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                Ok(crate::HoverInfo {
                    ty: c.ty.clone(),
                    docstring: None,
                    definition: c.location.clone(),
                })
            }
            crate::Name::GlobalConstant(fqn) => {
                let here = crate::db::Fqcn::from_str(&db, fqn.as_ref());
                let ty = crate::db::find_global_constant(&db, here)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                Ok(crate::HoverInfo {
                    ty: (*ty).clone(),
                    docstring: None,
                    definition: None,
                })
            }
        }
    }

    /// Raw reference locations indexed by string symbol key, kept for tests
    /// that use the legacy stringly-typed API. Prefer [`Self::indexed_references_to`]
    /// with a typed [`crate::Name`].
    #[doc(hidden)]
    pub fn reference_locations(&self, symbol: &str) -> Vec<(Arc<str>, u32, u16, u16)> {
        use crate::db::MirDatabase;
        let db = self.snapshot_db();
        db.reference_locations(symbol)
    }

    /// Files declaring transitive subclasses of `class_fqn`, backed by the
    /// maintained subtype index (see [`Self::indexed_subtype_classes`]).
    /// Excludes `class_fqn`'s own declaring file — the caller adds it.
    ///
    /// Lets a reference-search caller scope a `protected` member to its class
    /// hierarchy without reconstructing that hierarchy from declaration text:
    /// subclasses are matched by resolved FQCN, so `extends \Ns\Base` and
    /// aliased `use` forms are all found. Read-only from the caller's
    /// perspective; may trigger an on-demand commit of stale/uncommitted
    /// candidates' class edges (same self-heal `indexed_subtype_classes` uses).
    pub fn subtype_files(&self, class_fqn: &str) -> Vec<Arc<str>> {
        let files = self.snapshot_db().source_file_paths();
        let mut out: Vec<Arc<str>> = self
            .indexed_subtype_classes(class_fqn, &files, false)
            .into_iter()
            .map(|s| s.file)
            .collect();
        out.sort();
        out.dedup();
        out
    }

    /// `use`-import occurrences of `symbol` — the import statement's own name
    /// token (`use Foo\Bar;`, `use function ...;`, `use const ...;`), not a
    /// usage site. Recorded under a `use:`-prefixed posting distinct from the
    /// plain `cls:`/`fn:`/`gcnst:` keys [`Self::indexed_references_to`] reads,
    /// so a symbol rename can also find/update the import line without a
    /// plain find-references query suddenly including import statements.
    ///
    /// Read-only posting-list lookup, filtered to `files` — no freshness pass:
    /// callers that need guaranteed-fresh results for an uncommitted file
    /// should analyze it first (e.g. via [`Self::indexed_references_to`] on
    /// the same file set).
    pub fn indexed_use_import_locations(
        &self,
        symbol: &crate::Name,
        files: &[Arc<str>],
    ) -> Vec<(Arc<str>, crate::Range)> {
        let key = format!("use:{}", symbol.codebase_key());
        let scope: rustc_hash::FxHashSet<&str> = files.iter().map(|f| f.as_ref()).collect();
        let guard = self.db.salsa.read();
        let mut out: Vec<(Arc<str>, crate::Range)> = guard
            .reference_locations(&key)
            .into_iter()
            .filter(|(file, ..)| scope.contains(file.as_ref()))
            .map(|(file, line, col_start, col_end)| {
                (file, span_range(line, col_start as u32, col_end as u32))
            })
            .collect();
        out.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then(a.1.start.line.cmp(&b.1.start.line))
                .then(a.1.start.column.cmp(&b.1.start.column))
        });
        out.dedup();
        out
    }

    /// Inverted-index find-references: posting-list lookup plus an on-demand
    /// freshness/completeness pass over `files` (the host's text-prefiltered
    /// candidate scope).
    ///
    /// A candidate whose postings were committed from its current input text
    /// (Arc identity) is answered from the index with no salsa work at all.
    /// Stale or never-committed candidates are analyzed via the memoized
    /// `analyze_file` query and committed, so each file pays that cost once
    /// per text change — after a background warm sweep the steady state is a
    /// pure lookup, O(results) instead of O(candidates).
    ///
    /// Results are filtered to `files` (the host controls scope — e.g.
    /// workspace files only, excluding stubs/vendor). With
    /// `include_declaration`, the symbol's declaration name span is appended
    /// when it lies inside the scope.
    ///
    /// `should_cancel` follows [`Self::references_to_in_files_cancellable`]'s
    /// contract: polled at phase boundaries and between cancellation retries;
    /// `true` aborts with `None`.
    pub fn indexed_references_to(
        &self,
        symbol: &crate::Name,
        files: &[Arc<str>],
        include_declaration: bool,
        should_cancel: &(dyn Fn() -> bool + Sync),
    ) -> Option<Vec<(Arc<str>, crate::Range)>> {
        use std::panic::AssertUnwindSafe;

        use rayon::prelude::*;

        let key = symbol.codebase_key();

        // Freshness pass: candidates whose postings are not exact for their
        // current text. Files not registered as `SourceFile` inputs are
        // skipped (the caller's text pre-filter already scoped the set).
        let stale: Vec<Arc<str>> = loop {
            if should_cancel() {
                return None;
            }
            let attempt = salsa::Cancelled::catch(AssertUnwindSafe(|| {
                let current_gen = self.index_generation();
                let db = self.snapshot_db();
                files
                    .iter()
                    .filter(|f| {
                        db.lookup_source_file(f.as_ref()).is_some_and(|sf| {
                            let text = sf.text(&db as &dyn MirDatabase);
                            !self.is_ref_committed(f.as_ref(), text, current_gen)
                        })
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            }));
            match attempt {
                Ok(v) => break v,
                Err(_) if should_cancel() => return None,
                Err(_) => {}
            }
        };

        if !stale.is_empty() {
            // Phase 1 (serial, no live snapshot held): warm up stale
            // candidates. See `references_to_in_files_cancellable` for why
            // this must be serial and snapshot-free.
            for path in &stale {
                if should_cancel() {
                    return None;
                }
                self.prepare_file_for_analysis(path);
            }

            // Phase 2 (parallel, pure) under a cancellation retry loop, then
            // a serial commit into both inverted indexes.
            let (commit_gen, analyzed) = loop {
                if should_cancel() {
                    return None;
                }
                // Generation before the snapshot: a file add racing the
                // analysis leaves these commits stale (self-healing on the
                // next query), never wrongly fresh.
                let gen = self.index_generation();
                let attempt = salsa::Cancelled::catch(AssertUnwindSafe(|| {
                    let db_main = self.snapshot_db();
                    stale
                        .par_iter()
                        .map_with(db_main, |db, path| {
                            let sf = db.lookup_source_file(path.as_ref())?;
                            let text = sf.text(&*db as &dyn MirDatabase).clone();
                            let out = crate::db::analyze_file(&*db as &dyn MirDatabase, sf).clone();
                            let defs =
                                crate::db::collect_file_definitions(&*db as &dyn MirDatabase, sf);
                            let entries = crate::db::subtype_index::entries_from_slice(&defs.slice);
                            // Stage the disk-cache write only when the commit
                            // below will rewrite postings (see the sweep in
                            // `reanalyze_file_set` for the cost rationale).
                            let put = if self.ref_commit_is_current(path.as_ref(), &text, &out) {
                                None
                            } else {
                                self.stage_ref_cache_put(
                                    &*db as &dyn MirDatabase,
                                    sf,
                                    path.as_ref(),
                                    &text,
                                    &out,
                                )
                            };
                            Some((path.clone(), text, out, entries, put))
                        })
                        .flatten()
                        .collect::<Vec<_>>()
                }));
                match attempt {
                    Ok(v) => break (gen, v),
                    Err(_) if should_cancel() => return None,
                    Err(_) => {}
                }
            };
            let mut analyzed = analyzed;
            let guard = self.db.salsa.read();
            for (file, text, out, entries, put) in analyzed.iter_mut() {
                // Pointer-identical memo ⇒ identical postings: skip the
                // index rewrite and only re-stamp the freshness mark.
                if !self.ref_commit_is_current(file.as_ref(), text, out) {
                    guard.set_file_reference_locations(file.as_ref(), out.ref_locs.to_vec());
                }
                if let Some(put) = put.take() {
                    self.apply_ref_cache_put(file.as_ref(), out, put);
                }
                self.mark_ref_committed(
                    file,
                    text,
                    Some(out),
                    commit_gen,
                    !out.has_unresolved_names(),
                );
                if !self.is_defs_committed(file.as_ref(), text) {
                    guard.set_file_class_edges(file, entries.clone());
                    self.mark_defs_committed(file, text);
                }
            }
        }

        // Posting lookup, filtered to the candidate scope.
        //
        // Member symbols resolve against the queried class plus its hierarchy
        // (mir records member refs under the *declaring* class, so a query on
        // an interface method must include implementor keys and vice versa).
        // Name-only fallback postings — receivers whose type couldn't be
        // resolved — are consulted only when the typed keys produce nothing,
        // mirroring the pre-index two-tier behavior: exact results when
        // resolution succeeds, by-name matches when nothing resolves.
        // `__construct` stays exact: `new Sub()` invokes `Sub::__construct`
        // even when only a parent declares one, so hierarchy fan-out would
        // wrongly return subtype instantiation sites for a parent query.
        let hierarchy: Vec<String> = match symbol {
            crate::Name::Method { class, name } => {
                if name.as_ref() == "__construct" || class.is_empty() {
                    if class.is_empty() {
                        Vec::new()
                    } else {
                        vec![class.trim_start_matches('\\').to_string()]
                    }
                } else {
                    self.member_hierarchy_classes(class.as_ref())
                }
            }
            crate::Name::Property { class, .. } | crate::Name::ClassConstant { class, .. } => {
                if class.is_empty() {
                    Vec::new()
                } else {
                    self.member_hierarchy_classes(class.as_ref())
                }
            }
            _ => Vec::new(),
        };
        let primary_keys: Vec<String> = match symbol {
            crate::Name::Method { name, .. } => hierarchy
                .iter()
                .map(|c| format!("meth:{c}::{name}"))
                .collect(),
            crate::Name::Property { name, .. } => hierarchy
                .iter()
                .map(|c| format!("prop:{c}::{name}"))
                .collect(),
            crate::Name::ClassConstant { name, .. } => hierarchy
                .iter()
                .map(|c| format!("cnst:{c}::{name}"))
                .collect(),
            _ => vec![key.clone()],
        };
        let fallback_key: Option<String> = match symbol {
            crate::Name::Method { name, .. } => Some(format!("methname:{name}")),
            crate::Name::Property { name, .. } => Some(format!("propname:{name}")),
            _ => None,
        };
        let scope: rustc_hash::FxHashSet<&str> = files.iter().map(|f| f.as_ref()).collect();
        let read_keys = |keys: &[String]| -> Vec<(Arc<str>, crate::Range)> {
            let guard = self.db.salsa.read();
            let mut merged: Vec<(Arc<str>, u32, u16, u16)> = Vec::new();
            for k in keys {
                merged.extend(guard.reference_locations(k));
            }
            merged
                .into_iter()
                .filter(|(file, ..)| scope.contains(file.as_ref()))
                .map(|(file, line, col_start, col_end)| {
                    (file, span_range(line, col_start as u32, col_end as u32))
                })
                .collect()
        };
        let mut out = read_keys(&primary_keys);
        if out.is_empty() {
            if let Some(fk) = fallback_key {
                out = read_keys(std::slice::from_ref(&fk));
            }
        }
        out.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then(a.1.start.line.cmp(&b.1.start.line))
                .then(a.1.start.column.cmp(&b.1.start.column))
        });
        out.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);

        if include_declaration {
            // Declaration lookup runs salsa queries (and may lazy-load); a
            // concurrent write cancels it — declarations are then simply
            // omitted rather than failing the whole request.
            let decls: Vec<(Arc<str>, crate::Range)> = match symbol {
                crate::Name::Method { class, .. }
                | crate::Name::Property { class, .. }
                | crate::Name::ClassConstant { class, .. } => {
                    if class.is_empty() {
                        // Unknown owner: declarations by name, recorded as
                        // `methdecl:`/`propdecl:`/`cnstdecl:` postings during
                        // class/trait/interface/enum analysis.
                        match symbol {
                            crate::Name::Method { name, .. } => {
                                read_keys(&[format!("methdecl:{name}")])
                            }
                            crate::Name::Property { name, .. } => {
                                read_keys(&[format!("propdecl:{name}")])
                            }
                            crate::Name::ClassConstant { name, .. } => {
                                read_keys(&[format!("cnstdecl:{name}")])
                            }
                            _ => Vec::new(),
                        }
                    } else {
                        salsa::Cancelled::catch(AssertUnwindSafe(|| {
                            self.member_decl_sites(&hierarchy, symbol)
                        }))
                        .unwrap_or_default()
                    }
                }
                _ => salsa::Cancelled::catch(AssertUnwindSafe(|| {
                    self.declaration_name_range(symbol).into_iter().collect()
                }))
                .unwrap_or_default(),
            };
            for (file, range) in decls {
                if scope.contains(file.as_ref())
                    && !out.iter().any(|(f, r)| *f == file && *r == range)
                {
                    out.push((file, range));
                }
            }
        }
        Some(out)
    }

    /// The queried class plus every class its members' references could be
    /// keyed under: resolved ancestors (a call on a subtype instance records
    /// the declaring ancestor) and transitive subtypes including trait users
    /// (a call on a subtype that overrides records the subtype). Display-form
    /// FQCNs, deduplicated case-insensitively.
    fn member_hierarchy_classes(&self, class_fqn: &str) -> Vec<String> {
        use std::panic::AssertUnwindSafe;
        let target = class_fqn.trim_start_matches('\\').to_string();
        let mut out: Vec<String> = vec![target.clone()];
        let ancestors = salsa::Cancelled::catch(AssertUnwindSafe(|| {
            let db = self.snapshot_db();
            let here = crate::db::Fqcn::from_str(&db, &target);
            crate::db::class_ancestors_by_fqcn(&db, here)
                .iter()
                .skip(1)
                .map(|a| a.trim_start_matches('\\').to_string())
                .collect::<Vec<_>>()
        }))
        .unwrap_or_default();
        out.extend(ancestors);
        let subs = {
            let guard = self.db.salsa.read();
            guard.subtype_sites_of(&target, true)
        };
        out.extend(
            subs.into_iter()
                .map(|s| s.fqcn.trim_start_matches('\\').to_string()),
        );
        let mut seen: rustc_hash::FxHashSet<String> = rustc_hash::FxHashSet::default();
        out.retain(|c| seen.insert(c.to_ascii_lowercase()));
        out
    }

    /// Own-member declaration sites for `symbol` across `classes`: each class
    /// that itself declares the member (not inherited) contributes its name
    /// token. Kind-specific lookups — a class often declares a property and a
    /// method with the same short name, and `member_location` can't tell them
    /// apart.
    fn member_decl_sites(
        &self,
        classes: &[String],
        symbol: &crate::Name,
    ) -> Vec<(Arc<str>, crate::Range)> {
        let mut out: Vec<(Arc<str>, crate::Range)> = Vec::new();
        let db = self.snapshot_db();
        for class in classes {
            let here = crate::db::Fqcn::from_str(&db, class);
            let (loc, needle) = match symbol {
                crate::Name::Method { name, .. } => {
                    let Some(m) = crate::db::find_method_in_class(&db, here, name) else {
                        continue;
                    };
                    (m.location.clone(), name.to_string())
                }
                crate::Name::Property { name, .. } => {
                    let Some(p) = crate::db::find_property_in_class(&db, here, name) else {
                        continue;
                    };
                    (p.location.clone(), name.to_string())
                }
                crate::Name::ClassConstant { name, .. } => {
                    let Some(c) = crate::db::find_class_constant_in_class(&db, here, name) else {
                        continue;
                    };
                    (c.location.clone(), name.to_string())
                }
                _ => continue,
            };
            let Some(loc) = loc else { continue };
            let range = self.refine_location_to_name(&loc, &needle);
            out.push((loc.file.clone(), range));
        }
        out
    }

    /// The symbol's declaration site, narrowed from the collector's
    /// whole-declaration span to the declared name's own token (matching the
    /// span shape of recorded references).
    pub fn declaration_name_range(&self, symbol: &crate::Name) -> Option<(Arc<str>, crate::Range)> {
        if let crate::Name::GlobalConstant(fqn) = symbol {
            return self.global_constant_decl_range(fqn);
        }
        let loc = self.definition_of(symbol).ok()?;
        let short = match symbol {
            crate::Name::Class(f) | crate::Name::Function(f) | crate::Name::GlobalConstant(f) => {
                crate::db::subtype_index::short_name_of(f)
            }
            crate::Name::Method { name, .. }
            | crate::Name::Property { name, .. }
            | crate::Name::ClassConstant { name, .. } => name.as_ref(),
        };
        // Property declarations carry a `$` sigil in source, but reference
        // ranges cover the bare name; the word-boundary search below lands on
        // the name right after the sigil.
        let file = loc.file.clone();
        let range = self.refine_location_to_name(&loc, short);
        Some((file, range))
    }

    /// Narrow a whole-declaration [`mir_types::Location`] to the first
    /// word-boundary occurrence of `needle` inside its line span. Falls back
    /// to the location's own coordinates when the text is unavailable or the
    /// name doesn't appear (e.g. stub-only declarations).
    fn refine_location_to_name(&self, loc: &mir_types::Location, needle: &str) -> crate::Range {
        let fallback = span_range(loc.line, loc.col_start as u32, loc.col_end as u32);
        let text = {
            let db = self.snapshot_db();
            db.lookup_source_file(loc.file.as_ref())
                .map(|sf| sf.text(&db as &dyn MirDatabase).clone())
        };
        let Some(text) = text else {
            return fallback;
        };
        let needle_chars = needle.chars().count() as u32;
        let first_line = loc.line.saturating_sub(1) as usize;
        // Exact-case first: PHP property/constant names are case-sensitive
        // and an early case-insensitive hit can land on an unrelated token
        // (a type hint sharing the name). Case-insensitive second, for
        // method/class needles that arrive lowercase-normalized.
        for case_insensitive in [false, true] {
            for (idx, line_text) in text.lines().enumerate().skip(first_line) {
                let line_no = idx as u32 + 1;
                if line_no > loc.line_end {
                    break;
                }
                let min_col = if line_no == loc.line {
                    loc.col_start as usize
                } else {
                    0
                };
                if let Some(col) = identifier_char_col(line_text, needle, min_col, case_insensitive)
                {
                    return span_range(line_no, col, col + needle_chars);
                }
            }
        }
        fallback
    }

    /// Transitive subtypes of `class_fqn` (classes/interfaces/enums whose
    /// resolved ancestor chain reaches it), answered from the maintained
    /// subtype edge index.
    ///
    /// `files` is the host's candidate scope for the on-demand completeness
    /// pass: per BFS round, not-yet-committed files whose text mentions a
    /// frontier name get their definitions committed, so results are complete
    /// even before a background sweep has covered the workspace. Committed
    /// files answer from the index with no parsing at all.
    ///
    /// `include_trait_users` also counts `use Trait;` composition as a
    /// subtype edge (visibility-scoping semantics); leave it off for
    /// goto-implementation semantics (extends/implements only).
    pub fn indexed_subtype_classes(
        &self,
        class_fqn: &str,
        files: &[Arc<str>],
        include_trait_users: bool,
    ) -> Vec<SubtypeClassSite> {
        let mut scanned: rustc_hash::FxHashSet<String> = rustc_hash::FxHashSet::default();
        let mut pending: Vec<String> = vec![class_fqn.trim_start_matches('\\').to_string()];
        let mut sites: Vec<crate::db::SubtypeSite> = Vec::new();
        while !pending.is_empty() {
            let needles: Vec<String> = pending
                .drain(..)
                .filter(|f| scanned.insert(f.clone()))
                .map(|f| crate::db::subtype_index::short_name_of(&f).to_string())
                .collect();
            if !needles.is_empty() {
                self.commit_defs_for_matching(files, &needles);
            }
            sites = {
                let guard = self.db.salsa.read();
                guard.subtype_sites_of_lenient(class_fqn, include_trait_users)
            };
            pending = sites
                .iter()
                .map(|s| s.fqcn.trim_start_matches('\\').to_string())
                .filter(|f| !scanned.contains(f))
                .collect();
        }
        let mut out: Vec<SubtypeClassSite> = sites
            .into_iter()
            .filter_map(|s| {
                let loc = s.location.as_ref()?;
                let short = crate::db::subtype_index::short_name_of(&s.fqcn).to_string();
                let range = self.refine_location_to_name(loc, &short);
                Some(SubtypeClassSite {
                    fqcn: s.fqcn,
                    kind: s.kind,
                    is_abstract: s.is_abstract,
                    file: s.file,
                    range,
                })
            })
            .collect();
        // Anonymous classes never reach the definition collector; their
        // `new class implements X {}` sites are recorded as `impl:` postings
        // during body analysis (exact FQCN key plus a short-name key for the
        // same written-form leniency named classes get above).
        let root_lc = class_fqn.trim_start_matches('\\').to_ascii_lowercase();
        let short_lc = crate::db::subtype_index::short_name_of(&root_lc).to_string();
        let scope: rustc_hash::FxHashSet<&str> = files.iter().map(|f| f.as_ref()).collect();
        let anon: Vec<(Arc<str>, u32, u16, u16)> = {
            let guard = self.db.salsa.read();
            let mut v = guard.reference_locations(&format!("impl:{root_lc}"));
            v.extend(guard.reference_locations(&format!("implshort:{short_lc}")));
            v.sort();
            v.dedup();
            v
        };
        for (file, line, cs, ce) in anon {
            if !scope.contains(file.as_ref()) {
                continue;
            }
            let range = span_range(line, cs as u32, ce as u32);
            if out.iter().any(|s| s.file == file && s.range == range) {
                continue;
            }
            out.push(SubtypeClassSite {
                fqcn: Arc::from("class@anonymous"),
                kind: crate::db::ClassLikeKind::Class,
                is_abstract: false,
                file,
                range,
            });
        }
        out
    }

    /// Concrete implementations of `class_fqn::method` across its transitive
    /// subtypes: the same-named non-abstract method available to each subtype
    /// (its own declaration, or one inherited/composed from a parent, trait,
    /// or mixin), as `(subtype fqcn, file, name range)`. Subtypes resolving to
    /// the same declaring location collapse to a single entry.
    pub fn indexed_method_implementations(
        &self,
        class_fqn: &str,
        method: &str,
        files: &[Arc<str>],
    ) -> Vec<(Arc<str>, Arc<str>, crate::Range)> {
        use std::panic::AssertUnwindSafe;
        let subs = self.indexed_subtype_classes(class_fqn, files, false);
        if subs.is_empty() {
            return Vec::new();
        }
        loop {
            let attempt = salsa::Cancelled::catch(AssertUnwindSafe(|| {
                let db = self.snapshot_db();
                let mut out: Vec<(Arc<str>, Arc<str>, crate::Range)> = Vec::new();
                for sub in &subs {
                    let here = crate::db::Fqcn::from_str(&db, sub.fqcn.as_ref());
                    let Some((_, m)) = crate::db::find_method_in_chain(&db, here, method) else {
                        continue;
                    };
                    if m.is_abstract {
                        continue;
                    }
                    let Some(loc) = m.location.as_ref() else {
                        continue;
                    };
                    let range = self.refine_location_to_name(loc, method);
                    out.push((sub.fqcn.clone(), loc.file.clone(), range));
                }
                out
            }));
            if let Ok(mut out) = attempt {
                out.sort_by(|a, b| a.1.cmp(&b.1).then(a.2.start.line.cmp(&b.2.start.line)));
                out.dedup_by(|a, b| a.1 == b.1 && a.2 == b.2);
                return out;
            }
        }
    }

    /// Commit definitions (class edges + freshness) for every file in `files`
    /// that is stale (committed against older text) or that has never been
    /// committed and mentions one of `shorts` as a whole identifier.
    fn commit_defs_for_matching(&self, files: &[Arc<str>], shorts: &[String]) {
        use std::panic::AssertUnwindSafe;

        use rayon::prelude::*;

        let committed_any: rustc_hash::FxHashSet<Arc<str>> = {
            let guard = self.defs_committed_keys();
            guard.into_iter().collect()
        };
        let work = loop {
            let attempt = salsa::Cancelled::catch(AssertUnwindSafe(|| {
                let db_main = self.snapshot_db();
                files
                    .par_iter()
                    .map_with(db_main, |db, path| {
                        let sf = db.lookup_source_file(path.as_ref())?;
                        let text = sf.text(&*db as &dyn MirDatabase).clone();
                        if self.is_defs_committed(path.as_ref(), &text) {
                            return None;
                        }
                        // Never-committed files must mention a frontier name;
                        // stale (previously committed) files recommit
                        // unconditionally — their classes may have re-parented.
                        if !committed_any.contains(path.as_ref())
                            && !shorts.iter().any(|s| mentions_identifier(&text, s))
                        {
                            return None;
                        }
                        let defs =
                            crate::db::collect_file_definitions(&*db as &dyn MirDatabase, sf);
                        let entries = crate::db::subtype_index::entries_from_slice(&defs.slice);
                        Some((path.clone(), text, entries))
                    })
                    .flatten()
                    .collect::<Vec<_>>()
            }));
            if let Ok(v) = attempt {
                break v;
            }
        };
        if work.is_empty() {
            return;
        }
        let guard = self.db.salsa.read();
        for (file, text, entries) in &work {
            guard.set_file_class_edges(file, entries.clone());
            self.mark_defs_committed(file, text);
        }
    }

    /// Declaration name span for a global constant. Constant slices carry no
    /// stored location, so this finds the declaring file via the workspace
    /// constants index and locates the `const NAME` / `define('NAME'` token
    /// textually.
    fn global_constant_decl_range(&self, fqn: &str) -> Option<(Arc<str>, crate::Range)> {
        use std::panic::AssertUnwindSafe;
        let short = crate::db::subtype_index::short_name_of(fqn).to_string();
        salsa::Cancelled::catch(AssertUnwindSafe(|| {
            let db = self.snapshot_db();
            let index = crate::db::workspace_index(&db);
            let loc = index
                .constants
                .get(&mir_types::Name::from(fqn.trim_start_matches('\\')))?;
            let file = loc.file().path(&db).clone();
            let sf = db.lookup_source_file(file.as_ref())?;
            let text = sf.text(&db as &dyn MirDatabase);
            for (idx, line) in text.lines().enumerate() {
                let trimmed = line.trim_start();
                let is_decl_line = trimmed.starts_with("const ")
                    || trimmed.contains("define(")
                    || trimmed.contains("define (");
                if !is_decl_line {
                    continue;
                }
                if let Some(col) = identifier_char_col(line, &short, 0, false) {
                    let n = short.chars().count() as u32;
                    return Some((file, span_range(idx as u32 + 1, col, col + n)));
                }
            }
            None
        }))
        .ok()
        .flatten()
    }

    /// Class-level issues (inheritance violations, abstract-method gaps, override
    /// incompatibilities) for the given set of files.
    ///
    /// These checks are cross-file by nature and are not emitted by
    /// [`crate::FileAnalyzer::analyze`]. Call this after ingesting or
    /// re-analyzing a file and its dependents to get the full diagnostic picture.
    ///
    /// Circular-inheritance checks always run against the full workspace graph
    /// regardless of the `files` filter — a cycle is a workspace-wide problem.
    pub fn class_issues(&self, files: &[Arc<str>]) -> Vec<crate::Issue> {
        let db = self.snapshot_db();
        let file_set: HashSet<Arc<str>> = files.iter().cloned().collect();
        // Read source texts through the snapshot already in hand — calling
        // `source_of` here would re-enter the session RwLock while this
        // snapshot is live, and a concurrent salsa write (which blocks new
        // readers behind the fair write lock while waiting for existing
        // snapshots to drop) turns that into a deadlock.
        let file_data: Vec<(Arc<str>, Arc<str>)> = files
            .iter()
            .filter_map(|f| {
                let sf = db.lookup_source_file(f)?;
                Some((
                    f.clone(),
                    sf.text(&db as &dyn crate::db::MirDatabase).clone(),
                ))
            })
            .collect();
        crate::class::ClassAnalyzer::with_files(&db, file_set, &file_data).analyze_all()
    }

    /// All declarations defined in `file` as a **hierarchical tree**.
    ///
    /// Classes/interfaces/traits/enums are returned with their methods,
    /// properties, and constants nested in `children`. Top-level functions
    /// and constants are returned with empty `children`.
    pub fn document_symbols(&self, file: &str) -> Vec<crate::symbol::DocumentSymbol> {
        use crate::symbol::{DeclarationKind, DocumentSymbol};

        let db = self.snapshot_db();
        let Some(sf) = db.lookup_source_file(file) else {
            return Vec::new();
        };
        let defs = crate::db::collect_file_definitions(&db, sf);
        let mut out: Vec<DocumentSymbol> = Vec::new();

        let class_children = |methods: &mir_codebase::definitions::MemberMap<
            Arc<mir_codebase::definitions::MethodDef>,
        >,
                              props: Option<
            &mir_codebase::definitions::MemberMap<mir_codebase::definitions::PropertyDef>,
        >,
                              consts: &mir_codebase::definitions::MemberMap<
            mir_codebase::definitions::ConstantDef,
        >,
                              is_enum: bool|
         -> Vec<DocumentSymbol> {
            let mut out: Vec<DocumentSymbol> = Vec::new();
            for (_, m) in methods.iter() {
                out.push(DocumentSymbol {
                    name: m.name.clone(),
                    kind: DeclarationKind::Method,
                    location: m.location.clone(),
                    children: Vec::new(),
                });
            }
            if let Some(props) = props {
                for (_, p) in props.iter() {
                    out.push(DocumentSymbol {
                        name: p.name.clone(),
                        kind: DeclarationKind::Property,
                        location: p.location.clone(),
                        children: Vec::new(),
                    });
                }
            }
            let const_kind = if is_enum {
                DeclarationKind::EnumCase
            } else {
                DeclarationKind::Constant
            };
            for (_, c) in consts.iter() {
                out.push(DocumentSymbol {
                    name: c.name.clone(),
                    kind: const_kind,
                    location: c.location.clone(),
                    children: Vec::new(),
                });
            }
            out
        };

        for c in defs.slice.classes.iter() {
            out.push(DocumentSymbol {
                name: c.fqcn.clone(),
                kind: DeclarationKind::Class,
                location: c.location.clone(),
                children: class_children(
                    &c.own_methods,
                    Some(&c.own_properties),
                    &c.own_constants,
                    false,
                ),
            });
        }
        for i in defs.slice.interfaces.iter() {
            out.push(DocumentSymbol {
                name: i.fqcn.clone(),
                kind: DeclarationKind::Interface,
                location: i.location.clone(),
                children: class_children(&i.own_methods, None, &i.own_constants, false),
            });
        }
        for t in defs.slice.traits.iter() {
            out.push(DocumentSymbol {
                name: t.fqcn.clone(),
                kind: DeclarationKind::Trait,
                location: t.location.clone(),
                children: class_children(
                    &t.own_methods,
                    Some(&t.own_properties),
                    &t.own_constants,
                    false,
                ),
            });
        }
        for e in defs.slice.enums.iter() {
            let mut children = class_children(&e.own_methods, None, &e.own_constants, true);
            for (_, case) in e.cases.iter() {
                children.push(DocumentSymbol {
                    name: case.name.clone(),
                    kind: DeclarationKind::EnumCase,
                    location: case.location.clone(),
                    children: Vec::new(),
                });
            }
            out.push(DocumentSymbol {
                name: e.fqcn.clone(),
                kind: DeclarationKind::Enum,
                location: e.location.clone(),
                children,
            });
        }
        for f in defs.slice.functions.iter() {
            out.push(DocumentSymbol {
                name: f.fqn.clone(),
                kind: DeclarationKind::Function,
                location: f.location.clone(),
                children: Vec::new(),
            });
        }
        for (name, _) in defs.slice.constants.iter() {
            out.push(DocumentSymbol {
                name: name.clone(),
                kind: DeclarationKind::Constant,
                location: None,
                children: Vec::new(),
            });
        }
        out
    }
}

/// A transitive subtype hit with its declaration name span, as returned by
/// [`AnalysisSession::indexed_subtype_classes`].
#[derive(Debug, Clone)]
pub struct SubtypeClassSite {
    /// Display-form FQCN (no leading `\`).
    pub fqcn: Arc<str>,
    pub kind: crate::db::ClassLikeKind,
    pub is_abstract: bool,
    pub file: Arc<str>,
    /// The declared name's own token (1-based line, 0-based char columns).
    pub range: crate::Range,
}

/// Build a [`crate::Range`] on one line from mir's native coordinates
/// (1-based line, 0-based columns).
fn span_range(line: u32, col_start: u32, col_end: u32) -> crate::Range {
    crate::Range {
        start: crate::Position {
            line,
            column: col_start,
        },
        end: crate::Position {
            line,
            column: col_end,
        },
    }
}

/// Char column of the first word-boundary occurrence of `needle` in `line`
/// at or after char column `min_col`. Columns are code points, matching the
/// collector's `Location` convention.
fn identifier_char_col(
    line: &str,
    needle: &str,
    min_col: usize,
    case_insensitive: bool,
) -> Option<u32> {
    if needle.is_empty() {
        return None;
    }
    let is_ident = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let chars: Vec<char> = line.chars().collect();
    let needle_chars: Vec<char> = needle.chars().collect();
    let n = needle_chars.len();
    if chars.len() < n {
        return None;
    }
    for start in min_col..=chars.len().saturating_sub(n) {
        let matches = chars[start..start + n]
            .iter()
            .zip(needle_chars.iter())
            .all(|(a, b)| {
                if case_insensitive {
                    a.eq_ignore_ascii_case(b)
                } else {
                    a == b
                }
            });
        if !matches {
            continue;
        }
        let before_ok = start == 0 || !is_ident(chars[start - 1]);
        let after = start + n;
        let after_ok = after >= chars.len() || !is_ident(chars[after]);
        if before_ok && after_ok {
            return Some(start as u32);
        }
    }
    None
}

/// Whether `hay` mentions `needle` as a whole identifier (ASCII word
/// boundaries; conservative near multibyte text). Mirrors the host-side
/// candidate prefilter so the completeness pass never analyzes files that
/// cannot name the symbol.
fn mentions_identifier(hay: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let hay_b = hay.as_bytes();
    let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut from = 0;
    while let Some(rel) = hay[from..].find(needle) {
        let idx = from + rel;
        let before_ok = idx == 0 || !is_ident(hay_b[idx - 1]);
        let end = idx + needle.len();
        let after_ok = end >= hay_b.len() || !is_ident(hay_b[end]);
        if before_ok && after_ok {
            return true;
        }
        // Step to the next char boundary, not just the next byte: PHP allows
        // non-ASCII bytes in identifiers, so `idx + 1` can land mid-codepoint
        // and panic the next `hay[from..]` slice.
        from = idx + 1;
        while from < hay_b.len() && (hay_b[from] & 0xC0) == 0x80 {
            from += 1;
        }
    }
    false
}
