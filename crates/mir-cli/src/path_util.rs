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

/// Best-effort `canonicalize()` (falling back to `p` unchanged if the path doesn't exist
/// yet or the call errors) followed by [`strip_verbatim_prefix`]. Two paths that denote the
/// same directory can still disagree byte-for-byte before this — one may have gone through
/// `canonicalize()` upstream (e.g. `composer::find_composer_root_for_path`) and picked up a
/// resolved symlink, 8.3-short-name expansion, or on-disk casing that the other path (built
/// by plain `PathBuf::join` from a config-relative entry) never went through. Applying the
/// same normalization to both sides before `starts_with`/`==` is the only way to make them
/// comparable.
pub(crate) fn normalize_for_compare(p: &Path) -> PathBuf {
    strip_verbatim_prefix(&p.canonicalize().unwrap_or_else(|_| p.to_path_buf()))
}
