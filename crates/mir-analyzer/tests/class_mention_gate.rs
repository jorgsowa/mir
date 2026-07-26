//! Class-mention gate index contracts for `indexed_references_to`.
//!
//! The gate's textual predicate ("does this never-committed file mention the
//! needle as a whole identifier") is memoized per file by the mention index.
//! These tests pin the two load-bearing properties:
//!  1. Equivalence — a query answered from recorded mention sets returns
//!     exactly what the raw-scan gate returns (cold vs warm runs agree).
//!  2. Fallback — entries recorded before a class name entered the universe
//!     cannot answer for it; the query falls back to a raw scan and still
//!     finds every reference (no false negatives, ever).

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, Name, PhpVersion};

fn no_cancel() -> impl Fn() -> bool + Sync {
    || false
}

/// Declaring files land via `ingest_file` (seeds the mention universe, as
/// the LSP's index_batch/finalize path does); referencing files land via
/// `set_file_text` only, so the gate is what admits them.
fn build_session(declared: &[(&str, &str)], raw: &[(&str, &str)]) -> AnalysisSession {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    for (path, text) in declared {
        session.ingest_file(Arc::from(*path), Arc::from(*text));
    }
    for (path, text) in raw {
        session.set_file_text(Arc::from(*path), Arc::from(*text));
    }
    session
}

fn refs_files(refs: &[(Arc<str>, mir_analyzer::Range)]) -> Vec<&str> {
    let mut v: Vec<&str> = refs.iter().map(|(f, _)| f.as_ref()).collect();
    v.sort();
    v.dedup();
    v
}

#[test]
fn warm_query_equals_cold_query() {
    let declared = [
        (
            "color.php",
            "<?php\nnamespace App;\nclass Color { public function mix(): void {} }\n",
        ),
        (
            "picker.php",
            "<?php\nnamespace App;\nclass ColorPicker { public function pick(): void {} }\n",
        ),
    ];
    let raw = [
        (
            "uses_color.php",
            "<?php\nnamespace App;\nfunction f(): void { $c = new Color(); $c->mix(); }\n",
        ),
        (
            "uses_picker.php",
            "<?php\nnamespace App;\nfunction g(): void { $p = new ColorPicker(); $p->pick(); }\n",
        ),
        (
            "unrelated.php",
            "<?php\nnamespace App;\nfunction h(): void { $x = 'colorless color_save'; }\n",
        ),
    ];
    let session = build_session(&declared, &raw);
    let files: Vec<Arc<str>> = declared
        .iter()
        .chain(raw.iter())
        .map(|(p, _)| Arc::from(*p))
        .collect();

    let symbol = Name::class("App\\Color");
    let cold = session
        .indexed_references_to(&symbol, &files, false, &no_cancel())
        .expect("not cancelled");
    assert_eq!(
        refs_files(&cold),
        vec!["uses_color.php"],
        "ColorPicker/uses_picker/unrelated must not count as Color references"
    );

    // The cold pass recorded mention sets for the gate-scanned files.
    let stats = session.class_mention_stats();
    assert!(
        stats.files_covered > 0,
        "cold query must populate mention sets: {stats:?}"
    );
    assert!(stats.universe_names >= 2, "{stats:?}");

    // Warm run answers the gate from the index; results must be identical
    // and no new raw-text scans may be recorded.
    let scans_before = stats.scans_recorded;
    let warm = session
        .indexed_references_to(&symbol, &files, false, &no_cancel())
        .expect("not cancelled");
    assert_eq!(cold, warm, "index-answered gate must not change results");
    assert_eq!(
        session.class_mention_stats().scans_recorded,
        scans_before,
        "warm gate must be lookup-only (no rescans)"
    );

    // A different class known at scan time is answerable from the same
    // recorded sets (no rescans needed for correctness).
    let picker = session
        .indexed_references_to(
            &Name::class("App\\ColorPicker"),
            &files,
            false,
            &no_cancel(),
        )
        .expect("not cancelled");
    assert_eq!(refs_files(&picker), vec!["uses_picker.php"]);
}

#[test]
fn constructor_query_uses_class_needle_gate() {
    let declared = [(
        "job.php",
        "<?php\nnamespace App;\nclass Job { public function __construct(public int $n) {} }\n",
    )];
    let raw = [
        (
            "spawn.php",
            "<?php\nnamespace App;\nfunction s(): Job { return new Job(1); }\n",
        ),
        (
            "noise.php",
            "<?php\nnamespace App;\nclass Other { public function __construct() {} }\n",
        ),
    ];
    let session = build_session(&declared, &raw);
    let files: Vec<Arc<str>> = declared
        .iter()
        .chain(raw.iter())
        .map(|(p, _)| Arc::from(*p))
        .collect();
    let symbol = Name::method("App\\Job", "__construct");
    let cold = session
        .indexed_references_to(&symbol, &files, false, &no_cancel())
        .expect("not cancelled");
    assert_eq!(refs_files(&cold), vec!["spawn.php"], "{cold:?}");
    let warm = session
        .indexed_references_to(&symbol, &files, false, &no_cancel())
        .expect("not cancelled");
    assert_eq!(cold, warm);
}

#[test]
fn class_declared_after_scan_falls_back_and_is_found() {
    let declared = [(
        "foo.php",
        "<?php\nnamespace App;\nclass Foo { public function go(): void {} }\n",
    )];
    // `late.php` mentions Bar before any Bar class exists anywhere. Its
    // mention set (recorded by the Foo query below) predates Bar's entry
    // into the universe, so a later Bar query must not trust it blindly.
    let raw = [
        (
            "late.php",
            "<?php\nnamespace App;\nfunction l(): void { $b = new Bar(); $f = new Foo(); }\n",
        ),
        (
            "plain.php",
            "<?php\nnamespace App;\nfunction p(): int { return 42; }\n",
        ),
    ];
    let session = build_session(&declared, &raw);
    let mut files: Vec<Arc<str>> = declared
        .iter()
        .chain(raw.iter())
        .map(|(p, _)| Arc::from(*p))
        .collect();

    // Populate mention sets at the pre-Bar epoch.
    let foo_refs = session
        .indexed_references_to(&Name::class("App\\Foo"), &files, false, &no_cancel())
        .expect("not cancelled");
    assert_eq!(refs_files(&foo_refs), vec!["late.php"]);
    let covered_before = session.class_mention_stats().files_covered;
    assert!(covered_before > 0);

    // Bar enters the workspace (and the universe) only now.
    session.ingest_file(
        Arc::from("bar.php"),
        Arc::from("<?php\nnamespace App;\nclass Bar { public function b(): void {} }\n"),
    );
    files.push(Arc::from("bar.php"));

    // `late.php`'s entry is from the older epoch: the gate must fall back
    // to a raw scan and still find the reference.
    let bar_refs = session
        .indexed_references_to(&Name::class("App\\Bar"), &files, false, &no_cancel())
        .expect("not cancelled");
    assert_eq!(
        refs_files(&bar_refs),
        vec!["late.php"],
        "pre-existing mention entries must not hide a class added later"
    );

    // And the follow-up query answers from upgraded entries, identically.
    let again = session
        .indexed_references_to(&Name::class("App\\Bar"), &files, false, &no_cancel())
        .expect("not cancelled");
    assert_eq!(bar_refs, again);
}

#[test]
fn edited_file_self_invalidates_its_mention_entry() {
    let declared = [(
        "widget.php",
        "<?php\nnamespace App;\nclass Widget { public function w(): void {} }\n",
    )];
    let raw = [(
        "buf.php",
        "<?php\nnamespace App;\nfunction b(): int { return 1; }\n",
    )];
    let session = build_session(&declared, &raw);
    let files: Vec<Arc<str>> = declared
        .iter()
        .chain(raw.iter())
        .map(|(p, _)| Arc::from(*p))
        .collect();
    let symbol = Name::class("App\\Widget");

    let before = session
        .indexed_references_to(&symbol, &files, false, &no_cancel())
        .expect("not cancelled");
    assert!(refs_files(&before).is_empty(), "{before:?}");

    // The edit adds a Widget reference to a file whose recorded mention set
    // says "no Widget". Arc identity must sideline that entry.
    session.set_file_text(
        Arc::from("buf.php"),
        Arc::from("<?php\nnamespace App;\nfunction b(): Widget { return new Widget(); }\n"),
    );
    let after = session
        .indexed_references_to(&symbol, &files, false, &no_cancel())
        .expect("not cancelled");
    assert_eq!(refs_files(&after), vec!["buf.php"]);
}

/// Pseudo-random equivalence sweep: many files with adversarial token soup,
/// every declared class queried cold then warm — results must agree and
/// match the naive expectation (a file counts iff it truly references).
#[test]
fn cold_and_warm_gate_agree_across_generated_workspace() {
    let class_names = [
        "Alpha",
        "AlphaBeta",
        "Beta",
        "Beta_Gamma",
        "Gamma",
        "GammaX",
    ];
    let declared: Vec<(String, String)> = class_names
        .iter()
        .map(|c| {
            (
                format!("decl_{c}.php"),
                format!("<?php\nnamespace Gen;\nclass {c} {{ public function m(): void {{}} }}\n"),
            )
        })
        .collect();
    // Deterministic "randomness": file i references class (i*7+3) % len, and
    // pads with near-miss tokens of every other class.
    let mut raw: Vec<(String, String)> = Vec::new();
    for i in 0..24usize {
        let target = class_names[(i * 7 + 3) % class_names.len()];
        let mut body = format!("<?php\nnamespace Gen;\nfunction f{i}(): void {{\n");
        body.push_str(&format!("    $t = new {target}();\n"));
        for c in class_names {
            body.push_str(&format!("    $s{i} = '{c}x x{c} {c}_'; // not refs\n"));
        }
        body.push_str("}\n");
        raw.push((format!("use_{i}.php"), body));
    }

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    for (p, t) in &declared {
        session.ingest_file(Arc::from(p.as_str()), Arc::from(t.as_str()));
    }
    for (p, t) in &raw {
        session.set_file_text(Arc::from(p.as_str()), Arc::from(t.as_str()));
    }
    let files: Vec<Arc<str>> = declared
        .iter()
        .chain(raw.iter())
        .map(|(p, _)| Arc::from(p.as_str()))
        .collect();

    for c in class_names {
        let symbol = Name::class(format!("Gen\\{c}").as_str());
        let expected: Vec<String> = (0..24usize)
            .filter(|i| class_names[(i * 7 + 3) % class_names.len()] == c)
            .map(|i| format!("use_{i}.php"))
            .collect();
        let cold = session
            .indexed_references_to(&symbol, &files, false, &no_cancel())
            .expect("not cancelled");
        let mut got = refs_files(&cold);
        got.sort();
        let mut want: Vec<&str> = expected.iter().map(|s| s.as_str()).collect();
        want.sort();
        assert_eq!(got, want, "class {c}: cold");
        let warm = session
            .indexed_references_to(&symbol, &files, false, &no_cancel())
            .expect("not cancelled");
        assert_eq!(cold, warm, "class {c}: warm must equal cold");
    }
    let stats = session.class_mention_stats();
    assert!(stats.files_covered >= raw.len(), "{stats:?}");
}
