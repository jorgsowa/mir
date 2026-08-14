//! `AnalysisSession::warm_start_files`: replaying disk-cached reference
//! locations and subtype-index class edges at session start, so a returning
//! session doesn't pay the on-demand analysis sweep the first time each file
//! is queried.

mod common;

use std::sync::Arc;

use mir_analyzer::cache::{hash_content, AnalysisCache};
use mir_analyzer::{AnalysisSession, Name, PhpVersion};

use self::common::create_temp_dir;

#[test]
fn warm_start_files_replays_reference_locations_from_disk_cache() {
    let dir = create_temp_dir("warm_start_ref_locs");
    let php_v = PhpVersion::LATEST.cache_byte();
    // An empty class: real analysis of this text produces no method-call
    // postings at all.
    let text = "<?php\nclass Widget {}\n";
    let file_path = "widget.php";

    {
        let disk_cache = AnalysisCache::open(dir.path(), php_v, 0);
        let hash = hash_content(text);
        // A posting that live analysis of `text` could never produce —
        // finding it after `warm_start_files` proves it came from disk-cache
        // replay, not a live re-analysis (which would also have overwritten
        // it, since postings are committed with replace-per-file semantics).
        let fabricated: Arc<[mir_analyzer::cache::CachedRefLoc]> =
            Arc::from(vec![(Arc::from("meth:App\\Other::bogus"), 5, 0, 5)]);
        disk_cache.put(
            file_path,
            hash,
            String::new(),
            Arc::from(Vec::new()),
            fabricated,
        );
        disk_cache.flush();
    }

    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
    session.ensure_all_stubs();
    session.warm_start_files(&[(Arc::from(file_path), Arc::from(text))]);

    let refs = session
        .indexed_references_to(
            &Name::method("App\\Other", "bogus"),
            &[Arc::from(file_path)],
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert_eq!(
        refs.len(),
        1,
        "fabricated posting must be visible right after warm_start_files: {refs:?}"
    );
    assert_eq!(refs[0].0.as_ref(), file_path);
}

#[test]
fn warm_start_files_replays_subtype_edges_from_disk_definition_cache() {
    let dir = create_temp_dir("warm_start_subtype_edges");
    let impl_path = "impl.php";
    let impl_text = "<?php\nnamespace Vendor;\nclass Impl implements \\Shop\\Shape {}\n";

    // A real ingest in an earlier "session" (simulated by a throwaway
    // AnalysisSession) populates the on-disk StubSlice definition cache for
    // impl.php: `ingest_file` -> `collect_and_ingest_file` writes it on a miss.
    {
        let seed = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
        seed.ensure_all_stubs();
        seed.ingest_file(Arc::from(impl_path), Arc::from(impl_text));
    }

    // A fresh session against the same cache dir. Never runs definition
    // collection on impl.php itself — only `warm_start_files`.
    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
    session.ensure_all_stubs();
    session.warm_start_files(&[(Arc::from(impl_path), Arc::from(impl_text))]);

    // Query scope deliberately excludes impl.php: `indexed_subtype_classes`'s
    // on-demand self-heal (`commit_defs_for_matching`) only ever looks at
    // files in this list, so `Impl` can only appear here because
    // `warm_start_files` itself committed its class edges from the disk
    // definition cache.
    let subs = session.indexed_subtype_classes("Shop\\Shape", &[], false);
    let fqcns: Vec<&str> = subs.iter().map(|s| s.fqcn.as_ref()).collect();
    assert!(
        fqcns.contains(&"Vendor\\Impl"),
        "subtype edges must be replayed from the disk definition cache: {fqcns:?}"
    );
}

/// A warm-started file whose cached issue set shows full resolution keeps
/// its replayed postings across workspace growth: registrations and lazy
/// loads that follow warm-up must not force a re-analysis of the replayed
/// set. The fabricated posting can only survive if no live re-analysis
/// (replace semantics) ever ran.
#[test]
fn warm_start_replay_survives_workspace_growth_when_resolved() {
    let dir = create_temp_dir("warm_start_immune");
    let php_v = PhpVersion::LATEST.cache_byte();
    let text = "<?php\nclass Widget {}\n";
    let file_path = "widget.php";

    {
        let disk_cache = AnalysisCache::open(dir.path(), php_v, 0);
        let fabricated: Arc<[mir_analyzer::cache::CachedRefLoc]> =
            Arc::from(vec![(Arc::from("meth:App\\Other::bogus"), 5, 0, 5)]);
        // Empty issue set: the previous run resolved everything.
        disk_cache.put(
            file_path,
            hash_content(text),
            String::new(),
            Arc::from(Vec::new()),
            fabricated,
        );
        disk_cache.flush();
    }

    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
    session.ensure_all_stubs();
    session.warm_start_files(&[(Arc::from(file_path), Arc::from(text))]);

    // Workspace grows after warm-up — the pattern background indexing and
    // lazy vendor loads produce in an LSP session.
    session.ingest_file(Arc::from("later.php"), Arc::from("<?php\nclass Later {}\n"));

    let refs = session
        .indexed_references_to(
            &Name::method("App\\Other", "bogus"),
            &[Arc::from(file_path)],
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert_eq!(
        refs.len(),
        1,
        "a fully-resolved replay must survive the growth bump: {refs:?}"
    );
}

/// The counterpart: a replay whose cached issues include an unresolved name
/// is re-verified once the workspace grows — live analysis of the empty
/// class overwrites the fabricated posting.
#[test]
fn warm_start_replay_reverifies_unresolved_files_after_growth() {
    let dir = create_temp_dir("warm_start_reverify");
    let php_v = PhpVersion::LATEST.cache_byte();
    let text = "<?php\nclass Widget {}\n";
    let file_path = "widget.php";

    {
        let disk_cache = AnalysisCache::open(dir.path(), php_v, 0);
        let fabricated: Arc<[mir_analyzer::cache::CachedRefLoc]> =
            Arc::from(vec![(Arc::from("meth:App\\Other::bogus"), 5, 0, 5)]);
        let unresolved = mir_issues::Issue::new(
            mir_issues::IssueKind::UndefinedClass {
                name: "App\\Other".into(),
            },
            mir_issues::Location::new(Arc::from(file_path), 5, 5, 0, 5),
        );
        disk_cache.put(
            file_path,
            hash_content(text),
            String::new(),
            Arc::from(vec![unresolved]),
            fabricated,
        );
        disk_cache.flush();
    }

    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
    session.ensure_all_stubs();
    let unresolved = session.warm_start_files(&[(Arc::from(file_path), Arc::from(text))]);
    assert_eq!(
        unresolved,
        vec![Arc::<str>::from(file_path)],
        "warm_start_files must report this file so a caller can proactively \
         re-analyze it off the request path: {unresolved:?}"
    );

    session.ingest_file(Arc::from("later.php"), Arc::from("<?php\nclass Later {}\n"));

    let refs = session
        .indexed_references_to(
            &Name::method("App\\Other", "bogus"),
            &[Arc::from(file_path)],
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert!(
        refs.is_empty(),
        "an unresolved replay must be re-verified after growth, not served \
         from the stale disk-cache posting: {refs:?}"
    );
}

/// [`AnalysisSession::warm_start_files`]'s return value distinguishes
/// resolved from unresolved replays across a mixed batch, and excludes files
/// with no cache hit at all (a first-ever boot, where nothing was replayed).
#[test]
fn warm_start_files_returns_only_unresolved_replays() {
    let dir = create_temp_dir("warm_start_return_value");
    let php_v = PhpVersion::LATEST.cache_byte();
    let resolved_text = "<?php\nclass Resolved {}\n";
    let unresolved_text = "<?php\nclass Unresolved {}\n";
    let uncached_text = "<?php\nclass Uncached {}\n";

    {
        let disk_cache = AnalysisCache::open(dir.path(), php_v, 0);
        disk_cache.put(
            "resolved.php",
            hash_content(resolved_text),
            String::new(),
            Arc::from(Vec::new()),
            Arc::from(Vec::new()),
        );
        let unresolved_issue = mir_issues::Issue::new(
            mir_issues::IssueKind::UndefinedClass {
                name: "App\\Other".into(),
            },
            mir_issues::Location::new(Arc::from("unresolved.php"), 1, 1, 0, 1),
        );
        disk_cache.put(
            "unresolved.php",
            hash_content(unresolved_text),
            String::new(),
            Arc::from(vec![unresolved_issue]),
            Arc::from(Vec::new()),
        );
        disk_cache.flush();
    }

    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
    session.ensure_all_stubs();
    let unresolved = session.warm_start_files(&[
        (Arc::from("resolved.php"), Arc::from(resolved_text)),
        (Arc::from("unresolved.php"), Arc::from(unresolved_text)),
        (Arc::from("uncached.php"), Arc::from(uncached_text)),
    ]);

    assert_eq!(
        unresolved,
        vec![Arc::<str>::from("unresolved.php")],
        "must include exactly the replayed-but-unresolved file, excluding both \
         the resolved replay and the never-cached file: {unresolved:?}"
    );
}

/// End-to-end round trip for the LSP session paths: postings committed by a
/// session's analysis sweep are persisted via `flush_analysis_cache`, and a
/// fresh session against the same cache dir answers a references query from
/// `warm_start_files` replay with no analysis sweep. The cancel probe flips
/// to `true` after the first consultation (the stale-set computation), so the
/// query can only complete if replay left nothing stale.
#[test]
fn session_sweep_persists_postings_for_next_launch() {
    let dir = create_temp_dir("sweep_persists_postings");
    let widget_path = "widget.php";
    let widget_text = "<?php\nclass Widget { public function spin(): void {} }\n";
    let caller_path = "caller.php";
    let caller_text = "<?php\n$w = new Widget();\n$w->spin();\n";

    {
        let seed = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
        seed.ensure_all_stubs();
        seed.ingest_file(Arc::from(widget_path), Arc::from(widget_text));
        seed.ingest_file(Arc::from(caller_path), Arc::from(caller_text));
        seed.reanalyze_files_cancellable(
            &[Arc::from(widget_path), Arc::from(caller_path)],
            &mir_analyzer::IndexCancel::new(),
        );
        seed.flush_analysis_cache();
    }

    // The write hook itself: the sweep must have stored caller.php's postings
    // keyed by its content hash.
    {
        let php_v = PhpVersion::LATEST.cache_byte();
        let disk_cache = AnalysisCache::open(dir.path(), php_v, 0);
        let (_, ref_locs) = disk_cache
            .get(caller_path, &hash_content(caller_text))
            .expect("sweep must persist an AnalysisCache entry for caller.php");
        assert!(
            !ref_locs.is_empty(),
            "persisted entry must carry the file's reference postings"
        );
    }

    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
    session.ensure_all_stubs();
    session.warm_start_files(&[
        (Arc::from(widget_path), Arc::from(widget_text)),
        (Arc::from(caller_path), Arc::from(caller_text)),
    ]);

    let consultations = std::sync::atomic::AtomicU32::new(0);
    let cancel_after_first =
        || consultations.fetch_add(1, std::sync::atomic::Ordering::Relaxed) >= 1;
    let refs = session
        .indexed_references_to(
            &Name::method("Widget", "spin"),
            &[Arc::from(widget_path), Arc::from(caller_path)],
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &cancel_after_first,
        )
        .expect("replayed postings must answer the query with no analysis sweep");
    assert_eq!(refs.len(), 1, "the $w->spin() call site: {refs:?}");
    assert_eq!(refs[0].0.as_ref(), caller_path);
}

/// Same round trip through the other commit site: the on-demand freshness
/// pass inside `indexed_references_to` (a query racing ahead of any sweep)
/// must persist what it commits.
#[test]
fn on_demand_query_commit_persists_postings_for_next_launch() {
    let dir = create_temp_dir("on_demand_persists_postings");
    let widget_path = "widget.php";
    let widget_text = "<?php\nclass Widget { public function spin(): void {} }\n";
    let caller_path = "caller.php";
    let caller_text = "<?php\n$w = new Widget();\n$w->spin();\n";

    {
        let seed = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
        seed.ensure_all_stubs();
        seed.ingest_file(Arc::from(widget_path), Arc::from(widget_text));
        seed.ingest_file(Arc::from(caller_path), Arc::from(caller_text));
        // No sweep: the query's own freshness pass analyzes and commits.
        let refs = seed
            .indexed_references_to(
                &Name::method("Widget", "spin"),
                &[Arc::from(widget_path), Arc::from(caller_path)],
                false,
                mir_analyzer::ReferenceIncludes::Plain,
                &|| false,
            )
            .expect("not cancelled");
        assert_eq!(refs.len(), 1, "sanity: live query finds the call site");
        seed.flush_analysis_cache();
    }

    let php_v = PhpVersion::LATEST.cache_byte();
    let disk_cache = AnalysisCache::open(dir.path(), php_v, 0);
    let (_, ref_locs) = disk_cache
        .get(caller_path, &hash_content(caller_text))
        .expect("on-demand commit must persist an AnalysisCache entry");
    assert!(!ref_locs.is_empty());
}

/// An entry already valid for the file's current content (e.g. written by the
/// CLI batch pipeline, which records a surface hash) is left untouched by the
/// session write hook — the fabricated postings prove no overwrite happened.
#[test]
fn session_sweep_does_not_clobber_valid_batch_entries() {
    let dir = create_temp_dir("sweep_no_clobber");
    let php_v = PhpVersion::LATEST.cache_byte();
    let file_path = "widget.php";
    let text = "<?php\nclass Widget {}\n";

    {
        let disk_cache = AnalysisCache::open(dir.path(), php_v, 0);
        let fabricated: Arc<[mir_analyzer::cache::CachedRefLoc]> =
            Arc::from(vec![(Arc::from("meth:App\\Other::bogus"), 5, 0, 5)]);
        disk_cache.put(
            file_path,
            hash_content(text),
            "batch-surface".to_string(),
            Arc::from(Vec::new()),
            fabricated,
        );
        disk_cache.flush();
    }

    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
    session.ensure_all_stubs();
    session.ingest_file(Arc::from(file_path), Arc::from(text));
    session.reanalyze_files_cancellable(&[Arc::from(file_path)], &mir_analyzer::IndexCancel::new());
    session.flush_analysis_cache();

    let disk_cache = AnalysisCache::open(dir.path(), php_v, 0);
    let (_, ref_locs) = disk_cache
        .get(file_path, &hash_content(text))
        .expect("entry must still exist");
    assert_eq!(
        ref_locs.len(),
        1,
        "a content-valid entry must not be overwritten by the session sweep"
    );
    assert_eq!(
        disk_cache.surface_hash(file_path).as_deref(),
        Some("batch-surface"),
        "batch-written surface hash must survive"
    );
}

#[test]
fn warm_start_files_is_a_no_op_without_a_cache() {
    // No `with_cache`/`with_cache_dir` attached — must not panic, and must
    // leave the file queryable (falling through to the normal lazy path).
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    let file_path = "plain.php";
    let text = "<?php\nclass Plain {}\n";
    session.warm_start_files(&[(Arc::from(file_path), Arc::from(text))]);

    let refs = session
        .indexed_references_to(
            &Name::class("Plain"),
            &[Arc::from(file_path)],
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert!(refs.is_empty(), "no reference sites expected: {refs:?}");
}

// ---------------------------------------------------------------------------
// Symbol-index singleton seeding (mir 0.64): warm_start_files projects
// declarations from the disk slices it already reads for edge replay and
// seeds the singleton, so a returning session's first query never runs the
// tracked O(all-files) workspace_symbol_index walk.
// ---------------------------------------------------------------------------

/// Simulated "session 1": ingest files against a cache dir so their
/// StubSlices land on disk, then drop the session.
fn seed_disk_caches(dir: &std::path::Path, files: &[(&str, &str)]) {
    let seed = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir);
    seed.ensure_all_stubs();
    for (path, text) in files {
        seed.ingest_file(Arc::from(*path), Arc::from(*text));
    }
    seed.flush_analysis_cache();
}

#[test]
fn warm_start_seeds_workspace_symbol_index_without_tracked_walk() {
    let dir = create_temp_dir("warm_start_seed_index");
    let files = [
        (
            "svc.php",
            "<?php\nnamespace App;\nclass Service { public static function run(): void {} }\n",
        ),
        (
            "caller.php",
            "<?php\nnamespace App;\nclass Caller { public function go(): void { Service::run(); } }\n",
        ),
    ];
    seed_disk_caches(dir.path(), &files);

    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
    let warm: Vec<(Arc<str>, Arc<str>)> = files
        .iter()
        .map(|(p, t)| (Arc::from(*p), Arc::from(*t)))
        .collect();
    session.warm_start_files(&warm);

    assert!(
        session.workspace_symbol_index_ready(),
        "warm_start over fully slice-covered files must seed the singleton"
    );
    let walks_after_seed = session.workspace_index_walks();

    let candidates: Vec<Arc<str>> = files.iter().map(|(p, _)| Arc::from(*p)).collect();
    let refs = session
        .indexed_references_to(
            &Name::method("App\\Service", "run"),
            &candidates,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert_eq!(refs.len(), 1, "must find the static call site: {refs:?}");
    assert_eq!(refs[0].0.as_ref(), "caller.php");
    assert_eq!(
        session.workspace_index_walks(),
        walks_after_seed,
        "a seeded session's query must not run the tracked O(all-files) walk"
    );
}

#[test]
fn warm_start_seed_skipped_when_slices_missing() {
    // Fresh cache dir: no slices on disk, so the coverage gap is the whole
    // workspace and seeding must be skipped (first-ever boot keeps the lazy
    // behavior; the background sweep owns the parse bill).
    let dir = create_temp_dir("warm_start_seed_skipped");
    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
    let files: Vec<(Arc<str>, Arc<str>)> = (0..8)
        .map(|i| {
            (
                Arc::from(format!("f{i}.php").as_str()),
                Arc::from(format!("<?php\nclass C{i} {{}}\n").as_str()),
            )
        })
        .collect();
    session.warm_start_files(&files);
    assert!(
        !session.workspace_symbol_index_ready(),
        "no slice coverage -> no seed"
    );
    // Queries still work through the tracked fallback.
    let refs = session
        .indexed_references_to(
            &Name::class("C3"),
            &files.iter().map(|(p, _)| p.clone()).collect::<Vec<_>>(),
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert!(refs.is_empty(), "{refs:?}");
}

#[test]
fn mirror_only_new_file_is_visible_after_settle() {
    let dir = create_temp_dir("warm_start_settle_new_file");
    let files = [(
        "base.php",
        "<?php\nnamespace App;\nclass Base { public static function ping(): void {} }\n",
    )];
    seed_disk_caches(dir.path(), &files);

    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
    session.warm_start_files(&[(Arc::from("base.php"), Arc::from(files[0].1))]);
    assert!(session.workspace_symbol_index_ready());

    // A new file arrives via a plain mirror write (watcher shape) — no
    // ingest_file, so only the pending-set/settle path can index it.
    let new_text =
        "<?php\nnamespace App;\nclass Fresh { public function hit(): void { Base::ping(); } }\n";
    session.set_file_text(Arc::from("fresh.php"), Arc::from(new_text));

    let candidates: Vec<Arc<str>> = vec![Arc::from("base.php"), Arc::from("fresh.php")];
    let refs = session
        .indexed_references_to(
            &Name::method("App\\Base", "ping"),
            &candidates,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert_eq!(
        refs.len(),
        1,
        "the mirror-only file's call site must be found: {refs:?}"
    );
    assert_eq!(refs[0].0.as_ref(), "fresh.php");

    // And the new class itself must resolve (singleton settled, not stale).
    let sub_files: Vec<Arc<str>> = candidates.clone();
    let sites = session.indexed_subtype_classes("App\\Fresh", &sub_files, false);
    assert!(
        sites.is_empty(),
        "no subtypes expected, but the query must not panic: {sites:?}"
    );
}

#[test]
fn mirror_only_class_rename_updates_index_after_settle() {
    let dir = create_temp_dir("warm_start_settle_rename");
    let files = [
        (
            "shape.php",
            "<?php\nnamespace App;\nclass OldShape { public static function draw(): void {} }\n",
        ),
        (
            "user.php",
            "<?php\nnamespace App;\nclass User { public function go(): void { OldShape::draw(); } }\n",
        ),
    ];
    seed_disk_caches(dir.path(), &files);

    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
    let warm: Vec<(Arc<str>, Arc<str>)> = files
        .iter()
        .map(|(p, t)| (Arc::from(*p), Arc::from(*t)))
        .collect();
    session.warm_start_files(&warm);
    assert!(session.workspace_symbol_index_ready());

    // Both files change on disk (git pull shape): the class is renamed and
    // the caller follows. Plain mirror writes only.
    session.set_file_text(
        Arc::from("shape.php"),
        Arc::from(
            "<?php\nnamespace App;\nclass NewShape { public static function draw(): void {} }\n",
        ),
    );
    session.set_file_text(
        Arc::from("user.php"),
        Arc::from("<?php\nnamespace App;\nclass User { public function go(): void { NewShape::draw(); } }\n"),
    );

    let candidates: Vec<Arc<str>> = files.iter().map(|(p, _)| Arc::from(*p)).collect();
    let refs = session
        .indexed_references_to(
            &Name::method("App\\NewShape", "draw"),
            &candidates,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert_eq!(
        refs.len(),
        1,
        "renamed class's call site must resolve through the settled index: {refs:?}"
    );
    assert_eq!(refs[0].0.as_ref(), "user.php");
}
