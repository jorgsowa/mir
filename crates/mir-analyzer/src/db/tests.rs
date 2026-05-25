// Import everything from parent module (mod.rs re-exports)
use super::*;

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::*;
    use std::sync::Arc;

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
        assert!(
            Arc::ptr_eq(&defs1.slice, &defs2.slice),
            "unchanged file must return the memoized result"
        );
    }

    #[test]
    fn analyze_file_accumulates_parse_errors() {
        let db = MirDb::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/parse_err.php"),
            Arc::from("<?php $x = \"unterminated"),
        );
        let input = AnalyzeFileInput::new(&db, Arc::from("8.2"));
        analyze_file(&db, file, input);
        let issues: Vec<&IssueAccumulator> = analyze_file::accumulated(&db, file, input);
        assert!(
            !issues.is_empty(),
            "expected parse error to surface as accumulated IssueAccumulator"
        );
        assert!(matches!(
            issues[0].0.kind,
            mir_issues::IssueKind::ParseError { .. }
        ));
    }

    #[test]
    fn analyze_file_clean_input_accumulates_nothing() {
        let db = MirDb::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/clean.php"),
            Arc::from("<?php class Foo {}"),
        );
        let input = AnalyzeFileInput::new(&db, Arc::from("8.2"));
        analyze_file(&db, file, input);
        let issues: Vec<&IssueAccumulator> = analyze_file::accumulated(&db, file, input);
        let refs: Vec<&RefLocAccumulator> = analyze_file::accumulated(&db, file, input);
        assert!(issues.is_empty());
        assert!(refs.is_empty());
    }

    #[test]
    fn infer_function_returns_some_for_existing_free_fn() {
        let db = MirDb::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/infer_fn_existing.php"),
            Arc::from("<?php function foo(): string { return \"hi\"; }"),
        );
        let _ = collect_file_definitions(&db, file);
        let input = AnalyzeFileInput::new(&db, Arc::from("8.2"));
        let result = infer_function(&db, file, Arc::from("foo"), input);
        assert!(result.is_some(), "expected infer_function to locate `foo`");
        let r = result.unwrap();
        assert!(
            r.return_type.is_some(),
            "free fn should produce a return type"
        );
    }

    #[test]
    fn infer_function_returns_none_for_unknown_fn() {
        let db = MirDb::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/infer_fn_unknown.php"),
            Arc::from("<?php function foo(): void {}"),
        );
        let _ = collect_file_definitions(&db, file);
        let input = AnalyzeFileInput::new(&db, Arc::from("8.2"));
        let result = infer_function(&db, file, Arc::from("not_a_fn"), input);
        assert!(result.is_none(), "missing function should yield None");
    }

    #[test]
    fn infer_function_memoized_on_repeat_call() {
        let db = MirDb::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/infer_fn_memo.php"),
            Arc::from("<?php function foo(): string { return \"hi\"; }"),
        );
        let _ = collect_file_definitions(&db, file);
        let input = AnalyzeFileInput::new(&db, Arc::from("8.2"));
        let r1 = infer_function(&db, file, Arc::from("foo"), input).unwrap();
        let r2 = infer_function(&db, file, Arc::from("foo"), input).unwrap();
        assert!(
            Arc::ptr_eq(&r1, &r2),
            "unchanged file + fqn must reuse the memoized Arc<FunctionInferenceResult>"
        );
    }

    #[test]
    fn collect_file_definitions_recomputes_on_change() {
        use salsa::Setter as _;
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

        assert!(
            !Arc::ptr_eq(&defs1.slice, &defs2.slice),
            "changed file must produce a new result"
        );
        assert_eq!(defs2.slice.classes.len(), 2);
    }
}
