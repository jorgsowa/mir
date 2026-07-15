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
        disk_cache.put(file_path, hash, String::new(), Arc::from(Vec::new()), fabricated);
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
