use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// Interned string identity for PHP class FQCNs, method names, and other
/// identifiers that appear repeatedly across the type system.
///
/// Backed by the process-global [`ustr`] interner: equal string values share a
/// single heap allocation.  Equality is pointer-based (O(1)) rather than
/// content-based (O(n)).  `Symbol` is `Copy` — cloning is a pointer copy, not a
/// refcount increment.
///
/// ## Serde
/// Serialised as a plain string; deserialised by interning the string value.
/// Round-trips transparently through `bincode` / `serde_json`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Symbol(ustr::Ustr);

impl Symbol {
    #[inline]
    pub fn new(s: &str) -> Self {
        Self(ustr::ustr(s))
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

// ---------------------------------------------------------------------------
// Conversions
// ---------------------------------------------------------------------------

impl From<&str> for Symbol {
    #[inline]
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for Symbol {
    #[inline]
    fn from(s: String) -> Self {
        Self::new(&s)
    }
}

impl From<Arc<str>> for Symbol {
    #[inline]
    fn from(s: Arc<str>) -> Self {
        Self::new(&s)
    }
}

impl From<Symbol> for String {
    #[inline]
    fn from(s: Symbol) -> String {
        s.as_str().to_string()
    }
}

impl From<Symbol> for Arc<str> {
    #[inline]
    fn from(s: Symbol) -> Arc<str> {
        Arc::from(s.as_str())
    }
}

// ---------------------------------------------------------------------------
// Deref + AsRef
// ---------------------------------------------------------------------------

impl std::ops::Deref for Symbol {
    type Target = str;
    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for Symbol {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

// ---------------------------------------------------------------------------
// Comparisons
// ---------------------------------------------------------------------------

impl PartialEq<str> for Symbol {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<Symbol> for str {
    #[inline]
    fn eq(&self, other: &Symbol) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<String> for Symbol {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<Arc<str>> for Symbol {
    #[inline]
    fn eq(&self, other: &Arc<str>) -> bool {
        self.as_str() == other.as_ref()
    }
}

impl PartialEq<Symbol> for Arc<str> {
    #[inline]
    fn eq(&self, other: &Symbol) -> bool {
        self.as_ref() == other.as_str()
    }
}

// ---------------------------------------------------------------------------
// Display / Debug
// ---------------------------------------------------------------------------

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Symbol({:?})", self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Serde
// ---------------------------------------------------------------------------

impl Serialize for Symbol {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Symbol {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = <&str>::deserialize(deserializer)?;
        Ok(Self::new(s))
    }
}
