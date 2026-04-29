use std::sync::Arc;

use mir_codebase::StubSlice;
use mir_issues::Issue;

/// Salsa database trait for mir incremental analysis.
/// This is the landing pad for the Salsa migration; queries will be added
/// phase by phase as described in the migration plan.
#[salsa::db]
pub trait MirDatabase: salsa::Database {
    /// The PHP version configured for this analysis run.
    fn php_version_str(&self) -> Arc<str>;
}

/// Source file registered as a Salsa input.
/// Setting `text` on an existing `SourceFile` is the single write that drives
/// all downstream query invalidation.
#[salsa::input]
pub struct SourceFile {
    pub path: Arc<str>,
    pub text: Arc<str>,
}

/// Result of the `collect_file_definitions` tracked query.
///
/// Bundles the [`StubSlice`] produced by Pass 1 together with any parse errors
/// and definition-collector issues, so no diagnostics are silently dropped.
#[derive(Clone, Debug)]
pub struct FileDefinitions {
    pub slice: Arc<StubSlice>,
    pub issues: Arc<Vec<Issue>>,
}

/// Pointer equality: two results are "equal" only if they share the exact same
/// `Arc` allocations. Two separate query executions always produce new `Arc`s,
/// so this is effectively always-not-equal and triggers dependent re-execution
/// whenever the function is re-run. Combined with `Update` below, this gives
/// correct always-update semantics without needing `PartialEq` on `StubSlice`.
impl PartialEq for FileDefinitions {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.slice, &other.slice) && Arc::ptr_eq(&self.issues, &other.issues)
    }
}

/// Always-update semantics: Salsa re-runs dependents whenever a new value is
/// produced.  Structural equality on `StubSlice` would require deriving it
/// across all contained types in `mir-codebase` (adding a salsa dep there).
/// The always-update approach is correct and safe for S1; a fine-grained impl
/// can be added in a later phase once the migration stabilises.
unsafe impl salsa::Update for FileDefinitions {
    unsafe fn maybe_update(old_pointer: *mut Self, new_value: Self) -> bool {
        unsafe { *old_pointer = new_value };
        true
    }
}

/// Salsa tracked query: parse `file` and collect all PHP definitions.
///
/// Result is memoized per `SourceFile`; on warm runs with unchanged text the
/// parse + definition collection are skipped entirely.
#[salsa::tracked]
pub fn collect_file_definitions(db: &dyn MirDatabase, file: SourceFile) -> FileDefinitions {
    let path = file.path(db);
    let text = file.text(db);

    let arena = bumpalo::Bump::new();
    let parsed = php_rs_parser::parse(&arena, &text);

    let mut all_issues: Vec<Issue> = parsed
        .errors
        .iter()
        .map(|err| {
            Issue::new(
                mir_issues::IssueKind::ParseError {
                    message: err.to_string(),
                },
                mir_issues::Location {
                    file: path.clone(),
                    line: 1,
                    line_end: 1,
                    col_start: 0,
                    col_end: 0,
                },
            )
        })
        .collect();

    let collector =
        crate::collector::DefinitionCollector::new_for_slice(path, &text, &parsed.source_map);
    let (slice, collector_issues) = collector.collect_slice(&parsed.program);
    all_issues.extend(collector_issues);

    FileDefinitions {
        slice: Arc::new(slice),
        issues: Arc::new(all_issues),
    }
}

/// Concrete in-process Salsa database.
#[salsa::db]
#[derive(Default)]
pub struct MirDb {
    storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for MirDb {}

#[salsa::db]
impl MirDatabase for MirDb {
    fn php_version_str(&self) -> Arc<str> {
        Arc::from("8.2")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use salsa::Setter as _;

    #[test]
    fn mirdb_constructs() {
        let _db = MirDb::default();
    }

    #[test]
    fn source_file_input_roundtrip() {
        let db = MirDb::default();
        let file = SourceFile::new(&db, Arc::from("/tmp/test.php"), Arc::from("<?php echo 1;"));
        assert_eq!(file.path(&db).as_ref(), "/tmp/test.php");
        assert_eq!(file.text(&db).as_ref(), "<?php echo 1;");
    }

    #[test]
    fn collect_file_definitions_basic() {
        let db = MirDb::default();
        let src = Arc::from("<?php class Foo {}");
        let file = SourceFile::new(&db, Arc::from("/tmp/foo.php"), src);
        let defs = collect_file_definitions(&db, file);
        assert!(defs.issues.is_empty());
        assert_eq!(defs.slice.classes.len(), 1);
        assert_eq!(defs.slice.classes[0].fqcn.as_ref(), "Foo");
    }

    #[test]
    fn collect_file_definitions_memoized() {
        let db = MirDb::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/memo.php"),
            Arc::from("<?php class Bar {}"),
        );

        let defs1 = collect_file_definitions(&db, file);
        let defs2 = collect_file_definitions(&db, file);
        // Memoized result must share the same Arc allocations.
        assert!(
            Arc::ptr_eq(&defs1.slice, &defs2.slice),
            "unchanged file must return the memoized result"
        );
    }

    #[test]
    fn collect_file_definitions_recomputes_on_change() {
        let mut db = MirDb::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/memo2.php"),
            Arc::from("<?php class Foo {}"),
        );

        let defs1 = collect_file_definitions(&db, file);
        file.set_text(&mut db)
            .to(Arc::from("<?php class Foo {} class Bar {}"));
        let defs2 = collect_file_definitions(&db, file);

        // After a text change the query must re-run and produce a new Arc.
        assert!(
            !Arc::ptr_eq(&defs1.slice, &defs2.slice),
            "changed file must produce a new result"
        );
        assert_eq!(defs2.slice.classes.len(), 2);
    }
}
