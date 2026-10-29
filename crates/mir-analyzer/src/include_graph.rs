//! Include closure: the files reachable from a set of seeds through statically
//! resolvable `require`/`include` targets.

use crate::ast_ext::ByteStringExt as _;
use crate::ast_ext::NameBareExt as _;
use php_ast::ast::{BinaryOp, MagicConstKind};
use php_ast::owned::visitor::OwnedVisitor;
use php_ast::owned::{Expr, ExprKind, Stmt, StmtKind};
use std::ops::ControlFlow;
use std::path::{Component, Path, PathBuf};

use crate::file_extensions::PhpFileExtensions;

/// Whether include targets under a `vendor` directory join the closure.
///
/// Project walks [`Skip`](Self::Skip) them: composer autoloading already
/// reaches vendor code, and re-adding it would index it as project code.
/// A closure seeded from vendor files themselves (a files-autoload bootstrap's
/// sibling requires) must [`Follow`](Self::Follow), since every target lives
/// inside `vendor`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VendorTargets {
    Skip,
    Follow,
}

/// Returns `seeds` plus every file reachable from them through `require`/`include`
/// targets that resolve to a real file with a PHP extension in `extensions`,
/// recursing since a newly-added file may itself reach further files. Only
/// statically-resolvable target shapes are followed: a literal string, and
/// `__DIR__` / `dirname(__FILE__)` concatenated with a literal string, the common
/// manual-bootstrap idiom (e.g. `require_once __DIR__ . '/../legacy/bootstrap.php'`).
/// A bare literal with no `__DIR__` resolves relative to the including file's
/// own directory.
pub fn include_closure(
    seeds: Vec<PathBuf>,
    vendor: VendorTargets,
    extensions: &PhpFileExtensions,
) -> Vec<PathBuf> {
    let mut out = seeds;
    let mut seen: rustc_hash::FxHashSet<PathBuf> = out.iter().cloned().collect();
    let mut queue: Vec<PathBuf> = out.clone();
    while let Some(file) = queue.pop() {
        let Ok(text) = crate::read_php_source(&file) else {
            continue;
        };
        if !may_include(&text) {
            continue;
        }
        let parsed = php_rs_parser::parse(&text);
        for resolved in static_include_paths(&parsed.program, &file) {
            if !extensions.is_php_source(&resolved) {
                continue;
            }
            if vendor == VendorTargets::Skip
                && resolved.components().any(|c| c.as_os_str() == "vendor")
            {
                continue;
            }
            if !resolved.is_file() {
                continue;
            }
            if seen.insert(resolved.clone()) {
                out.push(resolved.clone());
                queue.push(resolved);
            }
        }
    }
    out
}

/// Cheap bailout: the (common) majority of files don't even mention require/include.
pub(crate) fn may_include(text: &str) -> bool {
    text.contains("require") || text.contains("include")
}

/// Paths the statically resolvable `require`/`include` targets in `program`
/// name, a relative one taken from `file`'s directory.
pub(crate) fn static_include_paths(program: &php_ast::owned::Program, file: &Path) -> Vec<PathBuf> {
    guarded_include_paths(program, file)
        .into_iter()
        .map(|(path, _)| path)
        .collect()
}

/// [`static_include_paths`], each with the lowest `PHP_VERSION_ID` the `if`
/// conditions around the include guarantee (0 when none do).
pub(crate) fn guarded_include_paths(
    program: &php_ast::owned::Program,
    file: &Path,
) -> Vec<(PathBuf, u32)> {
    let Some(dir) = file.parent() else {
        return Vec::new();
    };
    let mut scanner = IncludeTargetScanner {
        file,
        min_version_id: 0,
        targets: Vec::new(),
    };
    let _ = scanner.visit_program(program);
    scanner
        .targets
        .into_iter()
        .map(|(target, min_version_id)| {
            let resolved = if target.from_file || Path::new(&target.path).is_absolute() {
                PathBuf::from(target.path)
            } else {
                dir.join(target.path)
            };
            (lexically_normalize(&resolved), min_version_id)
        })
        .collect()
}

/// The analyzed files PHP never loads at `version_id`: every include of them
/// among `files` sits under a `PHP_VERSION_ID` check above it.
pub(crate) fn files_loaded_only_above<'f>(
    files: impl Iterator<
        Item = (
            &'f std::sync::Arc<str>,
            &'f str,
            &'f php_ast::owned::Program,
        ),
    >,
    version_id: u32,
) -> rustc_hash::FxHashSet<std::sync::Arc<str>> {
    let files: Vec<_> = files.collect();
    let by_path: rustc_hash::FxHashMap<PathBuf, &std::sync::Arc<str>> = files
        .iter()
        .map(|(file, _, _)| (lexically_normalize(Path::new(file.as_ref())), *file))
        .collect();
    let mut loaded_only_above: rustc_hash::FxHashMap<&std::sync::Arc<str>, bool> =
        Default::default();
    for (file, source, program) in &files {
        if !may_include(source) {
            continue;
        }
        for (path, min_version_id) in guarded_include_paths(program, Path::new(file.as_ref())) {
            if let Some(target) = by_path.get(&path) {
                *loaded_only_above.entry(target).or_insert(true) &= min_version_id > version_id;
            }
        }
    }
    loaded_only_above
        .into_iter()
        .filter(|(_, only_above)| *only_above)
        .map(|(file, _)| file.clone())
        .collect()
}

/// Collapses `.`/`..` components lexically instead of relying on the OS to
/// resolve them, since a `__DIR__`-derived base can be a Windows verbatim
/// (`\\?\`-prefixed) path — those disable `..` resolution by the OS, so a
/// naively-concatenated `\\?\C:\...\src/../legacy/helpers.php` would never
/// resolve to a real file even though the target genuinely exists.
fn lexically_normalize(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(_)) = components.peek().copied() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(_) => unreachable!(),
            Component::RootDir => ret.push(component.as_os_str()),
            // A leading `./` stays: analyzed files are keyed by the path as given.
            Component::CurDir => ret.push(component.as_os_str()),
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => ret.push(c),
        }
    }
    ret
}

struct IncludeTargetScanner<'a> {
    file: &'a Path,
    /// The lowest `PHP_VERSION_ID` the enclosing `if` branches guarantee.
    min_version_id: u32,
    targets: Vec<(IncludeTarget, u32)>,
}

impl IncludeTargetScanner<'_> {
    fn guarded(
        &mut self,
        min_version_id: u32,
        visit: impl FnOnce(&mut Self) -> ControlFlow<()>,
    ) -> ControlFlow<()> {
        let outer = self.min_version_id;
        self.min_version_id = outer.max(min_version_id);
        let flow = visit(self);
        self.min_version_id = outer;
        flow
    }
}

impl OwnedVisitor for IncludeTargetScanner<'_> {
    type Break = ();

    fn visit_stmt(&mut self, stmt: &Stmt) -> ControlFlow<()> {
        let StmtKind::If(if_stmt) = &stmt.kind else {
            return crate::recursion::walk_stmt(self, stmt);
        };
        self.visit_expr(&if_stmt.condition)?;
        self.guarded(implied_min_version_id(&if_stmt.condition, true), |s| {
            s.visit_stmt(&if_stmt.then_branch)
        })?;
        // Each later branch runs only once every earlier condition was false.
        let mut all_false = implied_min_version_id(&if_stmt.condition, false);
        for branch in if_stmt.elseif_branches.iter() {
            let then = all_false.max(implied_min_version_id(&branch.condition, true));
            self.guarded(all_false, |s| s.visit_expr(&branch.condition))?;
            self.guarded(then, |s| s.visit_stmt(&branch.body))?;
            all_false = all_false.max(implied_min_version_id(&branch.condition, false));
        }
        match &if_stmt.else_branch {
            Some(else_branch) => self.guarded(all_false, |s| s.visit_stmt(else_branch)),
            None => ControlFlow::Continue(()),
        }
    }

    fn visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        if let ExprKind::Include(_, inner) = &expr.kind {
            if let Some(target) = resolve_static_include_target(inner, self.file) {
                self.targets.push((target, self.min_version_id));
            }
        }
        crate::recursion::walk_expr(self, expr)
    }
}

/// The lowest `PHP_VERSION_ID` that `condition` evaluating to `is_true`
/// guarantees, from comparisons of `PHP_VERSION_ID` with an integer; 0 for none.
fn implied_min_version_id(condition: &Expr, is_true: bool) -> u32 {
    use php_ast::ast::UnaryPrefixOp;
    match &condition.kind {
        ExprKind::Parenthesized(inner) => implied_min_version_id(inner, is_true),
        ExprKind::UnaryPrefix(u) if u.op == UnaryPrefixOp::BooleanNot => {
            implied_min_version_id(&u.operand, !is_true)
        }
        ExprKind::Binary(b) => {
            let both = || {
                (
                    implied_min_version_id(&b.left, is_true),
                    implied_min_version_id(&b.right, is_true),
                )
            };
            match b.op {
                BinaryOp::BooleanAnd | BinaryOp::LogicalAnd if is_true => {
                    let (l, r) = both();
                    l.max(r)
                }
                BinaryOp::BooleanAnd | BinaryOp::LogicalAnd => {
                    let (l, r) = both();
                    l.min(r)
                }
                BinaryOp::BooleanOr | BinaryOp::LogicalOr if is_true => {
                    let (l, r) = both();
                    l.min(r)
                }
                BinaryOp::BooleanOr | BinaryOp::LogicalOr => {
                    let (l, r) = both();
                    l.max(r)
                }
                _ => version_comparison_min(b.op, &b.left, &b.right, is_true).unwrap_or(0),
            }
        }
        _ => 0,
    }
}

/// [`implied_min_version_id`] of `left op right` comparing `PHP_VERSION_ID` with an integer.
fn version_comparison_min(op: BinaryOp, left: &Expr, right: &Expr, is_true: bool) -> Option<u32> {
    let is_version_id = |e: &Expr| {
        e.kind
            .name_bare()
            .is_some_and(|name| name == "PHP_VERSION_ID")
    };
    let int = |e: &Expr| match e.kind {
        ExprKind::Int(n) => u32::try_from(n).ok(),
        _ => None,
    };
    // Normalized to `PHP_VERSION_ID op n`.
    let (op, n) = if is_version_id(left) {
        (op, int(right)?)
    } else if is_version_id(right) {
        let flipped = match op {
            BinaryOp::Less => BinaryOp::Greater,
            BinaryOp::LessOrEqual => BinaryOp::GreaterOrEqual,
            BinaryOp::Greater => BinaryOp::Less,
            BinaryOp::GreaterOrEqual => BinaryOp::LessOrEqual,
            other => other,
        };
        (flipped, int(left)?)
    } else {
        return None;
    };
    match (op, is_true) {
        (BinaryOp::GreaterOrEqual, true) | (BinaryOp::Less, false) => Some(n),
        (BinaryOp::Greater, true) | (BinaryOp::LessOrEqual, false) => n.checked_add(1),
        _ => None,
    }
}

/// Statically evaluates the include-target expression shapes real code
/// actually uses: a literal string, `__DIR__`, `__FILE__`, `dirname()` of those
/// (with an optional literal level), and `.` concatenations. Anything else (a
/// variable, another call) can't be resolved without running the program, so
/// is left alone.
struct IncludeTarget {
    path: String,
    /// Built from `__DIR__`/`__FILE__`, so already relative to where `file` is, not to its directory.
    from_file: bool,
}

fn resolve_static_include_target(expr: &Expr, file: &Path) -> Option<IncludeTarget> {
    let anchored = |path: String| {
        Some(IncludeTarget {
            path,
            from_file: true,
        })
    };
    match &expr.kind {
        ExprKind::String(s) => Some(IncludeTarget {
            path: s.as_str().to_string(),
            from_file: false,
        }),
        ExprKind::MagicConst(MagicConstKind::File) => anchored(file.to_string_lossy().into_owned()),
        ExprKind::MagicConst(MagicConstKind::Dir) => {
            anchored(file.parent()?.to_string_lossy().into_owned())
        }
        ExprKind::Parenthesized(inner) => resolve_static_include_target(inner, file),
        ExprKind::Binary(b) if b.op == BinaryOp::Concat => {
            let left = resolve_static_include_target(&b.left, file)?;
            let right = resolve_static_include_target(&b.right, file)?;
            Some(IncludeTarget {
                path: format!("{}{}", left.path, right.path),
                from_file: left.from_file,
            })
        }
        ExprKind::FunctionCall(call) => {
            let name = call.name.kind.name_bare()?;
            if !name.eq_ignore_ascii_case("dirname") || call.args.len() > 2 {
                return None;
            }
            let target = resolve_static_include_target(call.args.first()?.value()?, file)?;
            let levels = match call.args.get(1) {
                None => 1,
                Some(arg) => match arg.value()?.kind {
                    ExprKind::Int(n) if n >= 1 => n,
                    _ => return None,
                },
            };
            let mut path = Path::new(&target.path);
            for _ in 0..levels {
                path = path.parent()?;
            }
            // `dirname('pkg')` is `.` in PHP.
            let path = match path.to_string_lossy() {
                dir if dir.is_empty() => ".".to_string(),
                dir => dir.into_owned(),
            };
            Some(IncludeTarget {
                path,
                from_file: target.from_file,
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn includes(file: &str, code: &str) -> Vec<(PathBuf, u32)> {
        let parsed = php_rs_parser::parse(code);
        guarded_include_paths(&parsed.program, Path::new(file))
    }

    #[test]
    fn dir_relative_targets_keep_a_relative_file_path() {
        let found = includes(
            "./pkg/bootstrap80.php",
            "<?php if (\\PHP_VERSION_ID >= 80500) { require __DIR__.'/bootstrap85.php'; }
            require __FILE__;
            require dirname(__DIR__).'/autoload.php';
            require dirname(__FILE__, 2).'/up.php';
            require 'sibling.php';",
        );
        assert_eq!(
            found,
            vec![
                (PathBuf::from("./pkg/bootstrap85.php"), 80500),
                (PathBuf::from("./pkg/bootstrap80.php"), 0),
                (PathBuf::from("./autoload.php"), 0),
                (PathBuf::from("./up.php"), 0),
                (PathBuf::from("./pkg/sibling.php"), 0),
            ]
        );
    }

    #[test]
    fn dir_relative_targets_keep_a_bare_relative_file_path() {
        let found = includes("pkg/a.php", "<?php require __DIR__.'/b.php';");
        assert_eq!(found, vec![(PathBuf::from("pkg/b.php"), 0)]);
    }

    #[test]
    fn dir_relative_targets_keep_an_absolute_file_path() {
        let found = includes(
            "/app/pkg/a.php",
            "<?php require __DIR__.'/../b.php'; require 'c.php'; require '/etc/d.php';",
        );
        assert_eq!(
            found,
            vec![
                (PathBuf::from("/app/b.php"), 0),
                (PathBuf::from("/app/pkg/c.php"), 0),
                (PathBuf::from("/etc/d.php"), 0),
            ]
        );
    }
}
