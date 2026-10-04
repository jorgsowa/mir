//! Analysis of deeply nested input must not overflow a small thread stack.
//!
//! Each case runs on a 512 KiB stack, far below what the recursion needs unguarded.

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, FileAnalyzer, PhpVersion};

const SMALL_STACK: usize = 512 * 1024;

fn analyze_on_small_stack(files: Vec<(&'static str, String)>) -> Vec<String> {
    std::thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(move || {
            let mut session = AnalysisSession::new(PhpVersion::LATEST);
            for (path, src) in &files {
                session.ingest_file(Arc::from(*path), Arc::from(src.as_str()));
            }
            let (path, src) = files.last().unwrap();
            let parsed =
                mir_analyzer::recursion::parse_versioned(src, PhpVersion::LATEST.parser_version());
            FileAnalyzer::new(&mut session)
                .analyze_diagnostics_only(
                    Arc::from(*path),
                    src,
                    &parsed.program,
                    &parsed.source_map,
                )
                .issues
                .iter()
                .map(|i| i.kind.name().to_string())
                .collect()
        })
        .unwrap()
        .join()
        .expect("analysis overflowed or panicked")
}

#[test]
fn long_fluent_method_chain() {
    let src = format!(
        "<?php\nclass B {{ public function a(): static {{ return $this; }} }}\n\
         function f(B $b): void {{ $b{}; }}\n",
        "->a()".repeat(5000)
    );
    let issues = analyze_on_small_stack(vec![("<test>", src)]);
    assert!(issues.is_empty(), "{issues:?}");
}

#[test]
fn long_left_associative_binary_expression() {
    let src = format!(
        "<?php\nfunction f(int $x): int {{ return $x{}; }}\n",
        " + $x".repeat(5000)
    );
    let issues = analyze_on_small_stack(vec![("<test>", src)]);
    assert!(issues.is_empty(), "{issues:?}");
}

#[test]
fn deeply_nested_statements() {
    let depth = 400;
    let src = format!(
        "<?php\nfunction f(bool $c): void {{ {}echo 1;{} }}\n",
        "if ($c) { ".repeat(depth),
        " }".repeat(depth)
    );
    analyze_on_small_stack(vec![("<test>", src)]);
}

#[test]
fn nested_cross_file_return_type_inference() {
    let depth = 150;
    let mut files: Vec<(&'static str, String)> = Vec::new();
    for i in 0..depth {
        let next = if i + 1 < depth {
            format!("return (new C{}())->m();", i + 1)
        } else {
            "return 1;".to_string()
        };
        let path: &'static str = Box::leak(format!("<c{i}>").into_boxed_str());
        files.push((
            path,
            format!("<?php\nclass C{i} {{ public function m() {{ {next} }} }}\n"),
        ));
    }
    files.push((
        "<main>",
        "<?php\nfunction f(): int { return (new C0())->m(); }\n".to_string(),
    ));
    analyze_on_small_stack(files);
}
