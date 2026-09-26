//! Warm-cache guards for symbol_at, go-to-definition and find-references: asserted on work done, not wall-clock time.

use std::sync::Arc;
use std::time::{Duration, Instant};

use mir_analyzer::db::Work;
use mir_analyzer::{AnalysisSession, BatchOptions, PhpVersion, ReferenceIncludes};

use crate::common::{create_temp_dir, path_to_arc_str, write_file};

const WARM_CALLS: u64 = 20;

/// Work a warm cursor query must not redo: each one means a lost memo or a fallback walk.
const REWORK: [Work; 5] = [
    Work::WholeFileWalk,
    Work::ScopeAnalysis,
    Work::SymbolAllocated,
    Work::NameAtFallback,
    Work::SymbolAtFallback,
];

struct Project {
    // `analyze_paths` reads from disk, so the files must outlive the session.
    _dir: tempfile::TempDir,
    session: AnalysisSession,
    cursors: Cursors,
}

struct Cursors {
    file_a: Arc<str>,
    file_b: Arc<str>,
    /// Byte offset of the `greet` call in `file_b`.
    method_call: u32,
    /// Byte offset of the `helper` call in `file_b`.
    fn_call: u32,
}

/// Two cross-referencing files after one full analysis pass, as an editor has them once a project is open.
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
    let dir = create_temp_dir("lsp_warm_queries");
    let path_a = write_file(&dir, "GreeterA.php", src_a);
    let path_b = write_file(&dir, "CallerB.php", src_b);
    let cursors = Cursors {
        file_a: path_to_arc_str(&path_a),
        file_b: path_to_arc_str(&path_b),
        method_call: src_b.find("greet(\"World\")").unwrap() as u32 + 1,
        fn_call: src_b.find("helper(41)").unwrap() as u32 + 1,
    };

    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    let _ = session.analyze_paths(&[path_a, path_b], &BatchOptions::new().without_symbols());
    session.rebuild_workspace_symbol_index();

    Project {
        _dir: dir,
        session,
        cursors,
    }
}

fn definition_of_greet_call(c: &Cursors) -> impl FnMut(&mut AnalysisSession) + '_ {
    |session| {
        let def = crate::common::definition_at(session, c.file_b.as_ref(), c.method_call)
            .expect("definition lookup should resolve the greet() call");
        assert_eq!(def.file.as_ref(), c.file_a.as_ref());
    }
}

fn references_of_helper_call(c: &Cursors) -> impl FnMut(&mut AnalysisSession) + '_ {
    let files = vec![c.file_a.clone(), c.file_b.clone()];
    move |session| {
        let refs = crate::common::references_at(
            session,
            c.file_b.as_ref(),
            c.fn_call,
            &files,
            true,
            ReferenceIncludes::Plain,
        )
        .expect("references lookup should resolve helper() usages");
        for file in &files {
            assert!(
                refs.iter().any(|(f, _)| f == file),
                "expected a reference in {file}; got {refs:?}"
            );
        }
    }
}

fn symbol_of_helper_call(c: &Cursors) -> impl FnMut(&mut AnalysisSession) + '_ {
    |session| {
        let symbol = session
            .symbol_at(c.file_b.as_ref(), c.fn_call)
            .expect("symbol_at should resolve the helper() call");
        assert_eq!(symbol.resolved_type.to_string(), "int");
    }
}

/// Asserts every warm call takes the `compact` path and redoes none of [`REWORK`].
fn assert_warm_calls_do_no_rework(
    session: &mut AnalysisSession,
    compact: Work,
    mut call: impl FnMut(&mut AnalysisSession),
) {
    // The first call may still fault in a dependency's declarations.
    call(session);

    let compact_before = session.work_count(compact);
    let rework_before = REWORK.map(|work| session.work_count(work));
    for _ in 0..WARM_CALLS {
        call(session);
    }

    assert_eq!(
        session.work_count(compact) - compact_before,
        WARM_CALLS,
        "warm calls should take the {compact:?} path"
    );
    for (work, before) in REWORK.into_iter().zip(rework_before) {
        assert_eq!(
            session.work_count(work) - before,
            0,
            "warm calls redid {work:?}"
        );
    }
}

#[test]
fn warm_definition_does_no_rework() {
    let Project {
        _dir,
        mut session,
        cursors,
    } = warm_project();
    assert_warm_calls_do_no_rework(
        &mut session,
        Work::NameAtCompact,
        definition_of_greet_call(&cursors),
    );
}

#[test]
fn warm_references_do_no_rework() {
    let Project {
        _dir,
        mut session,
        cursors,
    } = warm_project();
    assert_warm_calls_do_no_rework(
        &mut session,
        Work::NameAtCompact,
        references_of_helper_call(&cursors),
    );
}

#[test]
fn warm_symbol_at_does_no_rework() {
    let Project {
        _dir,
        mut session,
        cursors,
    } = warm_project();
    assert_warm_calls_do_no_rework(
        &mut session,
        Work::SymbolAtCompact,
        symbol_of_helper_call(&cursors),
    );
}

/// Wall-clock variants, too noisy on a shared CPU for the normal run: `cargo test --release -p mir-analyzer --test it lsp_warm_queries::timing -- --ignored --nocapture`.
mod timing {
    use super::*;
    use mir_analyzer::{cache::AnalysisCache, discover_files, perf_fixture::PerfFixture};

    /// One frame of editor latency.
    const INSTANT: Duration = Duration::from_millis(50);
    const REPEATS: u32 = 200;

    fn assert_instant(
        label: &str,
        session: &mut AnalysisSession,
        mut call: impl FnMut(&mut AnalysisSession),
    ) {
        // The first call may still fault in a dependency's declarations.
        call(session);

        let mut max = Duration::ZERO;
        let mut total = Duration::ZERO;
        for _ in 0..REPEATS {
            let t = Instant::now();
            call(session);
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
    #[ignore = "wall-clock benchmark; run explicitly with --release --ignored --nocapture"]
    fn go_to_definition_is_instant_on_warm_cache() {
        let Project {
            _dir,
            mut session,
            cursors,
        } = warm_project();
        assert_instant(
            "definition(Greeter::greet call)",
            &mut session,
            definition_of_greet_call(&cursors),
        );
    }

    #[test]
    #[ignore = "wall-clock benchmark; run explicitly with --release --ignored --nocapture"]
    fn find_references_is_instant_on_warm_cache() {
        let Project {
            _dir,
            mut session,
            cursors,
        } = warm_project();
        assert_instant(
            "references(helper usages)",
            &mut session,
            references_of_helper_call(&cursors),
        );
    }

    #[test]
    #[ignore = "wall-clock benchmark; run explicitly with --release --ignored --nocapture"]
    fn symbol_at_is_instant_on_warm_cache() {
        let Project {
            _dir,
            mut session,
            cursors,
        } = warm_project();
        assert_instant(
            "symbol_at(helper call)",
            &mut session,
            symbol_of_helper_call(&cursors),
        );
    }

    /// A fully indexed real-world fixture (Laravel), with a resolvable `->method(` call site in a project file.
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
        let mut session = AnalysisSession::new(PhpVersion::LATEST).with_cache(cache);
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
        let Some((mut session, file, offset)) = warm_full_corpus_session() else {
            return;
        };
        assert_instant("definition(full corpus)", &mut session, |session| {
            let _ = crate::common::definition_at(session, file.as_ref(), offset);
        });
    }

    #[test]
    #[ignore = "needs a real fixture (Laravel/Symfony); run explicitly with --ignored --nocapture"]
    fn find_references_stays_instant_on_full_workspace_index() {
        let Some((mut session, file, offset)) = warm_full_corpus_session() else {
            return;
        };
        let files = vec![file.clone()];
        assert_instant("references(full corpus)", &mut session, |session| {
            let _ = crate::common::references_at(
                session,
                file.as_ref(),
                offset,
                &files,
                true,
                ReferenceIncludes::Plain,
            );
        });
    }
}
