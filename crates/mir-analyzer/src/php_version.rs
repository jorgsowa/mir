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
}
