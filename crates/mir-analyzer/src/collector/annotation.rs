use mir_codebase::definitions::{Assertion, AssertionKind};
use mir_issues::{Issue, IssueKind};
use mir_types::Type;
use std::sync::Arc;

pub(super) fn build_assertions(
    doc: &crate::parser::ParsedDocblock,
    resolve_union_doc_fn: impl Fn(Type) -> Type,
) -> Vec<Assertion> {
    let mut assertions = Vec::new();
    assertions.extend(
        doc.assertions
            .iter()
            .map(|(param, param_key, ty, negated)| Assertion {
                kind: AssertionKind::Assert,
                param: Arc::from(param.as_str()),
                param_key: param_key.clone(),
                ty: resolve_union_doc_fn(ty.clone()),
                negated: *negated,
            }),
    );
    assertions.extend(
        doc.assertions_if_true
            .iter()
            .map(|(param, param_key, ty, negated)| Assertion {
                kind: AssertionKind::AssertIfTrue,
                param: Arc::from(param.as_str()),
                param_key: param_key.clone(),
                ty: resolve_union_doc_fn(ty.clone()),
                negated: *negated,
            }),
    );
    assertions.extend(
        doc.assertions_if_false
            .iter()
            .map(|(param, param_key, ty, negated)| Assertion {
                kind: AssertionKind::AssertIfFalse,
                param: Arc::from(param.as_str()),
                param_key: param_key.clone(),
                ty: resolve_union_doc_fn(ty.clone()),
                negated: *negated,
            }),
    );
    assertions
}

pub(super) fn emit_docblock_issues(
    doc: &crate::parser::ParsedDocblock,
    span_start: u32,
    php_version: Option<crate::php_version::PhpVersion>,
    file: Arc<str>,
    source_map: &php_rs_parser::source_map::SourceMap,
    issues: &mut mir_issues::IssueBuffer,
) {
    if php_version.is_some()
        || (doc.invalid_annotations.is_empty() && doc.backslash_keyword_types.is_empty())
    {
        return;
    }
    let lc = source_map.offset_to_line_col(span_start);
    let line = lc.line + 1;
    let suppressed = doc.suppressed_issues.iter().any(|s| s == "InvalidDocblock");
    for msg in &doc.invalid_annotations {
        let issue = Issue::new(
            IssueKind::InvalidDocblock {
                message: msg.clone(),
            },
            mir_issues::Location {
                file: file.clone(),
                line,
                line_end: line,
                col_start: 0,
                col_end: 0,
            },
        );
        issues.add(if suppressed { issue.suppress() } else { issue });
    }
    let suppressed = doc
        .suppressed_issues
        .iter()
        .any(|s| s == "InvalidDocblockType");
    for o in &doc.backslash_keyword_types {
        // The docblock text's first line starts at the span's column
        // (indented docblocks); later lines start at column 0.
        let col = ((if o.line == 0 { lc.col + o.col } else { o.col }).min(u16::MAX as u32)) as u16;
        let token_line = line + o.line;
        let issue = Issue::new(
            IssueKind::InvalidDocblockType {
                message: format!(
                    "@{} backslash-qualified non-class type '{}' is not a fully qualified name",
                    o.tag, o.token
                ),
            },
            mir_issues::Location {
                file: file.clone(),
                line: token_line,
                line_end: token_line,
                col_start: col,
                col_end: (col as u32 + o.token.len() as u32).min(u16::MAX as u32) as u16,
            },
        );
        issues.add(if suppressed { issue.suppress() } else { issue });
    }
}
