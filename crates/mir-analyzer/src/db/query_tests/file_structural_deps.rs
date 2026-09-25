use std::sync::Arc;

use crate::{AnalysisSession, PhpVersion};

/// `file_structural_deps` is the salsa-tracked query that specifically walks
/// declarations (not body-level references, which are merged in separately by
/// `dependency_graph()` from the reference index). Testing it directly avoids
/// the reference-index path masking these gaps the way an end-to-end
/// `dependency_graph()` test would for widely-referenced constructs like
/// `implements`.
#[test]
fn file_structural_deps_includes_enum_method_param_type_hint() {
    use crate::db::{file_structural_deps, file_structural_symbols, MirDatabase};
    use mir_types::Name;

    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.set_file_text(
        Arc::from("/proj/Service.php"),
        Arc::from("<?php\nnamespace Vendor;\nclass Service {}\n"),
    );
    let consumer_src = "<?php\nnamespace Vendor;\nenum Suit {\n    case Hearts;\n    public function make(Service $s): void {}\n}\n";
    session.set_file_text(Arc::from("/proj/Consumer.php"), Arc::from(consumer_src));

    let db = session.snapshot_db();
    let sf = MirDatabase::lookup_source_file(&db, "/proj/Consumer.php")
        .expect("Consumer.php must be registered");
    let deps = file_structural_deps(&db, sf);
    assert!(
        deps.iter().any(|f| f.as_ref() == "/proj/Service.php"),
        "enum method param type hint must produce a structural dep on Service.php; \
         file_structural_deps never iterated defs.slice.enums. Got: {deps:?}"
    );
    let symbols = file_structural_symbols(&db, sf);
    let _: &std::sync::Arc<[Name]> = symbols;
    assert!(
        symbols.iter().any(|s| s.as_ref() == "Vendor\\Service"),
        "enum method param type hint must produce a structural symbol edge; got {symbols:?}"
    );
}

#[test]
fn file_structural_deps_includes_trait_property_and_method_type_hints() {
    use crate::db::{file_structural_deps, MirDatabase};

    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.set_file_text(
        Arc::from("/proj/Service.php"),
        Arc::from("<?php\nnamespace Vendor;\nclass Service {}\n"),
    );
    let consumer_src =
        "<?php\nnamespace Vendor;\ntrait HasService {\n    public Service $service;\n}\n";
    session.set_file_text(Arc::from("/proj/Consumer.php"), Arc::from(consumer_src));

    let db = session.snapshot_db();
    let sf = MirDatabase::lookup_source_file(&db, "/proj/Consumer.php")
        .expect("Consumer.php must be registered");
    let deps = file_structural_deps(&db, sf);
    assert!(
        deps.iter().any(|f| f.as_ref() == "/proj/Service.php"),
        "trait own-property type hint must produce a structural dep on Service.php; \
         file_structural_deps's trait branch only walked t.traits, never \
         t.own_properties/t.own_methods. Got: {deps:?}"
    );
}
