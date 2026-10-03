use std::path::Path;
use std::sync::Arc;

/// File extensions treated as PHP source, sorted and deduplicated. Defaults to `["php"]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhpFileExtensions(Arc<[String]>);

impl PhpFileExtensions {
    /// Strips a leading `.` and lowercases each entry; blank entries are
    /// dropped, and an empty result falls back to `["php"]`.
    pub fn new<I, S>(extensions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut normalized: Vec<String> = Vec::new();
        for ext in extensions {
            let ext = ext.as_ref().trim().trim_start_matches('.').to_lowercase();
            if !ext.is_empty() && !normalized.contains(&ext) {
                normalized.push(ext);
            }
        }
        if normalized.is_empty() {
            return Self::default();
        }
        normalized.sort();
        Self(normalized.into())
    }

    /// Whether `path`'s extension (case-insensitive) is one of these.
    pub fn is_php_source(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| self.0.iter().any(|x| x.eq_ignore_ascii_case(e)))
    }

    pub fn as_slice(&self) -> &[String] {
        &self.0
    }
}

impl Default for PhpFileExtensions {
    fn default() -> Self {
        Self(Arc::from(vec!["php".to_string()]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_dots_case_and_blanks() {
        let exts = PhpFileExtensions::new([".INC", "Module", " ", "inc"]);
        assert_eq!(exts.as_slice(), ["inc", "module"]);
    }

    #[test]
    fn empty_input_falls_back_to_php() {
        assert_eq!(
            PhpFileExtensions::new(Vec::<String>::new()),
            PhpFileExtensions::default()
        );
        assert_eq!(PhpFileExtensions::new(["", "."]).as_slice(), ["php"]);
    }

    #[test]
    fn equality_ignores_order() {
        assert_eq!(
            PhpFileExtensions::new(["php", "inc"]),
            PhpFileExtensions::new(["inc", "php"])
        );
    }

    #[test]
    fn matches_case_insensitively() {
        let exts = PhpFileExtensions::new(["php", "inc"]);
        assert!(exts.is_php_source(Path::new("a/B.INC")));
        assert!(exts.is_php_source(Path::new("a.php")));
        assert!(!exts.is_php_source(Path::new("a.module")));
        assert!(!exts.is_php_source(Path::new("noext")));
    }
}
