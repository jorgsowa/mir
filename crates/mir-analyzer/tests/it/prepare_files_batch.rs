//! `prepare_files_for_analysis` against the per-file `prepare_file_for_analysis`
//! loop it replaces: same loads, same diagnostics.

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, IndexCancel, PhpVersion};

const MULTI: &str = "vendor/multi.php";
const BASE: &str = "vendor/base.php";
const DECOY: &str = "vendor/app_lib_b.php";

/// `Lib\B` is only reachable by loading the file the resolver maps `Lib\A` to.
struct Vendor;

impl mir_analyzer::ClassResolver for Vendor {
    fn resolve(&self, fqcn: &str) -> Option<std::path::PathBuf> {
        let path = match fqcn.to_ascii_lowercase().as_str() {
            "lib\\a" => MULTI,
            "lib\\base" => BASE,
            "app\\lib\\b" => DECOY,
            _ => return None,
        };
        Some(std::path::PathBuf::from(path))
    }
}

impl mir_analyzer::SourceProvider for Vendor {
    fn read(&self, path: &str) -> Option<Arc<str>> {
        let text = match path {
            MULTI => {
                "<?php\nnamespace Lib;\nclass A {}\nclass B { public function go(): int { return 1; } }\n"
            }
            BASE => "<?php\nnamespace Lib;\nclass Base { public function run(): int { return 1; } }\n",
            DECOY => "<?php\nnamespace App\\Lib;\nclass B {}\n",
            _ => return None,
        };
        Some(Arc::from(text))
    }
}

fn workspace() -> (AnalysisSession, Vec<Arc<str>>) {
    let files: Vec<(Arc<str>, Arc<str>)> = [
        (
            "src/one.php",
            "<?php\nnamespace App;\nuse Lib\\A;\nfunction one(A $a): void {}\n",
        ),
        (
            "src/two.php",
            "<?php\nnamespace App;\nfunction two(): int { $b = new Lib\\B(); return $b->go(); }\n",
        ),
        (
            "src/three.php",
            "<?php\nnamespace App;\nclass Three extends \\Lib\\Base {\n    /** @return \\Lib\\Missing */\n    public function m() { return $this->run(); }\n}\n",
        ),
    ]
    .into_iter()
    .map(|(path, text)| (Arc::from(path), Arc::from(text)))
    .collect();
    let mut session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(Arc::new(Vendor))
        .with_source_provider(Arc::new(Vendor));
    session.set_workspace_files(files.iter().cloned());
    let paths = files.into_iter().map(|(path, _)| path).collect();
    (session, paths)
}

fn loaded_vendor_files(session: &AnalysisSession) -> Vec<&'static str> {
    [MULTI, BASE, DECOY]
        .into_iter()
        .filter(|path| session.lookup_source_file(path).is_some())
        .collect()
}

fn diagnostics(session: &mut AnalysisSession, paths: &[Arc<str>]) -> Vec<(String, Vec<String>)> {
    let mut out: Vec<(String, Vec<String>)> = session
        .reanalyze_files_cancellable(paths, &IndexCancel::new())
        .into_iter()
        .map(|(file, analysis)| {
            let mut issues: Vec<String> = analysis
                .issues
                .iter()
                .map(|i| format!("{}:{} {}", i.location.line, i.kind.name(), i.kind.message()))
                .collect();
            issues.sort();
            (file.to_string(), issues)
        })
        .collect();
    out.sort();
    out
}

#[test]
fn batch_prepare_loads_and_reports_what_the_per_file_loop_does() {
    let (mut serial, paths) = workspace();
    for path in &paths {
        serial.prepare_file_for_analysis(path);
    }
    let (mut batch, batch_paths) = workspace();
    let mut with_duplicate = batch_paths.clone();
    with_duplicate.push(batch_paths[0].clone());
    batch.prepare_files_for_analysis(&with_duplicate);

    assert_eq!(loaded_vendor_files(&batch), loaded_vendor_files(&serial));
    assert_eq!(
        diagnostics(&mut batch, &batch_paths),
        diagnostics(&mut serial, &paths)
    );
}

#[test]
fn batch_prepare_resolves_a_qualified_name_after_earlier_files_load() {
    let (mut session, paths) = workspace();
    session.prepare_files_for_analysis(&paths);
    // `Lib\B` exists once `one.php`'s warm-up loaded `vendor/multi.php`, so
    // `two.php`'s `new Lib\B()` must not fall back to `App\Lib\B`.
    assert_eq!(loaded_vendor_files(&session), vec![MULTI, BASE]);
}

#[test]
fn batch_prepare_stops_when_cancelled() {
    let (mut session, paths) = workspace();
    assert!(!session.prepare_files_for_analysis_cancellable(&paths, &|| true));
    assert!(loaded_vendor_files(&session).is_empty());
}
