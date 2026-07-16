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
    session.warm_start_files(&[(Arc::from(file_path), Arc::from(text))]);

    session.ingest_file(Arc::from("later.php"), Arc::from("<?php\nclass Later {}\n"));

    let refs = session
        .indexed_references_to(
            &Name::method("App\\Other", "bogus"),
            &[Arc::from(file_path)],
            false,
            &|| false,
        )
        .expect("not cancelled");
    assert!(
        refs.is_empty(),
        "an unresolved replay must be re-verified after growth, not served \
         from the stale disk-cache posting: {refs:?}"
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
            &|| false,
        )
        .expect("not cancelled");
    assert!(refs.is_empty(), "no reference sites expected: {refs:?}");
}
