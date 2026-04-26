//! Target PHP language version.
//!
//! Used by the analyzer and stub loader to make version-conditional decisions
//! (e.g. filtering stub symbols by `@since`/`@removed` markers). The type is
//! `Copy` and stores only major/minor — patch level is parsed but discarded,
//! since language features track the minor release.
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhpVersion {
    major: u8,
    minor: u8,
}

impl PhpVersion {
    pub const LATEST: PhpVersion = PhpVersion::new(8, 5);

    pub const fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    pub const fn major(self) -> u8 {
        self.major
    }

    pub const fn minor(self) -> u8 {
        self.minor
    }

    /// Return `true` if a stub symbol annotated with `@since`/`@removed` is
    /// available at this target version.
    ///
    /// `@since X.Y` excludes targets `< X.Y`. `@removed X.Y` excludes
    /// targets `>= X.Y` (the symbol is gone *as of* that release). Tags that
    /// fail to parse, or whose major version is outside the plausible PHP
    /// range, are ignored — some extension stubs (newrelic, mongodb) put
    /// library versions there (`@since 9.12`, `@since 1.17`) which must not
    /// drive PHP-version filtering.
    pub fn includes_symbol(self, since: Option<&str>, removed: Option<&str>) -> bool {
        let parse_php = |s: &str| -> Option<PhpVersion> {
            let v = s.parse::<PhpVersion>().ok()?;
            // PHP majors so far: 4, 5, 7, 8 (no 6). Accept LATEST.major + 1 as
            // forward-compat headroom; reject everything else as a library
            // version.
            if v.major() >= 4 && v.major() <= PhpVersion::LATEST.major() {
                Some(v)
            } else {
                None
            }
        };
        if let Some(s) = since.and_then(parse_php) {
            if self < s {
                return false;
            }
        }
        if let Some(r) = removed.and_then(parse_php) {
            if self >= r {
                return false;
            }
        }
        true
    }
}

impl Default for PhpVersion {
    fn default() -> Self {
        Self::LATEST
    }
}

impl fmt::Display for PhpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid PHP version `{0}`: expected `MAJOR.MINOR` (e.g. `8.2`)")]
pub struct ParsePhpVersionError(pub String);

impl FromStr for PhpVersion {
    type Err = ParsePhpVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.trim().split('.');
        let major = parts
            .next()
            .and_then(|p| p.parse::<u8>().ok())
            .ok_or_else(|| ParsePhpVersionError(s.to_string()))?;
        let minor = parts.next().and_then(|p| p.parse::<u8>().ok()).unwrap_or(0);
        // Ignore any patch component — language features track the minor release.
        Ok(Self::new(major, minor))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_major_minor() {
        assert_eq!("8.2".parse::<PhpVersion>().unwrap(), PhpVersion::new(8, 2));
    }

    #[test]
    fn parses_major_minor_patch() {
        assert_eq!(
            "8.3.7".parse::<PhpVersion>().unwrap(),
            PhpVersion::new(8, 3)
        );
    }

    #[test]
    fn parses_major_only() {
        assert_eq!("7".parse::<PhpVersion>().unwrap(), PhpVersion::new(7, 0));
    }

    #[test]
    fn rejects_garbage() {
        assert!("x.y".parse::<PhpVersion>().is_err());
        assert!("".parse::<PhpVersion>().is_err());
    }

    #[test]
    fn ordered_by_major_then_minor() {
        assert!(PhpVersion::new(8, 1) < PhpVersion::new(8, 2));
        assert!(PhpVersion::new(7, 4) < PhpVersion::new(8, 0));
    }

    #[test]
    fn displays_as_major_dot_minor() {
        assert_eq!(PhpVersion::new(8, 3).to_string(), "8.3");
    }

    #[test]
    fn includes_symbol_respects_since() {
        assert!(!PhpVersion::new(7, 4).includes_symbol(Some("8.0"), None));
        assert!(PhpVersion::new(8, 0).includes_symbol(Some("8.0"), None));
        assert!(PhpVersion::new(8, 5).includes_symbol(Some("8.0"), None));
    }

    #[test]
    fn includes_symbol_respects_removed() {
        assert!(PhpVersion::new(7, 4).includes_symbol(None, Some("8.0")));
        assert!(!PhpVersion::new(8, 0).includes_symbol(None, Some("8.0")));
        assert!(!PhpVersion::new(8, 5).includes_symbol(None, Some("8.0")));
    }

    #[test]
    fn includes_symbol_ignores_library_versions() {
        // newrelic uses `@since 9.12` for its own version; must not exclude on
        // PHP 8.5.
        assert!(PhpVersion::new(8, 5).includes_symbol(Some("9.12"), None));
        // mongodb uses `@since 1.17` for its driver version; harmless on its
        // own, but exercise the cap explicitly.
        assert!(PhpVersion::new(8, 5).includes_symbol(Some("1.17"), None));
        assert!(PhpVersion::new(8, 5).includes_symbol(Some("12.0"), None));
    }

    #[test]
    fn includes_symbol_ignores_garbage() {
        assert!(PhpVersion::new(8, 5).includes_symbol(Some("PECL"), None));
        assert!(PhpVersion::new(8, 5).includes_symbol(Some(""), None));
    }
}
