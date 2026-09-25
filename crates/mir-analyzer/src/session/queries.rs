use std::sync::Arc;
use std::time::Instant;

use super::snapshot::catch;
use super::*;

impl AnalysisSession {
    /// Open-file diagnostics entrypoint for issue-only consumers.
    ///
    /// Updates the session's current text for `file`, reuses the tracked parse
    /// and definition queries for syntax/collector diagnostics, then runs the
    /// targeted open-file body-analysis path without retaining whole-file
    /// `ResolvedSymbol` payloads.
    pub fn analyze_file_diagnostics(&mut self, file: &str, source: &str) -> crate::FileAnalysis {
        let file: Arc<str> = Arc::from(file);
        self.ingest_file(file.clone(), Arc::from(source));

        // Body analysis can load classes, a write that would wait forever on
        // a db handle still alive here.
        let (mut issues, parsed) = {
            let view = self.db_view();
            let db = view.db();
            let Some(sf) = db.lookup_source_file(file.as_ref()) else {
                return crate::FileAnalysis {
                    issues: Vec::new(),
                    symbols: Vec::new(),
                };
            };
            let defs = crate::db::collect_file_definitions(db, sf);
            let prepared = crate::db::prepare_analysis_file(db, sf);
            let parsed = (!prepared.has_hard_parse_errors)
                .then(|| (prepared.text.clone(), prepared.parsed.0.clone()));
            (Arc::unwrap_or_clone(defs.issues.clone()), parsed)
        };

        if let Some((text, parsed)) = parsed {
            let analysis = crate::FileAnalyzer::new(self).analyze_diagnostics_only(
                file.clone(),
                text.as_ref(),
                &parsed.program,
                &parsed.source_map,
            );
            issues.extend(analysis.issues);
        }

        self.apply_suppressions_and_emit_unused(&mut issues, std::slice::from_ref(&file));
        crate::FileAnalysis {
            issues,
            symbols: Vec::new(),
        }
    }

    fn codebase_name_at_via_resolve(
        &mut self,
        file: &str,
        byte_offset: u32,
    ) -> Result<crate::Name, crate::SymbolLookupError> {
        self.resolve_at(file, byte_offset)
            .ok_or(crate::SymbolLookupError::NotFound)?
            .to_symbol()
            .ok_or(crate::SymbolLookupError::NotFound)
    }

    /// Resolve the codebase-level symbol name at `byte_offset` in `file`.
    ///
    /// This is the compact cursor-navigation helper for consumers that only
    /// need symbol identity (for example, references queries) and not the full
    /// `ResolvedSymbol` payload.
    pub fn name_at(&mut self, file: &str, byte_offset: u32) -> Option<crate::Name> {
        crate::FileAnalyzer::new(self).resolve_name_at(Arc::from(file), byte_offset)
    }

    /// Resolve the symbol at `byte_offset` in `file`'s current ingested text.
    ///
    /// Canonical open-file navigation entrypoint: unlike
    /// [`crate::FileAnalysis::symbol_at`], this does not require the caller to
    /// retain a whole-file symbol list from diagnostics.
    pub fn symbol_at(&mut self, file: &str, byte_offset: u32) -> Option<crate::ResolvedSymbol> {
        self.resolve_at(file, byte_offset)
    }

    /// Resolve the symbol at `byte_offset` in `file`'s current ingested text.
    ///
    /// This powers hover / go-to-definition without requiring the caller to
    /// retain a whole-file `ResolvedSymbol` list: the containing scope is
    /// re-analyzed on demand and only its symbol payload is searched.
    ///
    /// **Side effects:** like [`Self::definition_of`] and [`Self::hover`], this
    /// may fault in direct dependencies of `file` by running the open-file
    /// warm-up path (`prepare_file_for_analysis`) before snapshotting.
    pub fn resolve_at(&mut self, file: &str, byte_offset: u32) -> Option<crate::ResolvedSymbol> {
        crate::FileAnalyzer::new(self).resolve_at(Arc::from(file), byte_offset)
    }

    /// Hover information for the symbol at `byte_offset` in `file`.
    ///
    /// Uses the targeted [`Self::resolve_at`] navigation path, then resolves
    /// the resulting symbol's hover payload.
    pub fn hover_at(
        &mut self,
        file: &str,
        byte_offset: u32,
    ) -> Result<crate::HoverInfo, crate::SymbolLookupError> {
        let started = Instant::now();
        let hover = (|| {
            let resolved = self
                .resolve_at(file, byte_offset)
                .ok_or(crate::SymbolLookupError::NotFound)?;
            let Some(name) = resolved.to_symbol() else {
                return Ok(crate::HoverInfo {
                    ty: resolved.resolved_type,
                    docstring: None,
                    definition: None,
                });
            };

            let mut hover = self.hover(&name)?;
            if !matches!(
                resolved.kind,
                crate::ReferenceKind::ClassReference(_) | crate::ReferenceKind::UseImport(_)
            ) {
                hover.ty = resolved.resolved_type;
            }
            Ok(hover)
        })();
        crate::metrics::record_hover_at(started.elapsed().as_micros() as u64);
        hover
    }

    /// Definition location for the symbol at `byte_offset` in `file`.
    ///
    /// Uses the targeted [`Self::resolve_at`] navigation path, then resolves
    /// the resulting symbol to its declaration site.
    pub fn definition_at(
        &mut self,
        file: &str,
        byte_offset: u32,
    ) -> Result<mir_types::Location, crate::SymbolLookupError> {
        let started = Instant::now();
        let definition = (|| {
            let name = self.codebase_name_at_via_resolve(file, byte_offset)?;
            self.definition_of(&name)
        })();
        crate::metrics::record_definition_at(started.elapsed().as_micros() as u64);
        definition
    }

    /// Reference locations for the symbol at `byte_offset` in `file`.
    ///
    /// This is the compact cursor-navigation entrypoint for find-references:
    /// resolve a typed symbol identity with [`Self::name_at`], then answer the
    /// query from the maintained reference index.
    pub fn references_at(
        &mut self,
        file: &str,
        byte_offset: u32,
        files: &[Arc<str>],
        include_declaration: bool,
        includes: crate::ReferenceIncludes,
    ) -> Result<Vec<(Arc<str>, crate::Range)>, crate::SymbolLookupError> {
        self.references_at_cancellable(
            file,
            byte_offset,
            files,
            include_declaration,
            includes,
            &|| false,
        )
        .map(|refs| refs.expect("uncancelled references_at query should not return None"))
    }

    /// Cancellable variant of [`Self::references_at`].
    ///
    /// Returns `Err(NotFound)` when no symbol exists at the cursor. Returns
    /// `Ok(None)` when `should_cancel` aborts the underlying indexed query.
    #[allow(clippy::type_complexity)]
    pub fn references_at_cancellable(
        &mut self,
        file: &str,
        byte_offset: u32,
        files: &[Arc<str>],
        include_declaration: bool,
        includes: crate::ReferenceIncludes,
        should_cancel: &(dyn Fn() -> bool + Sync),
    ) -> Result<Option<Vec<(Arc<str>, crate::Range)>>, crate::SymbolLookupError> {
        let name = self
            .name_at(file, byte_offset)
            .ok_or(crate::SymbolLookupError::NotFound)?;
        Ok(self.indexed_references_to(&name, files, include_declaration, includes, should_cancel))
    }

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
        &mut self,
        symbol: &crate::Name,
    ) -> Result<mir_types::Location, crate::SymbolLookupError> {
        self.load_symbol_owner(symbol);
        self.definition_of_cached(symbol)
    }

    /// Lazy-load the class (or function) `symbol` is rooted in, so a pure
    /// lookup against a following snapshot finds it. No-op when loaded.
    fn load_symbol_owner(&mut self, symbol: &crate::Name) {
        match symbol {
            crate::Name::Class(fqn) | crate::Name::Function(fqn) => {
                let _ = self.load_class(fqn.as_ref());
            }
            crate::Name::Method { class, .. }
            | crate::Name::Property { class, .. }
            | crate::Name::ClassConstant { class, .. } => {
                let _ = self.load_class(class.as_ref());
            }
            crate::Name::GlobalConstant(_) => {}
        }
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
        self.retry_snapshot(|snap| snap.definition_of_cached(symbol))
    }

    /// Hover information for a symbol: type, docstring, and definition location.
    ///
    /// For cursor-based editor navigation, prefer [`Self::hover_at`], which
    /// uses the targeted [`Self::resolve_at`] path instead of requiring a
    /// retained whole-file symbol list. This method assembles hover data once
    /// the caller already has a typed [`crate::Name`].
    ///
    /// **Side effects:** when `symbol`'s owning class isn't yet loaded, this
    /// may invoke the configured [`crate::SourceProvider`] to fault in
    /// dependencies. Use [`Self::hover_cached`] for a pure variant.
    ///
    /// Returns `Err(NotFound)` if the symbol doesn't exist. May still return
    /// `Ok` with `docstring: None` or `definition: None` if those specific
    /// pieces aren't available.
    pub fn hover(
        &mut self,
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
        self.retry_snapshot(|snap| snap.hover_cached(symbol))
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
    pub fn subtype_files(&mut self, class_fqn: &str) -> Vec<Arc<str>> {
        self.prepare_for_query(None);
        self.retry_snapshot(|snap| snap.subtype_files(class_fqn))
    }

    /// Compatibility wrapper for callers that only want `use` import items.
    /// Delegates to [`Self::indexed_references_to`] so import references share
    /// the same freshness/self-heal path, scope filtering, and memoization
    /// boundary as every other reference query.
    pub fn indexed_use_import_locations(
        &mut self,
        symbol: &crate::Name,
        files: &[Arc<str>],
    ) -> Vec<(Arc<str>, crate::Range)> {
        self.indexed_references_to(
            symbol,
            files,
            false,
            crate::ReferenceIncludes::UseImports,
            &|| false,
        )
        .unwrap_or_default()
    }

    /// See [`AnalysisSnapshot::indexed_references_to`]. Unlike the snapshot
    /// form, this runs the owner-side warm-up for stale candidates first, so
    /// every one of them is analyzed with its referenced classes loaded.
    ///
    /// `should_cancel` is polled at phase boundaries and between
    /// cancellation retries; `true` aborts with `None`.
    pub fn indexed_references_to(
        &mut self,
        symbol: &crate::Name,
        files: &[Arc<str>],
        include_declaration: bool,
        includes: crate::ReferenceIncludes,
        should_cancel: &(dyn Fn() -> bool + Sync),
    ) -> Option<Vec<(Arc<str>, crate::Range)>> {
        // No `should_cancel()` check before the memo probe: a hit does no
        // analysis work, and some callers count probe invocations to prove
        // a warm query needed no re-analysis.
        let key = {
            let snap = self.db_view();
            let key = snap.reference_query_key(symbol, files, include_declaration, includes);
            if let Some(hit) = self.index.ref_queries.get(&key) {
                return Some(hit);
            }
            key
        };
        let stale = loop {
            if should_cancel() {
                return None;
            }
            let snap = self.db_view();
            match catch(|| snap.stale_reference_candidates(symbol, files)) {
                Ok(stale) => break stale,
                Err(_) if should_cancel() => return None,
                Err(_) => {}
            }
        };

        if !stale.is_empty() {
            // Cached postings for fresh candidates don't need the pending
            // symbol-index work; stale ones are about to analyze against the
            // workspace, so reconcile first.
            if !self.settle_workspace_index_cancellable(should_cancel) {
                return None;
            }
            // Serial; a cancelled file retries in place. Parallel variants
            // deadlocked under `concurrent_reference_cancel`. The bump scope
            // closes before the commit snapshot captures its generation.
            {
                let mut session = self.defer_revision_bumps();
                for path in &stale {
                    loop {
                        if should_cancel() {
                            return None;
                        }
                        match catch(|| session.prepare_file_for_analysis(path)) {
                            Ok(()) => break,
                            Err(_) if should_cancel() => return None,
                            Err(_) => {}
                        }
                    }
                }
            }
            loop {
                if should_cancel() {
                    return None;
                }
                let snap = self.db_view();
                match snap.commit_reference_candidates(&stale) {
                    Ok(()) => break,
                    Err(_) if should_cancel() => return None,
                    Err(_) => {}
                }
            }
        }

        if include_declaration {
            self.load_symbol_owner(symbol);
        }
        loop {
            let snap = self.db_view();
            match catch(|| snap.read_references(symbol, files, include_declaration, includes)) {
                Ok(out) => {
                    snap.memoize_references(key, &out);
                    return Some(out);
                }
                Err(_) if should_cancel() => return None,
                Err(_) => {}
            }
        }
    }

    /// The symbol's declaration site, narrowed from the collector's
    /// whole-declaration span to the declared name's own token (matching the
    /// span shape of recorded references).
    pub fn declaration_name_range(
        &mut self,
        symbol: &crate::Name,
    ) -> Option<(Arc<str>, crate::Range)> {
        self.load_symbol_owner(symbol);
        self.retry_snapshot(|snap| catch(|| snap.declaration_name_range(symbol)))
    }

    /// See [`AnalysisSnapshot::indexed_subtype_classes`].
    pub fn indexed_subtype_classes(
        &mut self,
        class_fqn: &str,
        files: &[Arc<str>],
        include_trait_users: bool,
    ) -> Vec<SubtypeClassSite> {
        self.prepare_for_query(None);
        self.retry_snapshot(|snap| {
            snap.indexed_subtype_classes(class_fqn, files, include_trait_users)
        })
    }

    /// See [`AnalysisSnapshot::indexed_method_implementations`].
    pub fn indexed_method_implementations(
        &mut self,
        class_fqn: &str,
        method: &str,
        files: &[Arc<str>],
    ) -> Vec<(Arc<str>, Arc<str>, crate::Range)> {
        self.prepare_for_query(None);
        self.retry_snapshot(|snap| snap.indexed_method_implementations(class_fqn, method, files))
    }

    /// See [`AnalysisSnapshot::class_issues`]. Call this after ingesting or
    /// re-analyzing a file and its dependents to get the full diagnostic
    /// picture.
    pub fn class_issues(&mut self, files: &[Arc<str>]) -> Vec<crate::Issue> {
        self.prepare_for_query(None);
        self.retry_snapshot(|snap| snap.class_issues(files))
    }

    /// See [`AnalysisSnapshot::collector_issues`]. Correct regardless of
    /// which path put the file's text into the db (`ingest_file`,
    /// `set_file_text`, lazy vendor load) or how many times.
    pub fn collector_issues(&self, files: &[Arc<str>]) -> Vec<crate::Issue> {
        self.retry_snapshot(|snap| snap.collector_issues(files))
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
pub(super) fn span_range(line: u32, col_start: u32, col_end: u32) -> crate::Range {
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
pub(super) fn identifier_char_col(
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
/// boundaries; conservative near multibyte text). ASCII-case-insensitive:
/// PHP class, function, and method names are case-insensitive, so `new
/// COLOR()` must count as mentioning `Color`; for the case-sensitive kinds
/// (constants, properties) folding only widens the candidate superset.
///
/// Test-only semantic oracle: the production gates (references freshness,
/// subtype-BFS defs commit) answer this predicate through the persistent
/// `ClassMentionIndex`; the parity test below pins the scanner to these
/// exact boundary and case semantics.
#[cfg(test)]
fn mentions_identifier(hay: &str, needle: &str) -> bool {
    let hay = hay.as_bytes();
    let needle = needle.as_bytes();
    let n = needle.len();
    if n == 0 || hay.len() < n {
        return false;
    }
    let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let first = needle[0].to_ascii_lowercase();
    for i in 0..=(hay.len() - n) {
        if hay[i].to_ascii_lowercase() != first || !hay[i..i + n].eq_ignore_ascii_case(needle) {
            continue;
        }
        if (i == 0 || !is_ident(hay[i - 1])) && (i + n == hay.len() || !is_ident(hay[i + n])) {
            return true;
        }
    }
    false
}

fn short(fqn: &str) -> &str {
    fqn.rsplit('\\').next().unwrap_or(fqn)
}

/// A candidate-file admission predicate for `indexed_references_to`'s
/// freshness pass, chosen by [`ReferenceGate::for_symbol`]. A file is
/// admitted when its text mentions any of `idents` as a whole identifier
/// (word-bounded, ASCII-case-insensitive) OR contains any of `raw` as a
/// plain substring (ASCII-case-insensitive, no word bounds — used for
/// call-shaped tokens like `->__construct`). A file matching neither can
/// hold no posting for the symbol.
pub(super) struct ReferenceGate {
    pub(super) idents: Vec<String>,
    pub(super) raw: Vec<String>,
}

impl ReferenceGate {
    /// Choose the candidate-admission gate for `symbol`.
    ///
    /// For any known non-constructor/non-`__invoke` method, the member name
    /// alone is the sound gate. Every posting-producing reference spells that
    /// token: `$obj->m()`, `Owner::m()`, inherited `Sub::m()`,
    /// `self::`/`static::`/`parent::m()`, callable strings/arrays, and trait
    /// aliases (`orig as alias` records `orig`, alias calls record `alias`
    /// plus the origin key). Dropping the owner short name matters on common
    /// owner names (`User`, `Model`, `Widget`): otherwise a cold reference
    /// query analyzes files that only type-hint the owner and cannot contain a
    /// reference to the queried method. Dynamic member names (`$obj->$m()`)
    /// produce no posting, so nothing is lost there.
    ///
    /// For `__construct` with a known owner, the identifier needle is the
    /// owner's short name (`new Cls(` sites never spell the member name and
    /// the bare word `__construct` would admit every file *declaring* a
    /// constructor), complemented by the raw call tokens `->__construct` /
    /// `::__construct`: an explicit re-init `$obj->__construct()` is a real
    /// recorded reference whose file may never name the class.
    ///
    /// `__invoke` keeps the general owner/name gate because `$obj()` call sites
    /// do not spell `__invoke`.
    ///
    /// Everything else uses the general OR needles
    /// ([`reference_gate_needles`]).
    pub(super) fn for_symbol(db: &MirDbStorage, symbol: &crate::Name) -> Self {
        if let crate::Name::Method { class, name } = symbol {
            if name.as_ref() == "__construct" && !class.is_empty() {
                return Self {
                    idents: reference_gate_needles(symbol),
                    raw: vec!["->__construct".to_string(), "::__construct".to_string()],
                };
            }
            if name.as_ref() != "__construct" && name.as_ref() != "__invoke" && !class.is_empty() {
                let here = crate::db::Fqcn::from_str(db, class.as_ref());
                let is_static = crate::db::find_method_in_chain(db, here, name)
                    .map(|(_, m)| m.is_static)
                    .unwrap_or(false);
                if is_static || crate::db::class_exists(db, class.as_ref()) {
                    return Self {
                        idents: vec![name.to_string()],
                        raw: Vec::new(),
                    };
                }
            }
        }
        Self {
            idents: reference_gate_needles(symbol),
            raw: Vec::new(),
        }
    }
}

/// Identifier words whose whole-word presence in a file's text is necessary
/// for the file to hold any posting [`AnalysisSession::indexed_references_to`]
/// can return for `symbol`. Member symbols include the owner class's short
/// name alongside the member name: `__construct` postings are recorded at
/// `new Cls(` sites, which never spell the member name.
fn reference_gate_needles(symbol: &crate::Name) -> Vec<String> {
    let mut needles = match symbol {
        crate::Name::Class(f) | crate::Name::Function(f) | crate::Name::GlobalConstant(f) => {
            vec![short(f).to_string()]
        }
        // `__construct` is invoked only as `new Cls(...)`, `parent::__construct()`,
        // or `self::__construct()`/`static::__construct()` from inside a
        // subclass — every real call site textually names the class itself
        // (directly, or via the enclosing subclass's own `extends`/`use`),
        // never the bare word `__construct`. Gating on the class's short name
        // alone is exact (no lost call sites) and, unlike the general member
        // case, dropping the method-name needle here doesn't reintroduce a
        // false negative. This matters: `__construct` is one of the most
        // common tokens in any real codebase, so OR-ing it in as a needle
        // admits nearly every file as a "must re-analyze" candidate on a
        // cold query, defeating the gate's entire purpose for constructors.
        crate::Name::Method { class, name } if name.as_ref() == "__construct" => {
            if class.is_empty() {
                // No class to scope to (owner unknown) — fall back to gating
                // on the bare name, same as the general member case below.
                vec![name.to_string()]
            } else {
                vec![short(class).to_string()]
            }
        }
        crate::Name::Method { class, name }
        | crate::Name::Property { class, name }
        | crate::Name::ClassConstant { class, name } => {
            let mut v = vec![name.to_string()];
            if !class.is_empty() {
                v.push(short(class).to_string());
            }
            v
        }
    };
    // An empty needle can never match; dropping it keeps the "empty needle
    // set disables the gate" contract at the call site conservative.
    needles.retain(|n| !n.is_empty());
    needles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mentions_identifier_is_case_insensitive_and_word_bounded() {
        assert!(mentions_identifier("$this->save();", "save"));
        assert!(mentions_identifier("new COLOR()", "Color"));
        assert!(mentions_identifier("use App\\Color as Paint;", "color"));
        assert!(!mentions_identifier("$this->saveAll();", "save"));
        assert!(!mentions_identifier("return $unsaved;", "save"));
        assert!(!mentions_identifier("no occurrence", "save"));
        assert!(!mentions_identifier("anything", ""));
        // Multibyte neighbors are conservatively treated as boundaries, and
        // substring scans must not split codepoints.
        assert!(!mentions_identifier("function xÉclairFoo() {}", "Éclair"));
        assert!(mentions_identifier("implements Éclair {}", "Éclair"));
    }

    #[test]
    fn mention_scanner_membership_equals_per_needle_scans() {
        // The mention index is the sole implementation of the gates' textual
        // predicate, so scanner membership must equal the reference
        // per-needle predicate for every (hay, needle) pair — same boundary
        // and case semantics.
        use crate::db::MentionScanner;
        use std::sync::Arc;
        let universe = ["Color", "save", "ColorPicker", "Éclair", "C1", "_Wrap"];
        let names: Vec<(mir_types::Name, bool)> = universe
            .iter()
            .map(|s| (mir_types::Name::new(s).ascii_lowercase(), false))
            .collect();
        let scanner = Arc::new(MentionScanner::build(1, names).unwrap());
        let hays = [
            "$this->save();",
            "new COLOR()",
            "use App\\Color as Paint;",
            "$this->saveAll();",
            "return $unsaved;",
            "new ColorPicker(); Color::save();",
            "function xÉclairFoo() {}",
            "implements Éclair {}",
            "colorsave savecolor color_save",
            "class C1 extends _Wrap {}",
            "",
        ];
        for hay in hays {
            let scanned = scanner.scan(hay);
            for needle in universe {
                let expected = mentions_identifier(hay, needle);
                let name = mir_types::Name::new(needle).ascii_lowercase();
                assert_eq!(
                    scanned.binary_search(&name).is_ok(),
                    expected,
                    "needle {needle:?} on {hay:?}"
                );
            }
        }
    }

    #[test]
    fn gate_needles_cover_member_and_owner_class() {
        // A regular member (non-constructor) gates on both the member name
        // and the owner's short name — a call site may name only one.
        let n = reference_gate_needles(&crate::Name::method("App\\Job", "run"));
        assert!(n.contains(&"run".to_string()) && n.contains(&"Job".to_string()));
        let n = reference_gate_needles(&crate::Name::class("App\\Ui\\Color"));
        assert_eq!(n, vec!["Color".to_string()]);
        // Unknown-owner member symbols still gate on the member name alone.
        let n = reference_gate_needles(&crate::Name::method("", "run"));
        assert_eq!(n, vec!["run".to_string()]);
    }

    #[test]
    fn gate_needles_for_constructor_scope_to_owner_class_only() {
        // `__construct` is only ever spelled at `new Cls(`/`parent::__construct()`
        // sites, which always name the class — the bare method-name needle is
        // dropped so a cold constructor query doesn't admit nearly every file
        // in the workspace (every class defines *some* `__construct`).
        let n = reference_gate_needles(&crate::Name::method("App\\Job", "__construct"));
        assert_eq!(n, vec!["Job".to_string()]);
        // Unknown owner: nothing to scope to, fall back to the bare name.
        let n = reference_gate_needles(&crate::Name::method("", "__construct"));
        assert_eq!(n, vec!["__construct".to_string()]);
    }

    fn session_with(files: &[(&str, &str)]) -> crate::AnalysisSession {
        let mut session = crate::AnalysisSession::new(crate::PhpVersion::LATEST);
        for (path, text) in files {
            session.set_file_text(Arc::from(*path), Arc::from(*text));
        }
        session
    }

    #[test]
    fn gate_static_method_is_member_name_only() {
        // Regardless of subtypes: the member token alone is the sound gate
        // (an instance receiver `$obj::m()` never names the owner), and it
        // is also the selective part — the owner short name would only
        // widen the admitted set.
        let session = session_with(&[
            (
                "owner.php",
                "<?php\nclass Owner { public static function m(): void {} }\n",
            ),
            ("sub.php", "<?php\nclass Sub extends Owner {}\n"),
        ]);
        let gate = session
            .snapshot()
            .reference_gate(&crate::Name::method("Owner", "m"));
        assert_eq!(gate.idents, vec!["m".to_string()]);
        assert!(gate.raw.is_empty());
    }

    #[test]
    fn gate_instance_method_is_member_name_only() {
        let session = session_with(&[(
            "owner.php",
            "<?php\nclass Owner { public function m(): void {} }\n",
        )]);
        let gate = session
            .snapshot()
            .reference_gate(&crate::Name::method("Owner", "m"));
        assert_eq!(gate.idents, vec!["m".to_string()]);
        assert!(gate.raw.is_empty());
    }

    #[test]
    fn gate_invoke_keeps_owner_needle() {
        let session = session_with(&[(
            "owner.php",
            "<?php\nclass Owner { public function __invoke(): void {} }\n",
        )]);
        let gate = session
            .snapshot()
            .reference_gate(&crate::Name::method("Owner", "__invoke"));
        assert_eq!(
            gate.idents,
            reference_gate_needles(&crate::Name::method("Owner", "__invoke"))
        );
        assert!(gate.raw.is_empty());
    }

    #[test]
    fn gate_constructor_adds_raw_call_tokens() {
        // Owner short name for `new Cls(` sites, plus the raw call tokens
        // for explicit re-init (`$obj->__construct()`) whose file may never
        // name the class. The bare identifier `__construct` must NOT be a
        // needle — it would admit every file declaring a constructor.
        let session = session_with(&[(
            "owner.php",
            "<?php\nclass Owner { public function __construct() {} }\n",
        )]);
        let gate = session
            .snapshot()
            .reference_gate(&crate::Name::method("Owner", "__construct"));
        assert_eq!(
            gate.idents,
            reference_gate_needles(&crate::Name::method("Owner", "__construct"))
        );
        assert_eq!(
            gate.raw,
            vec!["->__construct".to_string(), "::__construct".to_string()]
        );
    }

    #[test]
    fn gate_unresolvable_owner_falls_back_to_general_needles() {
        let session = session_with(&[(
            "owner.php",
            "<?php\nclass Owner { public static function m(): void {} }\n",
        )]);
        let gate = session
            .snapshot()
            .reference_gate(&crate::Name::method("Nonexistent\\Missing", "m"));
        assert_eq!(
            gate.idents,
            reference_gate_needles(&crate::Name::method("Nonexistent\\Missing", "m"))
        );
        assert!(gate.raw.is_empty());
    }
}
