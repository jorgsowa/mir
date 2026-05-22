use rustc_hash::FxHashMap;

/// Stable numeric identity for a source file within one analysis session.
/// Assigned when the file is first registered; never reused within that session.
/// Small, `Copy`, and cheap to hash — used as HashMap keys in hot-path structures
/// (cache entries, reverse-dep graph) instead of heap-allocated path strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileId(u32);

/// Bidirectional mapping between file paths and stable [`FileId`]s.
///
/// IDs are assigned sequentially on first contact.  The mapping is append-only
/// within a session; IDs are never recycled.
#[derive(Debug, Default)]
pub struct FileIdMap {
    path_to_id: FxHashMap<Box<str>, FileId>,
    id_to_path: Vec<Box<str>>,
}

impl FileIdMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the [`FileId`] for `path`, assigning a fresh one if not yet seen.
    pub fn assign_or_get(&mut self, path: &str) -> FileId {
        if let Some(&id) = self.path_to_id.get(path) {
            return id;
        }
        let id = FileId(self.id_to_path.len() as u32);
        let key: Box<str> = Box::from(path);
        self.id_to_path.push(key.clone());
        self.path_to_id.insert(key, id);
        id
    }

    /// Look up the [`FileId`] for `path` without assigning a new one.
    pub fn get(&self, path: &str) -> Option<FileId> {
        self.path_to_id.get(path).copied()
    }

    /// Resolve a [`FileId`] back to its path string.
    pub fn path(&self, id: FileId) -> Option<&str> {
        self.id_to_path.get(id.0 as usize).map(|s| s.as_ref())
    }

    pub fn len(&self) -> usize {
        self.id_to_path.len()
    }

    pub fn is_empty(&self) -> bool {
        self.id_to_path.is_empty()
    }

    /// Iterate all `(FileId, path)` pairs in assignment order.
    pub fn iter(&self) -> impl Iterator<Item = (FileId, &str)> {
        self.id_to_path
            .iter()
            .enumerate()
            .map(|(i, p)| (FileId(i as u32), p.as_ref()))
    }
}
