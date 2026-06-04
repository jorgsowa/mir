//! Tests for the eager + background vendor-indexing model.
//!
//! Replaces the old lazy-load/retry/evict tests. The model is now:
//!
//! 1. **Eager background index** — the consumer enumerates vendor files
//!    (`Psr4Map::all_vendor_files`) and registers them via
//!    `AnalysisSession::index_batch`; the workspace symbol index is built
//!    incrementally and stays static, so `find_class_like` resolves any vendor
//!    class — including types reached only through inheritance/return types.
//! 2. **Priority indexing** — before the background walk finishes, the open
//!    file's *direct* references are faulted in so there is no transient false
//!    `UndefinedClass`, bounded to that file's direct refs (no transitive BFS).
//! 3. **Static inputs** — closing a buffer does NOT evict vendor (no per-file
//!    eviction); memory is bounded by the LRU memo + parse cache instead.

mod common;

use std::fs;
use std::sync::Arc;

use mir_analyzer::{AnalysisSession, FileAnalyzer, IndexCancel, IndexParallelism, PhpVersion};

use self::common::create_temp_dir;

// ─── helpers ────────────────────────────────────────────────────────────────

fn make_session(root: &std::path::Path) -> AnalysisSession {
    let psr4 = mir_analyzer::composer::Psr4Map::from_composer(root).expect("psr4 map");
    AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(psr4))
}

fn write_composer(root: &std::path::Path) {
    fs::write(
        root.join("composer.json"),
        r#"{
  "autoload": {
    "psr-4": {
      "App\\": "src/",
      "Vendor\\": "vendor/VendorLib/src/"
    }
  }
}"#,
    )
    .unwrap();
}

fn analyze(session: &AnalysisSession, path: Arc<str>, src: &str) -> mir_analyzer::FileAnalysis {
    let parsed = php_rs_parser::parse(src);
    assert!(
        parsed.errors.is_empty(),
        "parse errors: {:?}",
        parsed.errors
    );
    session.ingest_file(path.clone(), Arc::from(src));
    FileAnalyzer::new(session).analyze(path, src, &parsed.program, &parsed.source_map)
}

/// Every PHP file the analyzer should index for these fixtures. The fixtures
/// declare both `App\` and `Vendor\` directly in `composer.json` autoload, so
/// `from_composer` classifies them as project entries (real vendor entries come
/// from `vendor/composer/installed.json`, which the fixtures don't generate).
/// Indexing `project_files() ∪ all_vendor_files()` covers everything regardless
/// of that classification.
fn indexable_files(root: &std::path::Path) -> Vec<(Arc<str>, Arc<str>)> {
    let psr4 = mir_analyzer::composer::Psr4Map::from_composer(root).expect("psr4 map");
    let mut paths = psr4.project_files();
    paths.extend(psr4.all_vendor_files());
    paths.sort();
    paths.dedup();
    paths
        .into_iter()
        .filter_map(|p| {
            let text = fs::read_to_string(&p).ok()?;
            Some((
                Arc::from(p.to_string_lossy().as_ref()),
                Arc::from(text.as_str()),
            ))
        })
        .collect()
}

/// Drive the chunked background indexer over every file, then finalize — the
/// rust-analyzer-style eager warm-up a consumer performs at session start.
fn eager_index_vendor(session: &AnalysisSession, root: &std::path::Path) {
    let cancel = IndexCancel::new();
    let files = indexable_files(root);
    for chunk in files.chunks(2) {
        session.index_batch(chunk, IndexParallelism::Sequential, &cancel);
    }
    session.finalize_index();
}

// ─── eager index completeness ─────────────────────────────────────────────────

/// With the vendor tree eagerly indexed, a method call on the return value of an
/// **inherited** vendor method resolves and emits `UndefinedMethod` — the case
/// the old lazy retry loop existed to approximate, now covered for free because
/// the full symbol index is static.
#[test]
fn eager_index_resolves_inherited_return_type_member() {
    let root = create_temp_dir("eager_inherited");
    let vendor_src = root.path().join("vendor/VendorLib/src");
    let app_src = root.path().join("src");
    fs::create_dir_all(&vendor_src).unwrap();
    fs::create_dir_all(&app_src).unwrap();
    write_composer(root.path());

    fs::write(
        vendor_src.join("User.php"),
        "<?php\nnamespace Vendor;\nclass User {\n    public function getId(): int { return 0; }\n}\n",
    )
    .unwrap();
    fs::write(
        vendor_src.join("AbstractController.php"),
        "<?php\nnamespace Vendor;\nclass AbstractController {\n    \
         public function getUser(): \\Vendor\\User { return null; }\n}\n",
    )
    .unwrap();
    fs::write(
        vendor_src.join("BlogController.php"),
        "<?php\nnamespace Vendor;\nclass BlogController extends \\Vendor\\AbstractController {}\n",
    )
    .unwrap();

    let session = make_session(root.path());
    eager_index_vendor(&session, root.path());

    let open_path: Arc<str> = Arc::from(app_src.join("open.php").to_string_lossy().as_ref());
    let open_src = "<?php\n\
        use Vendor\\BlogController;\n\
        $c = new BlogController();\n\
        $u = $c->getUser();\n\
        $u->nope();\n";

    let analysis = analyze(&session, open_path, open_src);
    let undefined_method = analysis
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedMethod")
        .count();
    assert_eq!(
        undefined_method,
        1,
        "expected UndefinedMethod on Vendor\\User::nope() via inherited return type; issues: {:?}",
        analysis
            .issues
            .iter()
            .map(|i| i.kind.name())
            .collect::<Vec<_>>()
    );
    assert!(session.contains_class("Vendor\\User"));
}

// ─── priority indexing (partial warm-up window) ───────────────────────────────

/// Before any background indexing, opening a file resolves its **direct**
/// vendor reference via priority indexing — no transient false UndefinedClass.
#[test]
fn priority_index_resolves_direct_ref_before_background_walk() {
    let root = create_temp_dir("priority_direct");
    let vendor_src = root.path().join("vendor/VendorLib/src");
    let app_src = root.path().join("src");
    fs::create_dir_all(&vendor_src).unwrap();
    fs::create_dir_all(&app_src).unwrap();
    write_composer(root.path());

    fs::write(
        vendor_src.join("Service.php"),
        "<?php\nnamespace Vendor;\nclass Service { public function go(): void {} }\n",
    )
    .unwrap();

    let session = make_session(root.path());
    // NOTE: no eager_index_vendor — exercise the priority-index path only.
    let open_path: Arc<str> = Arc::from(app_src.join("open.php").to_string_lossy().as_ref());
    let open_src = "<?php\nuse Vendor\\Service;\n$s = new Service();\n$s->go();\n";

    let analysis = analyze(&session, open_path, open_src);
    assert!(
        session.contains_class("Vendor\\Service"),
        "priority indexing must fault in the open file's direct reference"
    );
    let undefined_class = analysis
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedClass")
        .count();
    assert_eq!(
        undefined_class, 0,
        "Service is a real class — no UndefinedClass"
    );
}

/// Priority indexing is bounded to the file's **direct** references: opening a
/// file that names only `Root` does not pull in the 180 transitively-reachable
/// Leaf/Deep classes (proves there is no transitive fan-out).
#[test]
fn priority_index_is_bounded_to_direct_refs() {
    let root = create_temp_dir("priority_bounded");
    let vendor_src = root.path().join("vendor/VendorLib/src");
    let app_src = root.path().join("src");
    fs::create_dir_all(&vendor_src).unwrap();
    fs::create_dir_all(&app_src).unwrap();
    write_composer(root.path());

    for leaf_idx in 0..30usize {
        for deep_idx in 0..5usize {
            let name = format!("Deep{leaf_idx}_{deep_idx}");
            fs::write(
                vendor_src.join(format!("{name}.php")),
                format!("<?php\nnamespace Vendor;\nclass {name} {{}}\n"),
            )
            .unwrap();
        }
    }
    for leaf_idx in 0..30usize {
        let methods: String = (0..5usize)
            .map(|d| {
                format!("    public function d{d}(): \\Vendor\\Deep{leaf_idx}_{d} {{ return new \\Vendor\\Deep{leaf_idx}_{d}(); }}\n")
            })
            .collect();
        fs::write(
            vendor_src.join(format!("Leaf{leaf_idx}.php")),
            format!("<?php\nnamespace Vendor;\nclass Leaf{leaf_idx} {{\n{methods}}}\n"),
        )
        .unwrap();
    }
    let root_methods: String = (0..30usize)
        .map(|i| format!("    public function leaf{i}(): \\Vendor\\Leaf{i} {{ return new \\Vendor\\Leaf{i}(); }}\n"))
        .collect();
    fs::write(
        vendor_src.join("Root.php"),
        format!("<?php\nnamespace Vendor;\nclass Root {{\n{root_methods}}}\n"),
    )
    .unwrap();

    let session = make_session(root.path());
    session.ensure_all_stubs();
    let stub_baseline = session.tracked_file_count();

    let open_path: Arc<str> = Arc::from(app_src.join("open.php").to_string_lossy().as_ref());
    let open_src = "<?php\nuse Vendor\\Root;\n$r = new Root();\n$r->nope();\n";
    analyze(&session, open_path, open_src);

    let vendor_loaded = session.tracked_file_count().saturating_sub(stub_baseline);
    assert!(
        vendor_loaded <= 2,
        "priority indexing should load only Root (direct ref), got {vendor_loaded} vendor files"
    );
    assert!(session.contains_class("Vendor\\Root"));
    assert!(
        !session.contains_class("Vendor\\Leaf0"),
        "transitively-reachable Leaf0 must NOT be priority-loaded"
    );
}

// ─── static inputs (no eviction on close) ─────────────────────────────────────

/// Closing a buffer does NOT evict vendor classes — vendor inputs are static in
/// the eager model. (The old refcounted per-file eviction is gone.)
#[test]
fn invalidate_file_keeps_vendor_static() {
    let root = create_temp_dir("static_vendor");
    let vendor_src = root.path().join("vendor/VendorLib/src");
    let app_src = root.path().join("src");
    fs::create_dir_all(&vendor_src).unwrap();
    fs::create_dir_all(&app_src).unwrap();
    write_composer(root.path());

    fs::write(
        vendor_src.join("Service.php"),
        "<?php\nnamespace Vendor;\nclass Service { public function foo(): void {} }\n",
    )
    .unwrap();

    let session = make_session(root.path());
    eager_index_vendor(&session, root.path());
    assert!(session.contains_class("Vendor\\Service"));

    let open_path: Arc<str> = Arc::from(app_src.join("open.php").to_string_lossy().as_ref());
    let open_src = "<?php\nuse Vendor\\Service;\n$s = new Service();\n$s->foo();\n";
    analyze(&session, open_path.clone(), open_src);

    session.invalidate_file(open_path.as_ref());

    assert!(
        session.contains_class("Vendor\\Service"),
        "vendor classes are static — closing a project buffer must not evict them"
    );
}
