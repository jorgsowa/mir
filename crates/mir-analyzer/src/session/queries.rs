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
    /// that use the legacy stringly-typed API. Prefer [`Self::references_to`]
    /// with a typed [`crate::Name`].
    #[doc(hidden)]
    pub fn reference_locations(&self, symbol: &str) -> Vec<(Arc<str>, u32, u16, u16)> {
        use crate::db::MirDatabase;
        let db = self.snapshot_db();
        db.reference_locations(symbol)
    }

    /// Every recorded reference to `symbol` with its source location as a Range.
    /// Use [`crate::FileAnalysis::symbol_at`] to find the symbol at a cursor,
    /// build a [`crate::Name`] from it, and pass it here.
    pub fn references_to(&self, symbol: &crate::Name) -> Vec<(Arc<str>, crate::Range)> {
        let db = self.snapshot_db();
        let key = symbol.codebase_key();
        db.reference_locations(&key)
            .into_iter()
            .map(|(file, line, col_start, col_end)| {
                let range = crate::Range {
                    start: crate::Position {
                        line,
                        column: col_start as u32,
                    },
                    end: crate::Position {
                        line,
                        column: col_end as u32,
                    },
                };
                (file, range)
            })
            .collect()
    }

    /// Every recorded reference to `symbol` that originates in one of `files`,
    /// computed directly from the memoized [`crate::db::analyze_file`] query.
    ///
    /// Unlike [`Self::references_to`] — which reads the imperatively-maintained
    /// reverse index and therefore requires the files to have been
    /// `ingest_file`d first — this analyzes the given files on demand via salsa:
    /// warm files are memo hits, cold files analyze exactly once, and nothing
    /// mutates the shared reference index. Repeated queries don't churn
    /// reverse-deps or evict caches, so per-request cost stays flat across a
    /// session regardless of how much of the workspace has been touched.
    ///
    /// `files` are source paths; any not registered as a `SourceFile` input are
    /// silently skipped (their refs are simply absent — the caller's text
    /// pre-filter already scoped the set).
    pub fn references_to_in_files(
        &self,
        symbol: &crate::Name,
        files: &[Arc<str>],
    ) -> Vec<(Arc<str>, crate::Range)> {
        self.references_to_in_files_cancellable(symbol, files, &|| false)
            .unwrap_or_default()
    }

    /// Cancellable variant of [`Self::references_to_in_files`].
    ///
    /// `should_cancel` is polled at Phase-1 file boundaries and between
    /// Phase-2 retry attempts; when it returns `true` the query aborts with
    /// `None`. Without it a sustained write stream (rapid edits, background
    /// indexing) keeps cancelling Phase 2's snapshot reads and the retry loop
    /// spins with no way for the caller to abandon the now-stale request.
    pub fn references_to_in_files_cancellable(
        &self,
        symbol: &crate::Name,
        files: &[Arc<str>],
        should_cancel: &(dyn Fn() -> bool + Sync),
    ) -> Option<Vec<(Arc<str>, crate::Range)>> {
        use crate::db::MirDatabase;
        use rayon::prelude::*;
        use std::panic::AssertUnwindSafe;

        let key = symbol.codebase_key();

        // Phase 1 (serial, no live snapshot held across the writes): fault in
        // each file's class references. `analyze_file` resolves names, and an
        // unresolved class triggers `load_class`, which mutates salsa inputs —
        // doing that from inside the parallel phase would cancel the very
        // snapshots it runs on. Each parse takes a scoped snapshot it drops
        // before warming up. Runs ONCE, outside the retry loop: re-running its
        // writes per retry would amplify cancellation (each write cancels
        // sibling readers). Mirrors `reanalyze_dependents`.
        //
        // Files already warmed against their current text skip the parse +
        // AST walk entirely (`prepared_files`), so a warm repeat query pays
        // one map lookup per candidate instead of a serial parse sweep.
        for path in files {
            if should_cancel() {
                return None;
            }
            self.prepare_file_for_analysis(path);
        }

        // Phase 2 (parallel, pure) under a `salsa::Cancelled` retry loop: every
        // referenced class is now loaded, so this is a memoized read with no
        // writes. An external writer (background indexer) bumping the revision
        // still cancels in-flight reads; catch it, let the snapshot unwind and
        // drop (holding one across the retry would deadlock the writer), and
        // retry. Mirrors the host's `snapshot_query` retry on the read path.
        loop {
            let attempt = salsa::Cancelled::catch(AssertUnwindSafe(|| {
                let db_main = self.snapshot_db();
                files
                    .par_iter()
                    .map_with(db_main, |db, path| {
                        let Some(sf) = db.lookup_source_file(path.as_ref()) else {
                            return Vec::new();
                        };
                        let out = crate::db::analyze_file(&*db as &dyn MirDatabase, sf);
                        out.ref_locs
                            .iter()
                            .filter(|loc| loc.symbol_key.as_ref() == key.as_str())
                            .map(|loc| {
                                (
                                    loc.file.clone(),
                                    crate::Range {
                                        start: crate::Position {
                                            line: loc.line,
                                            column: loc.col_start as u32,
                                        },
                                        end: crate::Position {
                                            line: loc.line,
                                            column: loc.col_end as u32,
                                        },
                                    },
                                )
                            })
                            .collect::<Vec<_>>()
                    })
                    .flatten()
                    .collect::<Vec<_>>()
            }));
            if let Ok(refs) = attempt {
                return Some(refs);
            }
            // A write landed mid-read. Before retrying, let the caller decide
            // whether the request is still worth answering.
            if should_cancel() {
                return None;
            }
        }
    }

    /// Files declaring transitive subclasses of `class_fqn`, computed from the
    /// resolved inheritance graph (see [`crate::db::class_subtype_files`]).
    /// Excludes `class_fqn`'s own declaring file — the caller adds it.
    ///
    /// Lets a reference-search caller scope a `protected` member to its class
    /// hierarchy without reconstructing that hierarchy from declaration text:
    /// subclasses are matched by resolved FQCN, so `extends \Ns\Base` and
    /// aliased `use` forms are all found. Read-only; no input mutation.
    pub fn subtype_files(&self, class_fqn: &str) -> Vec<Arc<str>> {
        let db = self.snapshot_db();
        let here = crate::db::Fqcn::from_str(&db, class_fqn);
        crate::db::class_subtype_files(&db, here).to_vec()
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
                Some((f.clone(), sf.text(&db as &dyn crate::db::MirDatabase)))
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

        let class_children =
            |methods: &mir_codebase::definitions::MemberMap<Arc<mir_codebase::definitions::MethodDef>>,
             props: Option<
                &mir_codebase::definitions::MemberMap<mir_codebase::definitions::PropertyDef>,
            >,
             consts: &mir_codebase::definitions::MemberMap<mir_codebase::definitions::ConstantDef>,
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
