use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfFixtureKind {
    Laravel,
    Symfony,
}

#[derive(Debug, Clone)]
pub struct PerfFixture {
    kind: PerfFixtureKind,
    root: PathBuf,
}

impl PerfFixture {
    pub fn discover() -> Option<Self> {
        let manifest_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("benches/fixtures");
        let candidates = [
            std::env::var_os("MIR_PERF_FIXTURE").map(PathBuf::from),
            std::env::var_os("MIR_LARAVEL_FIXTURE").map(PathBuf::from),
            std::env::var_os("MIR_SYMFONY_FIXTURE").map(PathBuf::from),
            Some(manifest_root.join("laravel")),
            Some(manifest_root.join("symfony")),
        ];

        for root in candidates.into_iter().flatten() {
            if let Some(kind) = detect_fixture_kind(&root) {
                return Some(Self { kind, root });
            }
        }

        None
    }

    pub fn kind(&self) -> PerfFixtureKind {
        self.kind
    }

    pub fn id(&self) -> &'static str {
        match self.kind {
            PerfFixtureKind::Laravel => "laravel",
            PerfFixtureKind::Symfony => "symfony",
        }
    }

    pub fn label(&self) -> &'static str {
        match self.kind {
            PerfFixtureKind::Laravel => "Laravel",
            PerfFixtureKind::Symfony => "Symfony",
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn src_root(&self) -> PathBuf {
        self.root.join("src")
    }

    pub fn vendor_root(&self) -> PathBuf {
        self.root.join("vendor")
    }

    pub fn has_full_corpus(&self) -> bool {
        self.src_root().is_dir() && self.vendor_root().is_dir()
    }

    pub fn open_file(&self) -> PathBuf {
        self.root.join(self.open_file_rel())
    }

    pub fn open_file_label(&self) -> &'static str {
        match self.kind {
            PerfFixtureKind::Laravel => "Login.php",
            PerfFixtureKind::Symfony => "Request.php",
        }
    }

    pub fn open_symbol_probe(&self) -> &'static str {
        match self.kind {
            PerfFixtureKind::Laravel => "SerializesModels",
            PerfFixtureKind::Symfony => "Request",
        }
    }

    pub fn lazy_load_targets(&self) -> &'static [&'static str] {
        match self.kind {
            PerfFixtureKind::Laravel => &[
                "Illuminate\\Foundation\\Application",
                "Illuminate\\Database\\Eloquent\\Model",
                "Illuminate\\Support\\Collection",
                "Illuminate\\Http\\Request",
            ],
            PerfFixtureKind::Symfony => &[
                "Symfony\\Component\\Routing\\Router",
                "Symfony\\Component\\HttpFoundation\\RequestStack",
                "Symfony\\Component\\String\\Slugger\\AsciiSlugger",
                "Symfony\\Component\\String\\TruncateMode",
            ],
        }
    }

    pub fn high_fanout_file(&self) -> PathBuf {
        self.root.join(self.high_fanout_rel())
    }

    pub fn leaf_file(&self) -> PathBuf {
        self.root.join(self.leaf_rel())
    }

    pub fn read_query_target_class(&self) -> &'static str {
        match self.kind {
            PerfFixtureKind::Laravel => "Illuminate\\Database\\Eloquent\\Model",
            PerfFixtureKind::Symfony => "Symfony\\Component\\HttpFoundation\\Request",
        }
    }

    pub fn concurrent_target_class(&self) -> &'static str {
        match self.kind {
            PerfFixtureKind::Laravel => "Illuminate\\Auth\\Events\\Login",
            PerfFixtureKind::Symfony => "Symfony\\Component\\HttpFoundation\\Request",
        }
    }

    pub fn open_file_closure_candidates(&self) -> &'static [&'static str] {
        match self.kind {
            PerfFixtureKind::Laravel => &[
                "src/Illuminate/Database/Eloquent/Builder.php",
                "src/Illuminate/Routing/Router.php",
                "src/Illuminate/Http/Request.php",
            ],
            PerfFixtureKind::Symfony => &[
                "src/Symfony/Component/HttpFoundation/Request.php",
                "src/Symfony/Component/Routing/Router.php",
                "src/Symfony/Component/String/Slugger/AsciiSlugger.php",
            ],
        }
    }

    fn open_file_rel(&self) -> &'static str {
        match self.kind {
            PerfFixtureKind::Laravel => "src/Illuminate/Auth/Events/Login.php",
            PerfFixtureKind::Symfony => "src/Symfony/Component/HttpFoundation/Request.php",
        }
    }

    fn high_fanout_rel(&self) -> &'static str {
        match self.kind {
            PerfFixtureKind::Laravel => "src/Illuminate/Database/Eloquent/Model.php",
            PerfFixtureKind::Symfony => "src/Symfony/Component/HttpFoundation/Request.php",
        }
    }

    fn leaf_rel(&self) -> &'static str {
        match self.kind {
            PerfFixtureKind::Laravel => "src/Illuminate/Auth/Events/Login.php",
            PerfFixtureKind::Symfony => "src/Symfony/Component/String/TruncateMode.php",
        }
    }
}

fn detect_fixture_kind(root: &Path) -> Option<PerfFixtureKind> {
    if is_laravel_fixture(root) {
        return Some(PerfFixtureKind::Laravel);
    }
    if is_symfony_fixture(root) {
        return Some(PerfFixtureKind::Symfony);
    }
    None
}

fn is_laravel_fixture(root: &Path) -> bool {
    root.join("composer.json").is_file()
        && root.join("src/Illuminate/Auth/Events/Login.php").is_file()
        && root
            .join("src/Illuminate/Database/Eloquent/Model.php")
            .is_file()
}

fn is_symfony_fixture(root: &Path) -> bool {
    root.join("composer.json").is_file()
        && root
            .join("src/Symfony/Component/HttpFoundation/Request.php")
            .is_file()
        && root
            .join("src/Symfony/Component/String/TruncateMode.php")
            .is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_file(root: &Path, rel: &str) {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create fixture parent");
        }
        fs::write(path, "<?php\n").expect("write fixture file");
    }

    #[test]
    fn detects_minimal_laravel_fixture() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_file(dir.path(), "composer.json");
        write_file(dir.path(), "src/Illuminate/Auth/Events/Login.php");
        write_file(dir.path(), "src/Illuminate/Database/Eloquent/Model.php");

        assert_eq!(
            detect_fixture_kind(dir.path()),
            Some(PerfFixtureKind::Laravel)
        );
    }

    #[test]
    fn detects_minimal_symfony_fixture() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_file(dir.path(), "composer.json");
        write_file(
            dir.path(),
            "src/Symfony/Component/HttpFoundation/Request.php",
        );
        write_file(dir.path(), "src/Symfony/Component/String/TruncateMode.php");

        assert_eq!(
            detect_fixture_kind(dir.path()),
            Some(PerfFixtureKind::Symfony)
        );
    }

    #[test]
    fn preset_paths_match_kind() {
        let laravel = PerfFixture {
            kind: PerfFixtureKind::Laravel,
            root: PathBuf::from("/tmp/laravel"),
        };
        assert_eq!(
            laravel.open_file(),
            PathBuf::from("/tmp/laravel/src/Illuminate/Auth/Events/Login.php")
        );
        assert_eq!(
            laravel.high_fanout_file(),
            PathBuf::from("/tmp/laravel/src/Illuminate/Database/Eloquent/Model.php")
        );

        let symfony = PerfFixture {
            kind: PerfFixtureKind::Symfony,
            root: PathBuf::from("/tmp/symfony"),
        };
        assert_eq!(
            symfony.open_file(),
            PathBuf::from("/tmp/symfony/src/Symfony/Component/HttpFoundation/Request.php")
        );
        assert_eq!(
            symfony.leaf_file(),
            PathBuf::from("/tmp/symfony/src/Symfony/Component/String/TruncateMode.php")
        );
    }
}
