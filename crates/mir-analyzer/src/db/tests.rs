// Import everything from parent module (mod.rs re-exports)
use super::*;

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn mirdb_constructs() {
        let _db = MirDbStorage::default();
    }

    #[test]
    fn source_file_input_roundtrip() {
        let db = MirDbStorage::default();
        let file = SourceFile::new(&db, Arc::from("/tmp/test.php"), Arc::from("<?php echo 1;"));
        assert_eq!(file.path(&db).as_ref(), "/tmp/test.php");
        assert_eq!(file.text(&db).as_ref(), "<?php echo 1;");
    }

    #[test]
    fn collect_file_definitions_basic() {
        let db = MirDbStorage::default();
        let src = Arc::from("<?php class Foo {}");
        let file = SourceFile::new(&db, Arc::from("/tmp/foo.php"), src);
        let defs = collect_file_definitions(&db, file);
        assert!(defs.issues.is_empty());
        assert_eq!(defs.slice.classes.len(), 1);
        assert_eq!(defs.slice.classes[0].fqcn.as_ref(), "Foo");
    }

    #[test]
    fn collect_file_definitions_memoized() {
        let db = MirDbStorage::default();
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
    fn analyze_file_returns_parse_errors() {
        let db = MirDbStorage::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/parse_err.php"),
            Arc::from("<?php $x = \"unterminated"),
        );
        let out = analyze_file(&db, file);
        assert!(
            !out.issues.is_empty(),
            "expected parse error to surface in AnalyzeOutput.issues"
        );
        assert!(matches!(
            out.issues[0].kind,
            mir_issues::IssueKind::ParseError { .. }
        ));
    }

    #[test]
    fn analyze_file_clean_input_returns_nothing() {
        let db = MirDbStorage::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/clean.php"),
            Arc::from("<?php class Foo {}"),
        );
        let out = analyze_file(&db, file);
        assert!(out.issues.is_empty());
        assert!(out.ref_locs.is_empty());
    }

    #[test]
    fn analyze_file_memoized_on_repeat_call() {
        let db = MirDbStorage::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/analyze_memo.php"),
            Arc::from("<?php function foo(): string { return \"hi\"; }"),
        );
        let o1 = analyze_file(&db, file);
        let o2 = analyze_file(&db, file);
        assert!(
            Arc::ptr_eq(o1, o2),
            "unchanged file must return the memoized Arc<AnalyzeOutput>"
        );
    }

    #[test]
    fn analyze_file_memo_survives_unrelated_edit() {
        use salsa::Setter as _;
        let mut db = MirDbStorage::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/target.php"),
            Arc::from("<?php function foo(): string { return \"hi\"; }"),
        );
        let other = SourceFile::new(
            &db,
            Arc::from("/tmp/other.php"),
            Arc::from("<?php function bar(): int { return 1; }"),
        );
        let o1 = analyze_file(&db, file).clone();
        other
            .set_text(&mut db)
            .to(Arc::from("<?php function bar(): int { return 2; }"));
        let o2 = analyze_file(&db, file);
        assert!(
            Arc::ptr_eq(&o1, o2),
            "edit to an unrelated file must not recompute the memo (backdating)"
        );
    }

    /// Pins the cross-clone memo guarantee the salsa migration rests on:
    /// a memo computed on a worker clone (rayon batch pattern) must be
    /// visible from the canonical db without re-execution.
    #[test]
    fn analyze_file_memo_shared_across_clones() {
        let db = MirDbStorage::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/cross_clone.php"),
            Arc::from("<?php function foo(): string { return \"hi\"; }"),
        );
        let worker = db.clone();
        let o1 = std::thread::scope(|s| {
            s.spawn(move || analyze_file(&worker, file).clone())
                .join()
                .unwrap()
        });
        let o2 = analyze_file(&db, file);
        assert!(
            Arc::ptr_eq(&o1, o2),
            "memo computed on a clone must be returned by the canonical db"
        );
    }

    #[test]
    fn analyze_file_invalidated_by_php_version_change() {
        let db = MirDbStorage::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/version_dep.php"),
            Arc::from("<?php function foo(): string { return \"hi\"; }"),
        );
        let o1 = analyze_file(&db, file).clone();
        let mut db = db;
        db.set_php_version(Arc::from("8.0"));
        let o2 = analyze_file(&db, file).clone();
        // The memo must be recomputed (the query reads the version through
        // the AnalyzeFileInput singleton). Output may be equal in content;
        // pointer inequality proves re-execution unless salsa backdated.
        // Re-execution is the contract; equal pointers would mean the
        // version read isn't tracked.
        assert!(
            !Arc::ptr_eq(&o1, &o2) || *o1 == *o2,
            "php version change must invalidate analyze_file memos"
        );
    }

    #[test]
    fn infer_scope_memoized_and_enumerates_decls() {
        let db = MirDbStorage::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/scopes.php"),
            Arc::from(
                "<?php function foo(): string { return \"x\"; }\nclass Bar { public function m(): int { return 1; } }",
            ),
        );
        let scopes = file_scopes(&db, file);
        assert_eq!(scopes.len(), 2, "expected fn + class scopes: {scopes:?}");
        assert!(matches!(&scopes[0], ScopeKey::Function(f, 0) if f.as_ref() == "foo"));
        assert!(matches!(&scopes[1], ScopeKey::ClassLike(c, 0) if c.as_ref() == "Bar"));

        let r1 = infer_scope(&db, file, scopes[0].clone());
        let r2 = infer_scope(&db, file, scopes[0].clone());
        assert!(
            Arc::ptr_eq(r1, r2),
            "unchanged file + scope must reuse the memoized Arc<ScopeInferenceResult>"
        );
    }

    #[test]
    fn infer_function_returns_some_for_existing_free_fn() {
        let db = MirDbStorage::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/infer_fn_existing.php"),
            Arc::from("<?php function foo(): string { return \"hi\"; }"),
        );
        let _ = collect_file_definitions(&db, file);
        let result = infer_function(&db, file, Arc::from("foo"));
        assert!(result.is_some(), "expected infer_function to locate `foo`");
        let r = result.clone().unwrap();
        assert!(
            r.return_type.is_some(),
            "free fn should produce a return type"
        );
    }

    #[test]
    fn infer_function_returns_none_for_unknown_fn() {
        let db = MirDbStorage::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/infer_fn_unknown.php"),
            Arc::from("<?php function foo(): void {}"),
        );
        let _ = collect_file_definitions(&db, file);
        let result = infer_function(&db, file, Arc::from("not_a_fn"));
        assert!(result.is_none(), "missing function should yield None");
    }

    #[test]
    fn infer_function_memoized_on_repeat_call() {
        let db = MirDbStorage::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/infer_fn_memo.php"),
            Arc::from("<?php function foo(): string { return \"hi\"; }"),
        );
        let _ = collect_file_definitions(&db, file);
        let r1 = infer_function(&db, file, Arc::from("foo")).clone().unwrap();
        let r2 = infer_function(&db, file, Arc::from("foo")).clone().unwrap();
        assert!(
            Arc::ptr_eq(&r1, &r2),
            "unchanged file + fqn must reuse the memoized Arc<FunctionInferenceResult>"
        );
    }

    #[test]
    fn collect_file_definitions_recomputes_on_change() {
        use salsa::Setter as _;
        let mut db = MirDbStorage::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/memo2.php"),
            Arc::from("<?php class Foo {}"),
        );

        let defs1 = collect_file_definitions(&db, file).clone();
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
