//! False-positive baseline harness: counts issues by kind and severity over the
//! full Laravel corpus, so the §1.2 correctness gate (the `UndefinedClass`
//! count) is reproducible run-to-run.
//!
//! Run explicitly (it is `#[ignore]`d like the other measurement harnesses):
//!
//! ```sh
//! cargo test --release -p mir-analyzer --test measure_baseline -- --ignored --nocapture
//! ```
//!
//! Emits a stable, greppable table to stderr:
//! - per-kind counts (suppressed issues excluded), sorted descending;
//! - a severity rollup (Error / Warning / Info);
//! - the headline `UndefinedClass` gate count.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use mir_analyzer::{discover_files, AnalysisSession, BatchOptions, PhpVersion, Severity};

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("fixtures")
        .join("laravel")
}

#[test]
#[ignore = "measurement harness; run explicitly with --release --ignored"]
fn measure_baseline() {
    let root = fixtures_root();
    if !root.exists() {
        eprintln!("Skipping: fixture not present at {}", root.display());
        return;
    }

    let vendor_files = discover_files(&root.join("vendor"));
    let project_files = discover_files(&root.join("src"));
    let all_files: Vec<PathBuf> = vendor_files
        .iter()
        .chain(project_files.iter())
        .cloned()
        .collect();
    eprintln!(
        "[baseline] corpus: {} files ({} vendor + {} project)",
        all_files.len(),
        vendor_files.len(),
        project_files.len()
    );

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let opts = BatchOptions::new();
    let result = session.analyze_paths(&all_files, &opts);

    // The correctness gate measures false positives in *project* code only:
    // `src/` is analyzed against `vendor/` as resolution context, so issues
    // located inside vendored dependencies are not part of the reported
    // surface. Filter to the project file set (matching how a user runs mir on
    // their own `src/`).
    let project_set: HashSet<String> = project_files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    // Exclude suppressed issues — they are not part of the reported surface.
    // Track per (kind, severity) so the gate (Error+Warning) is separable from
    // the Info-level completeness noise.
    let sev_tag = |s: Severity| match s {
        Severity::Error => "ERR ",
        Severity::Warning => "WARN",
        Severity::Info => "info",
    };
    let mut by_kind: HashMap<(&'static str, &'static str), u64> = HashMap::new();
    let (mut errors, mut warnings, mut infos) = (0u64, 0u64, 0u64);
    let mut total = 0u64;
    for issue in result
        .issues
        .iter()
        .filter(|i| !i.suppressed)
        .filter(|i| project_set.contains(i.location.file.as_ref()))
    {
        total += 1;
        *by_kind
            .entry((issue.kind.name(), sev_tag(issue.severity)))
            .or_insert(0) += 1;
        match issue.severity {
            Severity::Error => errors += 1,
            Severity::Warning => warnings += 1,
            Severity::Info => infos += 1,
        }
    }

    // Optional drill-down: `MIR_DUMP_KIND=PossiblyNullArgument` prints
    // file:line + snippet for every project-code issue of that kind, so FP
    // patterns can be eyeballed without re-running the whole analysis by hand.
    if let Ok(target) = std::env::var("MIR_DUMP_KIND") {
        let mut shown = 0u64;
        for issue in result
            .issues
            .iter()
            .filter(|i| !i.suppressed)
            .filter(|i| project_set.contains(i.location.file.as_ref()))
            .filter(|i| i.kind.name() == target)
        {
            let file = issue.location.file.as_ref();
            let short = file.rsplit("/src/").next().unwrap_or(file);
            eprintln!(
                "[dump {target}] src/{short}:{} | {}",
                issue.location.line,
                issue.snippet.as_deref().unwrap_or("").replace('\n', " ")
            );
            shown += 1;
        }
        eprintln!("[dump {target}] === {shown} occurrences ===");
        return;
    }

    let mut kinds: Vec<((&'static str, &'static str), u64)> = by_kind.into_iter().collect();
    // Sort by count desc, then name asc for a stable diffable ordering.
    kinds.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0 .0.cmp(b.0 .0)));

    eprintln!("[baseline] === Error+Warning kinds only (the gate) ===");
    for ((name, sev), count) in kinds.iter().filter(|((_, s), _)| *s != "info") {
        eprintln!("[baseline] {sev} {count:>6}  {name}");
    }
    eprintln!("[baseline] === per-kind × severity (all, suppressed excluded) ===");
    for ((name, sev), count) in &kinds {
        eprintln!("[baseline] {sev} {count:>6}  {name}");
    }
    eprintln!("[baseline] === severity rollup ===");
    eprintln!("[baseline] Error           {errors:>6}");
    eprintln!("[baseline] Warning         {warnings:>6}");
    eprintln!("[baseline] Error+Warning   {:>6}", errors + warnings);
    eprintln!("[baseline] Info            {infos:>6}");
    eprintln!("[baseline] TOTAL           {total:>6}");
    eprintln!(
        "[baseline] === GATE: UndefinedClass = {} ===",
        kinds
            .iter()
            .filter(|((n, _), _)| *n == "UndefinedClass")
            .map(|(_, c)| *c)
            .sum::<u64>()
    );
}
