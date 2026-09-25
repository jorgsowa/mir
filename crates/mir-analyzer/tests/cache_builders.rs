//! Attaching a disk cache keeps every setting configured on the session before it.

mod common;

use std::sync::Arc;

use mir_analyzer::cache::AnalysisCache;
use mir_analyzer::{AnalysisSession, FileAnalyzer, IssueKind, PhpVersion};

use self::common::create_temp_dir;

const VENDOR_PATH: &str = "/virtual/vendor/Base.php";
const VENDOR_BASE: &str =
    "<?php\nnamespace Vendor;\nclass Base { public function fromVendor(): int { return 1; } }\n";
const CALLER: &str =
    "<?php\nnamespace App;\nfunction use_vendor(\\Vendor\\Base $b): int { return $b->fromVendor(); }\n";

/// Serves `Vendor\Base` from memory only, so a fallback to disk can't find it.
struct InMemoryVendor;

impl mir_analyzer::ClassResolver for InMemoryVendor {
    fn resolve(&self, fqcn: &str) -> Option<std::path::PathBuf> {
        (fqcn == "Vendor\\Base").then(|| std::path::PathBuf::from(VENDOR_PATH))
    }
}

impl mir_analyzer::SourceProvider for InMemoryVendor {
    fn read(&self, path: &str) -> Option<Arc<str>> {
        (path == VENDOR_PATH).then(|| Arc::from(VENDOR_BASE))
    }
}

fn configured_session() -> AnalysisSession {
    AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(Arc::new(InMemoryVendor))
        .with_source_provider(Arc::new(InMemoryVendor))
}

fn assert_vendor_class_loads_on_demand(mut session: AnalysisSession) {
    let path: Arc<str> = Arc::from("caller.php");
    session.ingest_file(path.clone(), Arc::from(CALLER));
    let parsed = php_rs_parser::parse(CALLER);
    let result = FileAnalyzer::new(&mut session).analyze_diagnostics_only(
        path,
        CALLER,
        &parsed.program,
        &parsed.source_map,
    );
    assert!(
        !result.issues.iter().any(|i| matches!(
            i.kind,
            IssueKind::UndefinedClass { .. } | IssueKind::UndefinedMethod { .. }
        )),
        "{:?}",
        result.issues
    );
}

#[test]
fn with_cache_dir_keeps_the_resolver_and_source_provider() {
    let cache_dir = create_temp_dir("cache_builders_dir");
    assert_vendor_class_loads_on_demand(configured_session().with_cache_dir(cache_dir.path()));
}

#[test]
fn with_cache_keeps_the_resolver_and_source_provider() {
    let cache_dir = create_temp_dir("cache_builders_cache");
    let cache = Arc::new(AnalysisCache::open(
        cache_dir.path(),
        PhpVersion::LATEST.cache_byte(),
        0,
    ));
    assert_vendor_class_loads_on_demand(configured_session().with_cache(cache));
}
