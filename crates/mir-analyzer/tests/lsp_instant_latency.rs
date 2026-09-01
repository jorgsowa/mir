//! Hard latency assertions for the cursor-driven LSP entry points: go-to-
//! definition, find-references ("usages"), and hover.
//!
//! These are NOT benchmarks (see `benches/incremental_workload.rs` for the
//! Criterion suite that tracks nanosecond-level trends) — they're regression
//! guards that fail the build the moment one of these calls stops being
//! "instant" from an editor's point of view. Warm-cache latency more than a
//! few tens of milliseconds is a perceptible stall on every keystroke-driven
//! request, so a generous-but-real threshold is asserted rather than just
//! printed.
//!
//! Per-test setup here is a tiny cross-file synthetic project — not a full
//! real-world corpus — so every test in this file runs as part of the normal
//! `cargo test` suite (no fixture download, no `--ignored`). The two tests
//! that need a real, fully-indexed workspace to prove the guarantee still
//! holds at scale are marked `#[ignore]` and documented at the bottom.

mod common;

use std::sync::Arc;
use std::time::{Duration, Instant};

use mir_analyzer::{AnalysisSession, BatchOptions, PhpVersion, ReferenceIncludes};

use self::common::{create_temp_dir, path_to_arc_str, write_file};

/// Warm-cache calls must resolve well within one frame of editor latency.
/// Generous enough to absorb CI noise; tight enough to catch a real
/// regression (a reintroduced whole-file walk, a lost memo, an O(n) scan).
const INSTANT: Duration = Duration::from_millis(50);
const REPEATS: u32 = 200;

struct Project {
    // Kept alive for the test's duration: `analyze_paths` reads paths from
    // disk, so the backing temp files must outlive the analysis pass.
    _dir: tempfile::TempDir,
    session: AnalysisSession,
    file_a: Arc<str>,
    file_b: Arc<str>,
    /// Byte offset of the `greet` call in `file_b` (method go-to-definition).
    method_call_offset: u32,
    /// Byte offset of the `helper` call in `file_b` (function hover/usages).
    fn_call_offset: u32,
}

/// Builds a tiny two-file cross-reference project and runs one full analysis
/// pass so declarations, inference, and the reference index are all populated
/// — i.e. the state an editor has after a file is first opened and its
/// dependencies are pulled in, not a cold empty session.
fn warm_project() -> Project {
    let src_a = "<?php
namespace App;

class Greeter
{
    public function greet(string $name): string
    {
        return \"Hello, {$name}!\";
    }
}

function helper(int $x): int
{
    return $x + 1;
}
";
    let src_b = "<?php
namespace App;

function caller(): void
{
    $g = new Greeter();
    echo $g->greet(\"World\");
    echo helper(41);
}
";
    let dir = create_temp_dir("lsp_instant_latency");
    let path_a = write_file(&dir, "GreeterA.php", src_a);
    let path_b = write_file(&dir, "CallerB.php", src_b);
    let file_a = path_to_arc_str(&path_a);
    let file_b = path_to_arc_str(&path_b);

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    // One full pass over both files: populates diagnostics, inferred types,
    // and the maintained reference index the way opening a project would.
    let _ = session.analyze_paths(&[path_a, path_b], &BatchOptions::new().without_symbols());
    session.rebuild_workspace_symbol_index();

    let method_call_offset = src_b.find("greet(\"World\")").unwrap() as u32 + 1;
    let fn_call_offset = src_b.find("helper(41)").unwrap() as u32 + 1;

    Project {
        _dir: dir,
        session,
        file_a,
        file_b,
        method_call_offset,
        fn_call_offset,
    }
}

fn assert_instant(label: &str, mut call: impl FnMut()) {
    // Untimed warm-up call: the very first call may still fault in a
    // dependency's declarations. Steady-state latency is what matters here.
    call();

    let mut max = Duration::ZERO;
    let mut total = Duration::ZERO;
    for _ in 0..REPEATS {
        let t = Instant::now();
        call();
        let elapsed = t.elapsed();
        total += elapsed;
        max = max.max(elapsed);
    }
    let avg = total / REPEATS;
    eprintln!("{label}: avg {avg:?}, max {max:?} over {REPEATS} warm repeats");
    assert!(
        max < INSTANT,
        "{label} took up to {max:?} on a warm cache (avg {avg:?}) — expected < {INSTANT:?}"
    );
}

#[test]
fn go_to_definition_is_instant_on_warm_cache() {
    let p = warm_project();
    assert_instant("definition_at(Greeter::greet call)", || {
        let def = p
            .session
            .definition_at(p.file_b.as_ref(), p.method_call_offset)
            .expect("definition_at should resolve the greet() call");
        assert_eq!(def.file.as_ref(), p.file_a.as_ref());
    });
}

#[test]
fn find_references_usages_is_instant_on_warm_cache() {
    let p = warm_project();
    let files = vec![p.file_a.clone(), p.file_b.clone()];
    assert_instant("references_at(helper usages)", || {
        let refs = p
            .session
            .references_at(
                p.file_b.as_ref(),
                p.fn_call_offset,
                &files,
                true,
                ReferenceIncludes::Plain,
            )
            .expect("references_at should resolve helper() usages");
        assert!(
            !refs.is_empty(),
            "expected at least the declaration + call-site usage"
        );
    });
}

#[test]
fn hover_is_instant_on_warm_cache() {
    let p = warm_project();
    assert_instant("hover_at(helper call)", || {
        p.session
            .hover_at(p.file_b.as_ref(), p.fn_call_offset)
            .expect("hover_at should resolve the helper() call");
    });
}

// ---------------------------------------------------------------------------
// Full-corpus variants
// ---------------------------------------------------------------------------
//
// The tests above prove the entry points stay instant in isolation; they
// don't prove the reference index / symbol lookup stays instant once it's
// carrying a real workspace's worth of classes and postings. These variants
// warm a full real-world fixture (Laravel, via `PerfFixture`) and re-check
// the same guarantee against it. They're `#[ignore]`d because building that
// warm state costs real wall-clock time (parsing + analyzing the whole
// corpus once) — the thing being timed is only the *query* on top of it, not
// the warm-up.
//
//   cargo test -p mir-analyzer --test lsp_instant_latency -- --ignored --nocapture

mod full_corpus {
    use super::*;
    use mir_analyzer::{cache::AnalysisCache, discover_files, perf_fixture::PerfFixture};

    fn warm_full_corpus_session() -> Option<(AnalysisSession, Arc<str>, u32)> {
        let fixture = PerfFixture::discover()?;
        if !fixture.has_full_corpus() {
            eprintln!(
                "skipping: perf fixture incomplete at {}",
                fixture.root().display()
            );
            return None;
        }

        let vendor_files = discover_files(&fixture.vendor_root());
        let project_files = discover_files(&fixture.src_root());

        let cache_dir = tempfile::tempdir().unwrap();
        let cache = Arc::new(AnalysisCache::open(
            cache_dir.path(),
            PhpVersion::LATEST.cache_byte(),
            0,
        ));
        let session = AnalysisSession::new(PhpVersion::LATEST).with_cache(cache);
        session.ensure_all_stubs();

        let vendor_pairs: Vec<(Arc<str>, Arc<str>)> = vendor_files
            .iter()
            .filter_map(|path| {
                let src = std::fs::read_to_string(path).ok()?;
                Some((
                    Arc::from(path.to_string_lossy().as_ref()),
                    Arc::from(src.as_str()),
                ))
            })
            .collect();
        session.set_vendor_files(vendor_pairs);
        for path in &project_files {
            if let Ok(src) = std::fs::read_to_string(path) {
                let file: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
                session.ingest_file(file, Arc::from(src));
            }
        }
        session.rebuild_workspace_symbol_index();

        // Find a resolvable method-call site (`->something(`) in one of the
        // project files to exercise real cross-file resolution.
        for path in &project_files {
            let Ok(src) = std::fs::read_to_string(path) else {
                continue;
            };
            if let Some(rel) = src.find("->") {
                if let Some(paren_rel) = src[rel..].find('(') {
                    let name_start = rel + 2;
                    let name_end = rel + paren_rel;
                    if name_end > name_start
                        && src[name_start..name_end]
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '_')
                    {
                        let file: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
                        let offset = name_start as u32 + 1;
                        let _ = session.analyze_paths(
                            std::slice::from_ref(path),
                            &BatchOptions::new().without_symbols(),
                        );
                        return Some((session, file, offset));
                    }
                }
            }
        }
        eprintln!("skipping: no `->method(` call site found in fixture project files");
        None
    }

    #[test]
    #[ignore = "needs a real fixture (Laravel/Symfony); run explicitly with --ignored --nocapture"]
    fn go_to_definition_stays_instant_on_full_workspace_index() {
        let Some((session, file, offset)) = warm_full_corpus_session() else {
            return;
        };
        assert_instant("definition_at(full corpus)", || {
            let _ = session.definition_at(file.as_ref(), offset);
        });
    }

    #[test]
    #[ignore = "needs a real fixture (Laravel/Symfony); run explicitly with --ignored --nocapture"]
    fn find_references_stays_instant_on_full_workspace_index() {
        let Some((session, file, offset)) = warm_full_corpus_session() else {
            return;
        };
        let files = vec![file.clone()];
        assert_instant("references_at(full corpus)", || {
            let _ = session.references_at(
                file.as_ref(),
                offset,
                &files,
                true,
                ReferenceIncludes::Plain,
            );
        });
    }
}
