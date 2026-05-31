//! Open-file (LSP / `FileAnalyzer`) lazy-load completeness.
//!
//! `tests/lazy_load.rs` exercises the *batch* path (`analyze_paths`), which
//! already runs a full declared-type-closure fixpoint, so its diagnostics are
//! complete. This file exercises the *open-file* path
//! (`AnalysisSession::ingest_file` + `FileAnalyzer::analyze`), which the LSP
//! uses for the currently-edited buffer.
//!
//! For full open-file diagnostics, the loader must follow the same closure the
//! batch path does: not just the classes syntactically referenced in the open
//! file, but their inheritance chains and the classes named in their method
//! signatures (return / parameter / property types). Otherwise a value whose
//! type comes from a vendor method's return type, or a member inherited from a
//! vendor parent, is invisible — the type silently degrades and the diagnostic
//! is missed.
//!
//! Each test below is a 0 -> 1 flip: the expected diagnostic is *missed* when
//! the closure is too shallow and *surfaces* once the transitive declared-type
//! closure is loaded.

mod common;

use std::fs;
use std::sync::Arc;

use mir_analyzer::{AnalysisSession, FileAnalyzer, PhpVersion};

use self::common::create_temp_dir;

/// Set up a PSR-4 project (`App\` -> `src/`) with the given lazily-loadable
/// library files written under `src/`, then analyze `open_src` through the
/// open-file path (`ingest_file` + `FileAnalyzer::analyze`). The library files
/// are NOT ingested up front — they must be discovered via lazy loading during
/// analysis, exactly as vendor code would be for an open buffer.
fn analyze_open_file_with_psr4(
    lib_files: &[(&str, &str)],
    open_name: &str,
    open_src: &str,
) -> mir_analyzer::FileAnalysis {
    let root = create_temp_dir("lazy_load_open_file");
    fs::create_dir_all(root.path().join("src")).unwrap();
    for (name, src) in lib_files {
        fs::write(root.path().join("src").join(name), src).unwrap();
    }
    fs::write(
        root.path().join("composer.json"),
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    )
    .unwrap();
    let psr4 = mir_analyzer::composer::Psr4Map::from_composer(root.path()).expect("psr4 map");

    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(psr4));

    let open_path: Arc<str> = Arc::from(root.path().join(open_name).to_string_lossy().as_ref());
    let open_src_arc: Arc<str> = Arc::from(open_src);
    session.ingest_file(open_path.clone(), open_src_arc.clone());

    let parsed = php_rs_parser::parse(open_src);
    assert!(
        parsed.errors.is_empty(),
        "parser errors in open-file source: {:?}",
        parsed.errors
    );
    FileAnalyzer::new(&session).analyze(open_path, open_src, &parsed.program, &parsed.source_map)
}

fn undefined_method_count(analysis: &mir_analyzer::FileAnalysis) -> usize {
    analysis
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedMethod")
        .count()
}

/// The open file calls a method that only exists on the *return type* of a
/// lazily-loaded class. `App\Widget` is never named directly in the open file —
/// it is reachable only through `App\Service::make()`'s declared return type.
/// To know that `nope()` is undefined on the returned `Widget`, the loader must
/// follow the return-type edge and load `Widget`.
#[test]
fn open_file_resolves_method_on_lazily_loaded_return_type() {
    let analysis = analyze_open_file_with_psr4(
        &[
            (
                "Widget.php",
                "<?php\nnamespace App;\nclass Widget {\n    public function render(): void {}\n}\n",
            ),
            (
                "Service.php",
                "<?php\nnamespace App;\nclass Service {\n    public function make(): \\App\\Widget {\n        return new \\App\\Widget();\n    }\n}\n",
            ),
        ],
        "consumer.php",
        "<?php\n$s = new \\App\\Service();\n$w = $s->make();\n$w->nope();\n",
    );

    assert_eq!(
        undefined_method_count(&analysis),
        1,
        "Widget is reachable only via Service::make()'s return type. Once that closure \
         is loaded, $w->nope() is a known-class/unknown-method -> UndefinedMethod. A count \
         of 0 means Widget was never loaded and the value silently degraded to mixed."
    );
}

/// The open file references `App\Mid` directly, but the member it (in)correctly
/// calls would be inherited from `App\Base`, which is reachable only through
/// Mid's `extends` edge. Until Base is loaded, Mid has an unknown ancestor and
/// member checks against it are suppressed (to avoid false positives), so
/// `nope()` is missed. Once the inheritance closure is loaded, the ancestor
/// chain is complete and the genuinely-undefined `nope()` surfaces.
#[test]
fn open_file_resolves_inherited_chain_of_lazily_loaded_parent() {
    let analysis = analyze_open_file_with_psr4(
        &[
            (
                "Base.php",
                "<?php\nnamespace App;\nclass Base {\n    public function fromBase(): void {}\n}\n",
            ),
            (
                "Mid.php",
                "<?php\nnamespace App;\nclass Mid extends \\App\\Base {}\n",
            ),
        ],
        "consumer.php",
        "<?php\n$m = new \\App\\Mid();\n$m->nope();\n",
    );

    assert_eq!(
        undefined_method_count(&analysis),
        1,
        "Mid extends the lazily-loadable Base. Once the inheritance closure is loaded the \
         ancestor chain is complete and $m->nope() (absent on Mid and Base) is reported. A \
         count of 0 means Base was never loaded, Mid had an unknown ancestor, and the check \
         was suppressed."
    );
}

/// Control: a member that genuinely exists on the lazily-loaded parent must NOT
/// be flagged once the closure is loaded — guards against the fix over-reporting.
#[test]
fn open_file_does_not_flag_valid_inherited_member() {
    let analysis = analyze_open_file_with_psr4(
        &[
            (
                "Base.php",
                "<?php\nnamespace App;\nclass Base {\n    public function fromBase(): void {}\n}\n",
            ),
            (
                "Mid.php",
                "<?php\nnamespace App;\nclass Mid extends \\App\\Base {}\n",
            ),
        ],
        "consumer.php",
        "<?php\n$m = new \\App\\Mid();\n$m->fromBase();\n",
    );

    assert_eq!(
        undefined_method_count(&analysis),
        0,
        "fromBase() is inherited from the lazily-loaded Base and must resolve cleanly"
    );
}
