use std::path::Path;
use std::sync::Arc;

/// File extensions treated as PHP source. Defaults to `["php"]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileExtensions(Arc<[String]>);

impl FileExtensions {
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
        Self(normalized.into())
    }

    pub fn as_slice(&self) -> &[String] {
        &self.0
    }
}

impl Default for FileExtensions {
    fn default() -> Self {
        Self(Arc::from(vec!["php".to_string()]))
    }
}

/// Whether `path`'s extension (case-insensitive) is one of `extensions`.
pub fn has_php_extension(path: &Path, extensions: &FileExtensions) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| extensions.0.iter().any(|x| x.eq_ignore_ascii_case(e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_dots_case_and_blanks() {
        let exts = FileExtensions::new([".INC", "Module", " ", "inc"]);
        assert_eq!(exts.as_slice(), ["inc", "module"]);
    }

    #[test]
    fn empty_input_falls_back_to_php() {
        assert_eq!(
            FileExtensions::new(Vec::<String>::new()),
            FileExtensions::default()
        );
        assert_eq!(FileExtensions::new(["", "."]).as_slice(), ["php"]);
    }

    #[test]
    fn matches_case_insensitively() {
        let exts = FileExtensions::new(["php", "inc"]);
        assert!(has_php_extension(Path::new("a/B.INC"), &exts));
        assert!(has_php_extension(Path::new("a.php"), &exts));
        assert!(!has_php_extension(Path::new("a.module"), &exts));
        assert!(!has_php_extension(Path::new("noext"), &exts));
    }
}
