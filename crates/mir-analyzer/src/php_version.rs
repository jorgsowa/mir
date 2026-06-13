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

    /// Encode the version into the single byte the [`crate::stub_cache`]
    /// header carries. Layout: `(major << 4) | (minor & 0x0F)`. PHP minor
    /// versions stay well below 16 so they fit unambiguously in the low
    /// nibble.
    pub const fn cache_byte(self) -> u8 {
        (self.major << 4) | (self.minor & 0x0F)
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

    /// Whether `self` falls within the **inclusive** `[from, to]` range used by
    /// phpstorm-stubs' `#[PhpStormStubsElementAvailable($from, $to)]`. Both
    /// bounds are inclusive (verified empirically: `Error::__clone` is declared
    /// `from:"7.0", to:"8.0"` then `'8.1'`, and the only gap-free partition is
    /// inclusive `to`). A `None` bound is open on that side.
    ///
    /// Unlike [`includes_symbol`](Self::includes_symbol) this is *not* the
    /// `@since`/`@removed` semantics — `removed` there is exclusive. Version
    /// strings that fail to parse are ignored (treated as absent), defensively;
    /// `PhpVersion: FromStr` already truncates any `x.y.z` patch component.
    pub fn in_range(self, from: Option<&str>, to_inclusive: Option<&str>) -> bool {
        let parse = |s: &str| s.parse::<PhpVersion>().ok();
        if let Some(f) = from.and_then(parse) {
            if self < f {
                return false;
            }
        }
        if let Some(t) = to_inclusive.and_then(parse) {
            if self > t {
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
        let minor = match parts.next() {
            Some(p) => p
                .parse::<u8>()
                .map_err(|_| ParsePhpVersionError(s.to_string()))?,
            None => 0,
        };
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
        assert!("8.x".parse::<PhpVersion>().is_err());
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

    #[test]
    fn in_range_inclusive_both_bounds() {
        let v = PhpVersion::new;
        // [7.0, 8.0] inclusive on both ends.
        assert!(v(7, 0).in_range(Some("7.0"), Some("8.0")));
        assert!(v(8, 0).in_range(Some("7.0"), Some("8.0"))); // upper bound is inclusive
        assert!(v(7, 4).in_range(Some("7.0"), Some("8.0")));
        assert!(!v(8, 1).in_range(Some("7.0"), Some("8.0")));
        assert!(!v(6, 4).in_range(Some("7.0"), Some("8.0")));
    }

    #[test]
    fn in_range_open_bounds() {
        let v = PhpVersion::new;
        // from-only: available at and after 8.0.
        assert!(!v(7, 4).in_range(Some("8.0"), None));
        assert!(v(8, 0).in_range(Some("8.0"), None));
        assert!(v(8, 5).in_range(Some("8.0"), None));
        // to-only: available at and before 8.0.
        assert!(v(7, 4).in_range(None, Some("8.0")));
        assert!(v(8, 0).in_range(None, Some("8.0")));
        assert!(!v(8, 1).in_range(None, Some("8.0")));
        // no bounds: always available.
        assert!(v(7, 4).in_range(None, None));
    }

    #[test]
    fn in_range_across_the_7_4_to_8_0_jump() {
        let v = PhpVersion::new;
        // The 7.4 → 8.0 boundary: the gap-free partition for Error::__clone.
        assert!(v(7, 4).in_range(Some("7.0"), Some("8.0")));
        assert!(!v(7, 4).in_range(Some("8.1"), None));
        assert!(!v(8, 1).in_range(Some("7.0"), Some("8.0")));
        assert!(v(8, 1).in_range(Some("8.1"), None));
    }

    #[test]
    fn in_range_ignores_unparseable_bounds() {
        let v = PhpVersion::new(8, 2);
        assert!(v.in_range(Some("garbage"), None));
        assert!(v.in_range(None, Some("")));
        // patch components are truncated by FromStr.
        assert!(v.in_range(Some("8.0.1"), Some("8.3.9")));
    }
}
