use std::path::{Path, PathBuf};

/// Return `true` if `dir` is inside a Composer vendor package directory.
///
/// Composer always places packages at `vendor/<org>/<pkg>/`, so a package root
/// has the path component `vendor` followed by at least 2 more segments.
/// `windows(3)` finds any three consecutive components whose first is `vendor`,
/// which matches `vendor/<org>/<pkg>` and deeper paths without accidentally
/// flagging a project root that merely lives one level under a directory named
/// `vendor` (e.g. `/srv/vendor/myapp/` — vendor+1 — forms no 3-window).
pub fn is_vendor_package_dir(dir: &Path) -> bool {
    let comps: Vec<_> = dir.components().collect();
    comps.windows(3).any(|w| w[0].as_os_str() == "vendor")
}

pub fn find_composer_root_for_path(path: &Path) -> Option<PathBuf> {
    let resolved = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let start = if resolved.is_dir() {
        resolved.as_path()
    } else {
        resolved.parent()?
    };

    // Skip any composer.json that lives inside a vendor/ subtree — those are
    // package manifests, not project roots. Keep walking up until we find a
    // composer.json that is not under any vendor/ ancestor.
    //
    // Known limitation: if vendor/ is a symlink, canonicalize() resolves it and
    // the resulting path may not contain a "vendor" component at all, causing
    // the walk to stop at the package's own composer.json.
    start
        .ancestors()
        .find(|dir| dir.join("composer.json").exists() && !is_vendor_package_dir(dir))
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::{find_composer_root_for_path, is_vendor_package_dir};
    use std::fs;
    use std::path::Path;

    fn temp_project(name: &str) -> std::path::PathBuf {
        let thread_name = std::thread::current()
            .name()
            .unwrap_or("test")
            .replace(|c: char| !c.is_ascii_alphanumeric(), "_");
        let root = std::env::temp_dir().join(format!(
            "mir_cli_{name}_{}_{}",
            std::process::id(),
            thread_name
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn composer_root_is_found_for_explicit_root_config_file() {
        let root = temp_project("root_config");
        fs::write(root.join("composer.json"), "{}").unwrap();
        fs::write(root.join(".php-cs-fixer.php"), "<?php\n").unwrap();

        let found = find_composer_root_for_path(&root.join(".php-cs-fixer.php"));

        assert_eq!(found, Some(root.canonicalize().unwrap()));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn composer_root_is_found_for_nested_file() {
        let root = temp_project("nested_file");
        let nested = root.join("src/App");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("composer.json"), "{}").unwrap();
        fs::write(nested.join("Service.php"), "<?php\n").unwrap();

        let found = find_composer_root_for_path(&nested.join("Service.php"));

        assert_eq!(found, Some(root.canonicalize().unwrap()));
        let _ = fs::remove_dir_all(root);
    }

    // Regression: when a path inside vendor/ is given, the package's own
    // composer.json must be skipped and the project root returned instead.
    #[test]
    fn composer_root_skips_vendor_package_composer_json() {
        let root = temp_project("vendor_skip");
        let pkg_dir = root.join("vendor/laravel/framework/src/Illuminate");
        fs::create_dir_all(&pkg_dir).unwrap();
        fs::write(root.join("composer.json"), "{}").unwrap();
        fs::write(root.join("vendor/laravel/framework/composer.json"), "{}").unwrap();
        fs::write(pkg_dir.join("Support.php"), "<?php\n").unwrap();

        let found = find_composer_root_for_path(&pkg_dir.join("Support.php"));

        assert_eq!(found, Some(root.canonicalize().unwrap()));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn composer_root_skips_vendor_subtree_directory_path() {
        let root = temp_project("vendor_skip_dir");
        let illuminate = root.join("vendor/laravel/framework/src/Illuminate");
        fs::create_dir_all(&illuminate).unwrap();
        fs::write(root.join("composer.json"), "{}").unwrap();
        fs::write(root.join("vendor/laravel/framework/composer.json"), "{}").unwrap();

        let found = find_composer_root_for_path(&illuminate);

        assert_eq!(found, Some(root.canonicalize().unwrap()));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn composer_root_skips_nested_vendor_in_vendor() {
        let root = temp_project("nested_vendor");
        let deep = root.join("vendor/league/flysystem/vendor/mockery/prophecy/src");
        fs::create_dir_all(&deep).unwrap();
        fs::write(root.join("composer.json"), "{}").unwrap();
        fs::write(root.join("vendor/league/flysystem/composer.json"), "{}").unwrap();
        fs::write(
            root.join("vendor/league/flysystem/vendor/mockery/prophecy/composer.json"),
            "{}",
        )
        .unwrap();
        fs::write(deep.join("Prophecy.php"), "<?php\n").unwrap();

        let found = find_composer_root_for_path(&deep.join("Prophecy.php"));

        assert_eq!(found, Some(root.canonicalize().unwrap()));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn composer_root_is_found_when_project_is_one_level_under_vendor_named_dir() {
        let root = temp_project("under_vendor_dir");
        let project_dir = root.join("vendor/myapp");
        fs::create_dir_all(project_dir.join("src")).unwrap();
        fs::write(project_dir.join("composer.json"), "{}").unwrap();
        fs::write(project_dir.join("src/Service.php"), "<?php\n").unwrap();

        let found = find_composer_root_for_path(&project_dir.join("src/Service.php"));

        assert_eq!(found, Some(project_dir.canonicalize().unwrap()));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn composer_root_is_found_for_standalone_package_checkout() {
        let root = temp_project("standalone_pkg");
        let src = root.join("src/Illuminate");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            root.join("composer.json"),
            r#"{"name":"laravel/framework"}"#,
        )
        .unwrap();
        fs::write(src.join("Support.php"), "<?php\n").unwrap();

        let found = find_composer_root_for_path(&src.join("Support.php"));

        assert_eq!(found, Some(root.canonicalize().unwrap()));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn is_vendor_package_dir_true_for_package_root_and_deeper() {
        assert!(is_vendor_package_dir(Path::new("vendor/laravel/framework")));
        assert!(is_vendor_package_dir(Path::new(
            "vendor/laravel/framework/src/Illuminate"
        )));
        assert!(is_vendor_package_dir(Path::new(
            "vendor/league/flysystem/vendor/mockery/prophecy"
        )));
        assert!(is_vendor_package_dir(Path::new("/srv/vendor/myapp/src")));
    }

    #[test]
    fn is_vendor_package_dir_false_for_vendor_plus_one_and_above() {
        assert!(!is_vendor_package_dir(Path::new("vendor")));
        assert!(!is_vendor_package_dir(Path::new("vendor/laravel")));
        assert!(!is_vendor_package_dir(Path::new("/srv/vendor/myapp")));
        assert!(!is_vendor_package_dir(Path::new("src/App")));
        assert!(!is_vendor_package_dir(Path::new(".")));
        assert!(!is_vendor_package_dir(Path::new("vendor-plugins/myapp")));
    }
}
