pub mod atomic;
pub mod display;
pub mod union;

pub use atomic::Atomic;
pub use atomic::Variance;
pub use union::Union;

// ---------------------------------------------------------------------------
// PhpVersion
// ---------------------------------------------------------------------------

/// A PHP major.minor version target.
///
/// Used to filter stubs so that only functions and classes available in the
/// target version are registered in the `Codebase`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PhpVersion {
    pub major: u8,
    pub minor: u8,
}

impl PhpVersion {
    pub const fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    /// Parse from `"8.1"`, `"7.4"`, etc.  Returns `None` on malformed input.
    pub fn parse(s: &str) -> Option<Self> {
        let (maj, min) = s.split_once('.')?;
        Some(Self::new(maj.parse().ok()?, min.parse().ok()?))
    }

    pub const PHP_74: Self = Self::new(7, 4);
    pub const PHP_80: Self = Self::new(8, 0);
    pub const PHP_81: Self = Self::new(8, 1);
    pub const PHP_82: Self = Self::new(8, 2);
    pub const PHP_83: Self = Self::new(8, 3);
    pub const PHP_84: Self = Self::new(8, 4);
}
