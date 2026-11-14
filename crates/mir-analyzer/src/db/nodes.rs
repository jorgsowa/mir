use std::sync::Arc;

use mir_codebase::StubSlice;
use mir_issues::Issue;

// SourceFile input (S1)

/// Source file registered as a Salsa input.
/// Setting `contents` on an existing `SourceFile` is the single write that drives
/// all downstream query invalidation.
#[salsa::input]
pub struct SourceFile {
    pub path: Arc<str>,
    #[returns(ref)]
    pub contents: SourceText,
}

impl SourceFile {
    /// The file's text, decompressed when stored compressed.
    pub fn text<Db: ?Sized + salsa::Database>(self, db: &Db) -> Arc<str> {
        self.contents(db).text()
    }
}

/// A file's source, plain or LZ4-compressed. Clones share one allocation, and
/// [`Self::same`] identifies the version a freshness mark was taken from.
#[derive(Clone)]
pub struct SourceText(Arc<SourceTextRepr>);

struct SourceTextRepr {
    digest: std::sync::OnceLock<[u8; 32]>,
    len: usize,
    body: Body,
}

enum Body {
    Plain(Arc<str>),
    Compressed(Box<[u8]>),
}

/// Below this many bytes compression saves too little to pay for decompressing on every read.
const COMPRESS_MIN_LEN: usize = 512;

impl SourceText {
    /// Stored as is: reads are free, for texts read often or held only briefly.
    pub fn new(text: Arc<str>) -> Self {
        Self(Arc::new(SourceTextRepr {
            digest: std::sync::OnceLock::new(),
            len: text.len(),
            body: Body::Plain(text),
        }))
    }

    /// LZ4-compressed unless tiny, decompressed on every read: for texts held long and read rarely.
    pub fn compressed(text: Arc<str>) -> Self {
        if text.len() < COMPRESS_MIN_LEN {
            return Self::new(text);
        }
        Self(Arc::new(SourceTextRepr {
            digest: std::sync::OnceLock::from(crate::stub_cache::hash_source(&text)),
            len: text.len(),
            body: Body::Compressed(lz4_flex::block::compress(text.as_bytes()).into()),
        }))
    }

    /// [`Self::compressed`] when `compress`, else [`Self::new`].
    pub fn packed(text: Arc<str>, compress: bool) -> Self {
        if compress {
            Self::compressed(text)
        } else {
            Self::new(text)
        }
    }

    pub fn is_compressed(&self) -> bool {
        matches!(self.0.body, Body::Compressed(_))
    }

    pub fn text(&self) -> Arc<str> {
        match &self.0.body {
            Body::Plain(text) => text.clone(),
            Body::Compressed(bytes) => {
                let inflated = lz4_flex::block::decompress(bytes, self.0.len)
                    .expect("compressed by SourceText::compressed");
                Arc::from(String::from_utf8(inflated).expect("compressed from a str"))
            }
        }
    }

    /// The plain text without copying, or `None` when stored compressed.
    pub fn as_plain(&self) -> Option<&Arc<str>> {
        match &self.0.body {
            Body::Plain(text) => Some(text),
            Body::Compressed(_) => None,
        }
    }

    /// BLAKE3 digest of the text, as [`crate::stub_cache::hash_source`] computes it.
    pub fn digest(&self) -> &[u8; 32] {
        self.0.digest.get_or_init(|| match &self.0.body {
            Body::Plain(text) => crate::stub_cache::hash_source(text),
            Body::Compressed(_) => unreachable!("compressed texts hash on construction"),
        })
    }

    pub fn len(&self) -> usize {
        self.0.len
    }

    pub fn is_empty(&self) -> bool {
        self.0.len == 0
    }

    /// Whether `self` and `other` are the same stored version, not merely equal text.
    pub fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub fn matches(&self, text: &str) -> bool {
        if self.0.len != text.len() {
            return false;
        }
        match &self.0.body {
            Body::Plain(plain) => plain.as_ref() == text,
            Body::Compressed(_) => *self.digest() == crate::stub_cache::hash_source(text),
        }
    }
}

impl From<Arc<str>> for SourceText {
    fn from(text: Arc<str>) -> Self {
        Self::new(text)
    }
}

impl From<&str> for SourceText {
    fn from(text: &str) -> Self {
        Self::new(Arc::from(text))
    }
}

impl PartialEq for SourceText {
    fn eq(&self, other: &Self) -> bool {
        if self.same(other) {
            return true;
        }
        if self.0.len != other.0.len {
            return false;
        }
        match (&self.0.body, &other.0.body) {
            (Body::Plain(a), Body::Plain(b)) => a == b,
            _ => self.digest() == other.digest(),
        }
    }
}

impl Eq for SourceText {}

impl std::hash::Hash for SourceText {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.digest().hash(state);
    }
}

impl std::fmt::Debug for SourceText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceText")
            .field("len", &self.0.len)
            .finish()
    }
}

// FileDefinitions (S1)

/// Result of the `collect_file_definitions` tracked query.
#[derive(Clone, Debug)]
pub struct FileDefinitions {
    pub slice: Arc<StubSlice>,
    pub issues: Arc<Vec<Issue>>,
}

impl FileDefinitions {
    /// FQ names this file defines (classes, interfaces, traits, enums,
    /// functions, constants, global vars). Lets callers that already hold a
    /// `FileDefinitions` derive the symbol set without re-running the
    /// `collect_file_definitions` salsa query.
    pub(crate) fn defined_symbols(&self) -> rustc_hash::FxHashSet<Arc<str>> {
        let mut out = rustc_hash::FxHashSet::default();
        for c in self.slice.classes.iter() {
            out.insert(c.fqcn.clone());
        }
        for i in self.slice.interfaces.iter() {
            out.insert(i.fqcn.clone());
        }
        for t in self.slice.traits.iter() {
            out.insert(t.fqcn.clone());
        }
        for e in self.slice.enums.iter() {
            out.insert(e.fqcn.clone());
        }
        for f in self.slice.functions.iter() {
            out.insert(f.fqn.clone());
        }
        for (name, _) in self.slice.constants.iter() {
            out.insert(name.clone());
        }
        for (name, _) in self.slice.global_vars.iter() {
            let gname: Arc<str> = Arc::from(name.strip_prefix('$').unwrap_or(name.as_ref()));
            out.insert(gname);
        }
        out
    }
}

impl PartialEq for FileDefinitions {
    fn eq(&self, other: &Self) -> bool {
        // Structural comparison on the slice lets salsa skip workspace_symbol_index
        // rebuilds when file text changed but definitions are unchanged
        // (e.g. comment edits, whitespace-only saves in LSP).
        // Issues are deliberately excluded: diagnostics changing doesn't affect
        // symbol lookups and shouldn't force the workspace index to rebuild.
        if Arc::ptr_eq(&self.slice, &other.slice) {
            return true;
        }
        *self.slice == *other.slice
    }
}

#[cfg(test)]
mod source_text_tests {
    use super::*;

    #[test]
    fn round_trips_and_compresses_large_text() {
        let text: Arc<str> =
            Arc::from("<?php\nclass A { public function f(): int { return 1; } }\n".repeat(64));
        let stored = SourceText::compressed(text.clone());
        assert_eq!(stored.text(), text);
        assert!(matches!(&stored.0.body, Body::Compressed(bytes) if bytes.len() < text.len() / 2));
        assert!(stored.matches(&text));
        assert!(!stored.matches("<?php"));
    }

    #[test]
    fn identity_is_per_version_equality_per_content() {
        let a = SourceText::from("<?php echo 1;");
        let b = SourceText::from("<?php echo 1;");
        assert!(a.same(&a.clone()));
        assert!(!a.same(&b));
        assert_eq!(a, b);
        assert_ne!(a, SourceText::from("<?php echo 2;"));
    }

    #[test]
    fn plain_and_compressed_forms_of_one_text_are_equal() {
        let text: Arc<str> = Arc::from("<?php\necho 'x';\n".repeat(64));
        let plain = SourceText::new(text.clone());
        let packed = SourceText::compressed(text.clone());
        assert!(plain.as_plain().is_some_and(|t| Arc::ptr_eq(t, &text)));
        assert!(packed.is_compressed());
        assert_eq!(plain, packed);
        assert_eq!(plain.digest(), packed.digest());
        assert!(plain.matches(&text) && packed.matches(&text));
        assert_ne!(
            plain,
            SourceText::compressed(Arc::from("<?php\n".repeat(128)))
        );
    }
}
