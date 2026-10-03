//! Include closure: the files reachable from a set of seeds through statically
//! resolvable `require`/`include` targets.

use php_ast::ast::{BinaryOp, MagicConstKind};
use php_ast::owned::visitor::{walk_owned_expr, OwnedVisitor};
use php_ast::owned::{Expr, ExprKind};
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
        let Some(dir) = file.parent().map(Path::to_path_buf) else {
            continue;
        };
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        // Cheap bailout: skip the full parse for the (common) majority of
        // files that don't even mention require/include.
        if !text.contains("require") && !text.contains("include") {
            continue;
        }
        let parsed = php_rs_parser::parse(&text);
        let mut scanner = IncludeTargetScanner {
            file: &file,
            targets: Vec::new(),
        };
        let _ = scanner.visit_program(&parsed.program);
        for target in scanner.targets {
            let resolved = if Path::new(&target).is_absolute() {
                PathBuf::from(target)
            } else {
                dir.join(target)
            };
            let resolved = lexically_normalize(&resolved);
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
            Component::CurDir => {}
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
    targets: Vec<String>,
}

impl OwnedVisitor for IncludeTargetScanner<'_> {
    fn visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        if let ExprKind::Include(_, inner) = &expr.kind {
            if let Some(target) = resolve_static_include_target(inner, self.file) {
                self.targets.push(target);
            }
        }
        walk_owned_expr(self, expr)
    }
}

/// Statically evaluates the include-target expression shapes real code
/// actually uses: a literal string, `__DIR__`, `__FILE__`, `dirname()` of those
/// (with an optional literal level), and `.` concatenations. Anything else (a
/// variable, another call) can't be resolved without running the program, so
/// is left alone.
fn resolve_static_include_target(expr: &Expr, file: &Path) -> Option<String> {
    match &expr.kind {
        ExprKind::String(s) => Some(s.to_string()),
        ExprKind::MagicConst(MagicConstKind::File) => Some(file.to_string_lossy().into_owned()),
        ExprKind::MagicConst(MagicConstKind::Dir) => {
            Some(file.parent()?.to_string_lossy().into_owned())
        }
        ExprKind::Parenthesized(inner) => resolve_static_include_target(inner, file),
        ExprKind::Binary(b) if b.op == BinaryOp::Concat => {
            let left = resolve_static_include_target(&b.left, file)?;
            let right = resolve_static_include_target(&b.right, file)?;
            Some(format!("{left}{right}"))
        }
        ExprKind::FunctionCall(call) => {
            let ExprKind::Identifier(name) = &call.name.kind else {
                return None;
            };
            if !name.eq_ignore_ascii_case("dirname") || call.args.len() > 2 {
                return None;
            }
            let path = resolve_static_include_target(call.args.first()?.value.as_ref()?, file)?;
            let levels = match call.args.get(1) {
                None => 1,
                Some(arg) => match arg.value.as_ref()?.kind {
                    ExprKind::Int(n) if n >= 1 => n,
                    _ => return None,
                },
            };
            let mut path = Path::new(&path);
            for _ in 0..levels {
                path = path.parent()?;
            }
            Some(path.to_string_lossy().into_owned())
        }
        _ => None,
    }
}
