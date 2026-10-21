use super::*;

pub(super) const MAX_TRAIL_STEPS: usize = 6;

pub(super) fn trail_step(expr: &Expr, source: &str) -> crate::flow_state::TrailStep {
    let text = source
        .get(expr.span.start as usize..expr.span.end as usize)
        .unwrap_or("")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let label = match text.char_indices().nth(60) {
        Some((cut, _)) => format!("{}…", &text[..cut]),
        None => text,
    };
    crate::flow_state::TrailStep {
        label: std::sync::Arc::from(label),
        span: expr.span,
    }
}

/// How `kinds` of source data reach `expr`, source first; empty when it carries none.
pub fn taint_trail(
    expr: &Expr,
    kinds: Taint,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
    source: &str,
) -> Vec<crate::flow_state::TrailStep> {
    if !expr_taint(expr, ctx, db, file).concrete().intersects(kinds) {
        return Vec::new();
    }
    let child = |e: &Expr| taint_trail(e, kinds, ctx, db, file, source);
    let first =
        |exprs: &mut dyn Iterator<Item = &Expr>| exprs.map(child).find(|trail| !trail.is_empty());
    let nested = match &expr.kind {
        ExprKind::Variable(name) => {
            let sym = mir_types::Name::from(name.trim_start_matches('$'));
            ctx.taint_trails.get(&sym).map(|trail| trail.to_vec())
        }
        ExprKind::Parenthesized(inner)
        | ExprKind::ErrorSuppress(inner)
        | ExprKind::Cast(_, inner)
        | ExprKind::Print(inner) => Some(child(inner)),
        ExprKind::Assign(a) => Some(child(&a.value)),
        ExprKind::ArrayAccess(aa) => match &aa.array.kind {
            ExprKind::Variable(name) if is_superglobal(name.trim_start_matches('$')) => None,
            _ => Some(child(&aa.array)),
        },
        ExprKind::Binary(op) => first(&mut [&*op.left, &*op.right].into_iter()),
        ExprKind::NullCoalesce(nc) => first(&mut [&*nc.left, &*nc.right].into_iter()),
        ExprKind::Ternary(t) => first(
            &mut [
                t.then_expr.as_deref().unwrap_or(&t.condition),
                &*t.else_expr,
            ]
            .into_iter(),
        ),
        ExprKind::InterpolatedString(parts)
        | ExprKind::Heredoc { parts, .. }
        | ExprKind::ShellExec(parts) => first(&mut parts.iter().filter_map(|p| match p {
            StringPart::Expr(e) => Some(e),
            StringPart::Literal(_) => None,
            _ => None,
        })),
        ExprKind::Array(elements) => first(
            &mut elements
                .iter()
                .flat_map(|el| el.key.iter().chain(std::iter::once(el.value()))),
        ),
        ExprKind::FunctionCall(fc) => {
            first(&mut fc.args.iter().filter_map(|a| a.value())).map(|mut trail| {
                trail.push(trail_step(expr, source));
                trail
            })
        }
        _ => None,
    };
    match nested.filter(|trail| !trail.is_empty()) {
        Some(trail) => trail,
        None => vec![trail_step(expr, source)],
    }
}

/// `trail` extended by the variable `target` now holding its data.
pub fn extend_trail(
    mut trail: Vec<crate::flow_state::TrailStep>,
    target: &Expr,
    source: &str,
) -> std::sync::Arc<[crate::flow_state::TrailStep]> {
    trail.push(trail_step(target, source));
    if trail.len() > MAX_TRAIL_STEPS {
        trail.drain(1..trail.len() - (MAX_TRAIL_STEPS - 1));
    }
    std::sync::Arc::from(trail)
}

impl crate::expr::ExpressionAnalyzer<'_> {
    /// How `kinds` of source data reach `value`, as issue trace steps.
    pub fn taint_trace(
        &self,
        value: &Expr,
        kinds: Taint,
        ctx: &FlowState,
    ) -> Option<Box<[mir_issues::TaintStep]>> {
        let trail = taint_trail(value, kinds, ctx, self.db, &self.file, self.source);
        (!trail.is_empty()).then(|| {
            trail
                .iter()
                .map(|step| mir_issues::TaintStep {
                    label: step.label.to_string(),
                    location: self.span_location(step.span),
                })
                .collect()
        })
    }

    /// Emit a taint issue, with how `value`'s data got there.
    pub fn emit_taint(
        &mut self,
        kind: mir_issues::IssueKind,
        span: php_ast::Span,
        value: &Expr,
        kinds: Taint,
        ctx: &FlowState,
    ) {
        let mut issue = self.build_issue(kind, mir_issues::Severity::Error, span);
        issue.taint_trace = self.taint_trace(value, kinds, ctx);
        self.issues.add(issue);
    }
}
