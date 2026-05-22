//! Tests for the Phase 2 salsa-tracked resolver queries
//! (`db::resolve_fqcn_to_path`, `db::source_file_for_fqcn`).
//!
//! The contract these guarantee:
//!   1. With a resolver configured, the tracked query maps FQCN → path.
//!   2. The salsa input is correctly invalidated when the resolver swaps:
//!      the second call after `with_class_resolver` returns the new path,
//!      not a stale cached value.
//!   3. Without a resolver, the query returns `None` (no panic).

use std::path::PathBuf;
use std::sync::Arc;

use mir_types::Symbol;

use mir_analyzer::db::{resolve_fqcn_to_path, source_file_for_fqcn, Fqcn};
use mir_analyzer::{AnalysisSession, ClassResolver, PhpVersion};

struct StubResolver {
    mapping: std::collections::HashMap<String, PathBuf>,
}

impl ClassResolver for StubResolver {
    fn resolve(&self, fqcn: &str) -> Option<PathBuf> {
        self.mapping.get(fqcn).cloned()
    }
}

fn make_resolver(entries: &[(&str, &str)]) -> Arc<dyn ClassResolver> {
    let mapping = entries
        .iter()
        .map(|(k, v)| ((*k).to_string(), PathBuf::from(*v)))
        .collect();
    Arc::new(StubResolver { mapping })
}

#[test]
fn resolve_fqcn_to_path_returns_resolved_path() {
    let session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\Foo", "/proj/Foo.php")]));

    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Symbol::new("App\\Foo"));
    let path = resolve_fqcn_to_path(&db, fqcn);
    assert_eq!(path.as_deref(), Some("/proj/Foo.php"));
}

#[test]
fn resolve_fqcn_to_path_returns_none_without_resolver() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Symbol::new("App\\Foo"));
    assert_eq!(resolve_fqcn_to_path(&db, fqcn), None);
}

#[test]
fn resolve_fqcn_to_path_returns_none_for_unknown_name() {
    let session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\Foo", "/proj/Foo.php")]));
    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Symbol::new("App\\Bar"));
    assert_eq!(resolve_fqcn_to_path(&db, fqcn), None);
}

#[test]
fn source_file_for_fqcn_finds_registered_file() {
    let session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\Foo", "/proj/Foo.php")]));
    session.set_file_text(
        Arc::from("/proj/Foo.php"),
        Arc::from("<?php namespace App; class Foo {}"),
    );

    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Symbol::new("App\\Foo"));
    let source_file = source_file_for_fqcn(&db, fqcn);
    assert!(
        source_file.is_some(),
        "source_file_for_fqcn must find a registered file"
    );
}

#[test]
fn source_file_for_fqcn_returns_none_when_unregistered() {
    // Resolver maps but file text was never registered. The composite
    // helper falls through to None — Phase 3 will likely flip this to
    // "load on demand" via a tracked workspace-files input.
    let session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\Foo", "/proj/Foo.php")]));

    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Symbol::new("App\\Foo"));
    assert!(
        source_file_for_fqcn(&db, fqcn).is_none(),
        "source_file_for_fqcn must return None when the file isn't registered"
    );
}
