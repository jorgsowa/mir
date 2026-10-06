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

/// Infers an implicit `@psalm-assert-if-true`/`-if-false` pair for a
/// niladic predicate method whose entire body is `return $this->prop !==
/// null;` (or `=== null`, either operand order) — the common
/// `isAssigned()`-style getter. Lets call sites narrow `$this->prop`'s
/// nullability the same way an explicit docblock assertion would, without
/// requiring the author to write one.
pub(super) fn synthesize_predicate_assertions(m: &php_ast::owned::MethodDecl) -> Vec<Assertion> {
    use php_ast::ast::BinaryOp;
    use php_ast::owned::{ExprKind, StmtKind};

    let Some(body) = m.body.as_ref() else {
        return Vec::new();
    };
    let [stmt] = body.stmts.as_ref() else {
        return Vec::new();
    };
    let StmtKind::Return(Some(expr)) = &stmt.kind else {
        return Vec::new();
    };
    let ExprKind::Binary(bin) = &expr.kind else {
        return Vec::new();
    };
    // `!==` means the property is asserted non-null when the method returns
    // true; `===` means the opposite.
    let not_null_if_true = match bin.op {
        BinaryOp::NotIdentical => true,
        BinaryOp::Identical => false,
        _ => return Vec::new(),
    };
    let prop_side = match (&bin.left.kind, &bin.right.kind) {
        (ExprKind::Null, _) => &bin.right,
        (_, ExprKind::Null) => &bin.left,
        _ => return Vec::new(),
    };
    let Some((obj, prop)) = crate::narrowing::extract_prop_access(prop_side) else {
        return Vec::new();
    };
    if obj != "this" {
        return Vec::new();
    }

    let param: Arc<str> = Arc::from(format!("this->{prop}"));
    vec![
        Assertion {
            kind: AssertionKind::AssertIfTrue,
            param: param.clone(),
            param_key: Vec::new(),
            ty: Type::null(),
            negated: not_null_if_true,
        },
        Assertion {
            kind: AssertionKind::AssertIfFalse,
            param,
            param_key: Vec::new(),
            ty: Type::null(),
            negated: !not_null_if_true,
        },
    ]
}

/// Gives an assertion-less method the `@psalm-assert $this->prop` assertions of a
/// same-class method it calls as a top-level `$this->callee();` statement,
/// unless a later top-level statement reassigns that property.
pub(super) fn propagate_delegated_assertions(
    members: &[php_ast::owned::ClassMember],
    own_methods: &mut mir_codebase::definitions::MemberMap<
        Arc<mir_codebase::definitions::MethodDef>,
    >,
) {
    use php_ast::owned::{ClassMemberKind, ExprKind, StmtKind};

    fn this_prop_assert(a: &Assertion) -> Option<&str> {
        if a.kind != AssertionKind::Assert || !a.param_key.is_empty() {
            return None;
        }
        a.param
            .strip_prefix("this->")
            .filter(|p| !p.contains("->") && !p.contains('['))
    }

    let methods: Vec<&php_ast::owned::MethodDecl> = members
        .iter()
        .filter_map(|m| match &m.kind {
            ClassMemberKind::Method(m) if !m.is_static => Some(m),
            _ => None,
        })
        .collect();

    // Each pass resolves one more hop of delegation (`a()` -> `b()` -> `c()`).
    for _ in 0..methods.len() {
        let mut changed = false;
        for m in &methods {
            let key = crate::util::php_ident_lowercase(m.name.as_deref().unwrap_or_default());
            let Some(body) = m.body.as_ref() else {
                continue;
            };
            if own_methods
                .get(key.as_str())
                .is_none_or(|d| !d.assertions.is_empty())
            {
                continue;
            }
            let mut inferred = Vec::new();
            for (i, stmt) in body.stmts.iter().enumerate() {
                let StmtKind::Expression(e) = &stmt.kind else {
                    continue;
                };
                let ExprKind::MethodCall(mc) = &e.kind else {
                    continue;
                };
                let (ExprKind::Variable(obj), ExprKind::Identifier(callee)) =
                    (&mc.object.kind, &mc.method.kind)
                else {
                    continue;
                };
                if obj.trim_start_matches('$') != "this" {
                    continue;
                }
                let Some(callee_def) =
                    own_methods.get(crate::util::php_ident_lowercase(callee).as_str())
                else {
                    continue;
                };
                for a in callee_def.assertions.iter() {
                    let Some(prop) = this_prop_assert(a) else {
                        continue;
                    };
                    let reassigned_later = body.stmts[i + 1..].iter().any(|s| {
                        let StmtKind::Expression(e) = &s.kind else {
                            return false;
                        };
                        let ExprKind::Assign(asg) = &e.kind else {
                            return false;
                        };
                        matches!(
                            crate::narrowing::extract_prop_access(&asg.target),
                            Some((o, p)) if o == "this" && p == prop
                        )
                    });
                    if !reassigned_later {
                        inferred.push(a.clone());
                    }
                }
            }
            if !inferred.is_empty() {
                if let Some(def) = own_methods.get_mut(key.as_str()) {
                    Arc::make_mut(def).assertions = inferred;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
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
    let offset_location = |start: u32, end: u32| {
        let s = source_map.offset_to_line_col(span_start + start);
        let e = source_map.offset_to_line_col(span_start + end);
        // Columns are char counts; docblock tag heads rarely hold wide chars, so bytes suffice.
        mir_issues::Location {
            file: file.clone(),
            line: s.line + 1,
            line_end: e.line + 1,
            col_start: s.col.min(u16::MAX as u32) as u16,
            col_end: e.col.min(u16::MAX as u32) as u16,
        }
    };
    for (msg, &(start, end)) in doc
        .invalid_annotations
        .iter()
        .zip(&doc.invalid_annotation_spans)
    {
        let issue = Issue::new(
            IssueKind::InvalidDocblock {
                message: msg.clone(),
            },
            offset_location(start, end),
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
