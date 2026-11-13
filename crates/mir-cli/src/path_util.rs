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

/// [`normalize_for_compare`] over many paths, canonicalizing each distinct parent once.
pub(crate) fn normalize_all_for_compare(paths: &[PathBuf]) -> Vec<PathBuf> {
    use rayon::prelude::*;
    use std::collections::{HashMap, HashSet};

    let parents: HashSet<&Path> = paths.iter().filter_map(|p| p.parent()).collect();
    let canonical_parents: HashMap<&Path, PathBuf> = parents
        .into_par_iter()
        .map(|parent| (parent, normalize_for_compare(parent)))
        .collect();
    paths
        .par_iter()
        .map(|p| {
            // A path that exists and isn't a symlink canonicalizes to its parent's canonical
            // form plus its own name; anything else goes through realpath on its own.
            let plain_entry = p
                .symlink_metadata()
                .is_ok_and(|m| !m.file_type().is_symlink());
            match (plain_entry, p.parent(), p.file_name()) {
                (true, Some(parent), Some(name)) if !parent.as_os_str().is_empty() => {
                    canonical_parents[parent].join(name)
                }
                _ => normalize_for_compare(p),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_normalization_matches_per_path() {
        let dir = tempfile::TempDir::new().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("real/sub")).unwrap();
        std::fs::write(root.join("real/a.php"), "").unwrap();
        let mut paths = vec![
            root.join("real/a.php"),
            root.join("real/sub"),
            root.join("real/./a.php"),
            root.join("real/missing.php"),
            root.join("nowhere/b.php"),
        ];
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(root.join("real"), root.join("linked")).unwrap();
            std::os::unix::fs::symlink(root.join("real/a.php"), root.join("real/link.php"))
                .unwrap();
            paths.push(root.join("linked/a.php"));
            paths.push(root.join("real/link.php"));
        }
        let expected: Vec<PathBuf> = paths.iter().map(|p| normalize_for_compare(p)).collect();
        assert_eq!(normalize_all_for_compare(&paths), expected);
    }
}
