//! Shared helpers for comparing filesystem paths across platforms.

use std::path::{Path, PathBuf};

/// Strip Windows's `\\?\` verbatim-path prefix before prefix/equality comparisons.
/// `Path::canonicalize()` adds it on Windows (e.g. `composer::find_composer_root_for_path`
/// canonicalizes the CLI's target path), while paths resolved from `<ignoreFiles>`/
/// `<projectFiles>` config entries never go through `canonicalize()` — leaving two paths
/// that denote the same directory but compare unequal component-for-component
/// (`Prefix(VerbatimDisk)` vs `Prefix(Disk)`), so `starts_with`/`==` never match. No-op on
/// non-Windows platforms, where `canonicalize()` never rewrites the path this way.
#[cfg(windows)]
pub(crate) fn strip_verbatim_prefix(p: &Path) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{rest}"))
    } else if let Some(rest) = s.strip_prefix(r"\\?\") {
        PathBuf::from(rest)
    } else {
        p.to_path_buf()
    }
}

#[cfg(not(windows))]
pub(crate) fn strip_verbatim_prefix(p: &Path) -> PathBuf {
    p.to_path_buf()
}
