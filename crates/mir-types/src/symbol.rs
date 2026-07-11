use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// Interned string identity for PHP class FQCNs, method names, and other
/// identifiers that appear repeatedly across the type system.
///
/// Backed by the process-global [`ustr`] interner: equal string values share a
/// single heap allocation.  Equality is pointer-based (O(1)) rather than
/// content-based (O(n)).  `Name` is `Copy` — cloning is a pointer copy, not a
/// refcount increment.
///
/// ## Serde
/// Serialised as a plain string; deserialised by interning the string value.
/// Round-trips transparently through `bincode` / `serde_json`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Name(ustr::Ustr);

impl Name {
    #[inline]
    pub fn new(s: &str) -> Self {
        Self(ustr::ustr(s))
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// ASCII-lowercased twin of this name, memoized.
    ///
    /// PHP class and function names are case-insensitive for resolution, so
    /// every workspace symbol-index lookup needs the lowercase form. The
    /// naive `name.to_ascii_lowercase()` allocates a fresh `String` per call
    /// — measured at ~9% of total CLI CPU on Laravel-scale fixtures.
    ///
    /// This caches `self → lowercase(self)` in a process-global DashMap so
    /// every unique identifier is lowercased at most once. The result is
    /// itself a `Name`, so downstream HashMap lookups become `u64`-keyed
    /// (`ustr::Ustr` equality is pointer-eq, not content-eq).
    ///
    /// Fast path: if `self` is already all-lowercase, returns `self`
    /// directly without touching the cache.
    pub fn ascii_lowercase(self) -> Self {
        if self.as_str().bytes().all(|b| !b.is_ascii_uppercase()) {
            return self;
        }
        static CACHE: std::sync::OnceLock<
            dashmap::DashMap<ustr::Ustr, ustr::Ustr, rustc_hash::FxBuildHasher>,
        > = std::sync::OnceLock::new();
        let cache = CACHE.get_or_init(|| dashmap::DashMap::with_hasher(Default::default()));
        if let Some(v) = cache.get(&self.0) {
            return Name(*v);
        }
        // `to_ascii_lowercase` allocates but only on first sight of this
        // name; subsequent calls return from the cache.
        let lowered = ustr::ustr(&self.as_str().to_ascii_lowercase());
        // Normally bounded by the workspace's mixed-case identifier
        // vocabulary, but a long session of renames mints new names forever;
        // dropping the whole memo is cheap (entries are re-lowered on demand)
        // and keeps the map from growing without bound.
        const CACHE_CAP: usize = 1 << 16;
        if cache.len() >= CACHE_CAP {
            cache.clear();
        }
        cache.insert(self.0, lowered);
        Name(lowered)
    }
}

// ---------------------------------------------------------------------------
// Conversions
// ---------------------------------------------------------------------------

impl From<&str> for Name {
    #[inline]
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for Name {
    #[inline]
    fn from(s: String) -> Self {
        Self::new(&s)
    }
}

impl From<Arc<str>> for Name {
    #[inline]
    fn from(s: Arc<str>) -> Self {
        Self::new(&s)
    }
}

impl From<Name> for String {
    #[inline]
    fn from(s: Name) -> String {
        s.as_str().to_string()
    }
}

impl From<Name> for Arc<str> {
    #[inline]
    fn from(s: Name) -> Arc<str> {
        Arc::from(s.as_str())
    }
}

// ---------------------------------------------------------------------------
// Deref + AsRef
// ---------------------------------------------------------------------------

impl std::ops::Deref for Name {
    type Target = str;
    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for Name {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

// ---------------------------------------------------------------------------
// Comparisons
// ---------------------------------------------------------------------------

impl PartialEq<str> for Name {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<Name> for str {
    #[inline]
    fn eq(&self, other: &Name) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<String> for Name {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<Arc<str>> for Name {
    #[inline]
    fn eq(&self, other: &Arc<str>) -> bool {
        self.as_str() == other.as_ref()
    }
}

impl PartialEq<Name> for Arc<str> {
    #[inline]
    fn eq(&self, other: &Name) -> bool {
        self.as_ref() == other.as_str()
    }
}

// ---------------------------------------------------------------------------
// Display / Debug
// ---------------------------------------------------------------------------

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Name({:?})", self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Serde
// ---------------------------------------------------------------------------

impl Serialize for Name {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Use `Cow<str>` instead of `&str` so this round-trips through both
        // borrowable formats (`serde_json`, `bincode::deserialize(&bytes)`)
        // *and* streaming formats that cannot borrow (`bincode::deserialize_from(reader)`).
        // The stub-cache serializer uses the streaming variant, and `<&str>`
        // would error with `invalid type: string "...", expected a borrowed
        // string`, silently turning every cache hit into a miss.
        let s = std::borrow::Cow::<str>::deserialize(deserializer)?;
        Ok(Self::new(&s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Overflowing the lowercase memo must clear-and-continue, never return
    /// a wrong lowering for names cached before the clear.
    #[test]
    fn ascii_lowercase_correct_across_cache_overflow() {
        let early = Name::new("App\\EarlyService");
        assert_eq!(early.ascii_lowercase().as_str(), "app\\earlyservice");
        for i in 0..70_000 {
            let n = Name::new(&format!("App\\Storm{i}"));
            assert_eq!(
                n.ascii_lowercase().as_str(),
                format!("app\\storm{i}").as_str()
            );
        }
        assert_eq!(early.ascii_lowercase().as_str(), "app\\earlyservice");
    }
}
