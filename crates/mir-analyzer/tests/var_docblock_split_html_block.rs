//! Regression test (php-lsp#235): a `@var` docblock immediately followed by
//! `?>` (closing the PHP block), then inline HTML, then a re-opening
//! `<?php` — a Yii2-style view template pattern — must still attach to the
//! variable it annotates.
//!
//! `find_preceding_docblock` (`parser/mod.rs`) located a preceding docblock
//! by checking whether the source up to the target offset, trimmed, ends
//! with `*/`. It only skipped trailing whitespace and modifier keywords,
//! never a closing `?>` tag or the inline HTML/re-opening `<?php` between it
//! and the docblock — so the annotation was silently dropped and `$model`'s
//! type (and its very existence as a declared variable, since there's no
//! assignment) was lost by the time the second `<?php` block ran, producing
//! a false-positive `UndefinedVariable`.

mod common;

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, FileAnalyzer, PhpVersion};

fn issues_for(source: &str) -> Vec<String> {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("view.php");
    session.ingest_file(file.clone(), Arc::from(source));

    let parsed = php_rs_parser::parse(source);
    assert!(
        parsed.errors.is_empty(),
        "parser errors in test source: {:?}",
        parsed.errors
    );

    FileAnalyzer::new(&session)
        .analyze(file, source, &parsed.program, &parsed.source_map)
        .issues
        .iter()
        .map(|i| i.kind.name().to_string())
        .collect()
}

#[test]
fn var_annotation_survives_split_php_html_block() {
    let src = r#"<?php
class Post { public string $title = ''; }
/** @var Post $model */
?>
<div>
<?php if ($model->title !== ''): ?>
    <h1><?= $model->title ?></h1>
<?php endif; ?>
</div>
"#;
    let issues = issues_for(src);
    assert!(
        !issues.iter().any(|k| k == "UndefinedVariable"),
        "the @var annotation should survive the ?>...<?php split; got {issues:?}"
    );
}

/// Contrast: without the annotation, `$model` genuinely is undefined and
/// must still be flagged — guards against the fix over-broadly suppressing
/// UndefinedVariable across any split block.
#[test]
fn undefined_variable_across_split_block_is_still_flagged_without_annotation() {
    let src = r#"<?php
class Post { public string $title = ''; }
?>
<div>
<?php if ($model->title !== ''): ?>
    <h1><?= $model->title ?></h1>
<?php endif; ?>
</div>
"#;
    let issues = issues_for(src);
    assert!(
        issues.iter().any(|k| k == "UndefinedVariable"),
        "with no annotation at all, $model must still be flagged; got {issues:?}"
    );
}
