//! Pull-path workspace enumeration.
//!
//! A single `WorkspaceRevision` salsa input holds a monotonic counter
//! bumped whenever a file is added or removed (`upsert_source_file` /
//! `remove_source_file`). Edits to existing files don't bump the
//! revision; they invalidate `collect_file_definitions` directly.
//!
//! Tracked aggregators (`workspace_classes`, `workspace_functions`)
//! read `WorkspaceRevision::revision` to anchor on the set of files,
//! then enumerate via the off-salsa `source_files` registry and demand
//! `collect_file_definitions` / `collect_file_declarations` per file.

use std::ops::Range;
use std::sync::Arc;
use std::{fmt, fmt::Formatter};

use mir_types::Name;
use rustc_hash::FxHashMap;

use crate::db::{collect_file_definitions, MirDatabase, SourceFile};

#[salsa::input]
pub struct WorkspaceRevision {
    pub revision: u64,
}

#[salsa::tracked]
pub fn workspace_classes(db: &dyn MirDatabase) -> Arc<[Name]> {
    let rev = db
        .workspace_revision()
        .expect("WorkspaceRevision not initialized");
    let _ = rev.revision(db);

    let files = db.all_source_files();
    let mut out: Vec<Name> = Vec::new();
    for file in files.iter() {
        let decls = collect_file_declarations(db, *file);
        out.extend(decls.class_like().map(|decl| decl.name));
    }
    Arc::from(out)
}

#[salsa::tracked]
pub fn workspace_functions(db: &dyn MirDatabase) -> Arc<[Name]> {
    let rev = db
        .workspace_revision()
        .expect("WorkspaceRevision not initialized");
    let _ = rev.revision(db);

    let files = db.all_source_files();
    let mut out: Vec<Name> = Vec::new();
    for file in files.iter() {
        let decls = collect_file_declarations(db, *file);
        out.extend(decls.functions().map(|decl| decl.name));
    }
    Arc::from(out)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FileDecl {
    pub name: Name,
    key: Name,
    pub loc: SymbolLoc,
}

impl fmt::Debug for FileDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileDecl")
            .field("name", &self.name.as_str())
            .field("loc", &self.loc)
            .finish()
    }
}

impl FileDecl {
    pub fn lookup_key(self) -> Name {
        self.key
    }
}

#[derive(Clone)]
pub struct FileDeclarations {
    file: SourceFile,
    rows: Arc<[FileDeclRow]>,
    class_like: Range<usize>,
    functions: Range<usize>,
    constants: Range<usize>,
}

impl fmt::Debug for FileDeclarations {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileDeclarations")
            .field("class_like", &self.class_like().collect::<Vec<_>>())
            .field("functions", &self.functions().collect::<Vec<_>>())
            .field("constants", &self.constants().collect::<Vec<_>>())
            .finish()
    }
}

impl FileDeclarations {
    pub fn class_like_len(&self) -> usize {
        self.class_like.len()
    }

    pub fn function_len(&self) -> usize {
        self.functions.len()
    }

    pub fn constant_len(&self) -> usize {
        self.constants.len()
    }

    pub fn class_like(&self) -> impl ExactSizeIterator<Item = FileDecl> + '_ {
        self.rows[self.class_like.clone()]
            .iter()
            .map(|row| row.materialize(self.file))
    }

    pub fn functions(&self) -> impl ExactSizeIterator<Item = FileDecl> + '_ {
        self.rows[self.functions.clone()]
            .iter()
            .map(|row| row.materialize(self.file))
    }

    pub fn constants(&self) -> impl ExactSizeIterator<Item = FileDecl> + '_ {
        self.rows[self.constants.clone()]
            .iter()
            .map(|row| row.materialize(self.file))
    }

    pub fn class_like_at(&self, idx: usize) -> Option<FileDecl> {
        self.rows
            .get(self.class_like.start + idx)
            .map(|row| row.materialize(self.file))
            .filter(|_| idx < self.class_like.len())
    }

    pub fn function_at(&self, idx: usize) -> Option<FileDecl> {
        self.rows
            .get(self.functions.start + idx)
            .map(|row| row.materialize(self.file))
            .filter(|_| idx < self.functions.len())
    }

    pub fn constant_at(&self, idx: usize) -> Option<FileDecl> {
        self.rows
            .get(self.constants.start + idx)
            .map(|row| row.materialize(self.file))
            .filter(|_| idx < self.constants.len())
    }
}

impl PartialEq for FileDeclarations {
    fn eq(&self, other: &Self) -> bool {
        fn same_keys<'a>(
            left: impl Iterator<Item = FileDecl> + 'a,
            right: impl Iterator<Item = FileDecl> + 'a,
        ) -> bool {
            let mut left = left;
            let mut right = right;
            loop {
                match (left.next(), right.next()) {
                    (None, None) => return true,
                    (Some(a), Some(b)) if a.key == b.key => {}
                    _ => return false,
                }
            }
        }

        same_keys(self.class_like(), other.class_like())
            && same_keys(self.functions(), other.functions())
            && same_keys(self.constants(), other.constants())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct FileDeclRow {
    name: Name,
    key: Name,
    packed_loc: PackedSymbolLoc,
}

impl FileDeclRow {
    fn materialize(self, file: SourceFile) -> FileDecl {
        FileDecl {
            name: self.name,
            key: self.key,
            loc: self.packed_loc.materialize(file),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct PackedSymbolLoc(u32);

impl PackedSymbolLoc {
    const CLASS: u32 = 0;
    const INTERFACE: u32 = 1;
    const TRAIT: u32 = 2;
    const ENUM: u32 = 3;
    const FUNCTION: u32 = 4;
    const CONSTANT: u32 = 5;

    fn from_symbol_loc(loc: SymbolLoc) -> Self {
        let (kind, idx) = match loc {
            SymbolLoc::Class { idx, .. } => (Self::CLASS, idx as u32),
            SymbolLoc::Interface { idx, .. } => (Self::INTERFACE, idx as u32),
            SymbolLoc::Trait { idx, .. } => (Self::TRAIT, idx as u32),
            SymbolLoc::Enum { idx, .. } => (Self::ENUM, idx as u32),
            SymbolLoc::Function { idx, .. } => (Self::FUNCTION, idx as u32),
            SymbolLoc::Constant { idx, .. } => (Self::CONSTANT, idx as u32),
        };
        Self((idx << 3) | kind)
    }

    fn materialize(self, file: SourceFile) -> SymbolLoc {
        let idx = (self.0 >> 3) as usize;
        match self.0 & 0b111 {
            Self::CLASS => SymbolLoc::Class { file, idx },
            Self::INTERFACE => SymbolLoc::Interface { file, idx },
            Self::TRAIT => SymbolLoc::Trait { file, idx },
            Self::ENUM => SymbolLoc::Enum { file, idx },
            Self::FUNCTION => SymbolLoc::Function { file, idx },
            Self::CONSTANT => SymbolLoc::Constant { file, idx },
            _ => unreachable!("packed symbol kind must stay within 3 bits"),
        }
    }
}

struct FileDeclarationProjection {
    rows: Vec<FileDeclRow>,
    class_like: Range<usize>,
    functions: Range<usize>,
    constants: Range<usize>,
    structural_symbols: Vec<Arc<str>>,
}

#[salsa::tracked]
pub fn collect_file_declarations(db: &dyn MirDatabase, file: SourceFile) -> FileDeclarations {
    let defs = collect_file_definitions(db, file);
    decls_from_slice(&defs.slice, file)
}

pub fn decls_from_slice(
    slice: &mir_codebase::definitions::StubSlice,
    file: SourceFile,
) -> FileDeclarations {
    let projection = declaration_projection_from_slice(slice, file);
    FileDeclarations {
        file,
        rows: Arc::from(projection.rows),
        class_like: projection.class_like,
        functions: projection.functions,
        constants: projection.constants,
    }
}

pub(crate) fn structural_symbols_from_slice(
    slice: &mir_codebase::definitions::StubSlice,
    file: SourceFile,
) -> Arc<[Arc<str>]> {
    declaration_projection_from_slice(slice, file)
        .structural_symbols
        .into()
}

fn declaration_projection_from_slice(
    slice: &mir_codebase::definitions::StubSlice,
    file: SourceFile,
) -> FileDeclarationProjection {
    let mut rows = Vec::new();
    let mut structural_symbols = Vec::new();

    let push_named_objects = |out: &mut Vec<Arc<str>>, union: &mir_types::Type| {
        out.extend(union.types.iter().filter_map(|atomic| match atomic {
            mir_types::atomic::Atomic::TNamedObject { fqcn, .. } => {
                Some(Arc::<str>::from(fqcn.as_str()))
            }
            _ => None,
        }));
    };

    let class_like_start = rows.len();
    for (idx, c) in slice.classes.iter().enumerate() {
        rows.push(FileDeclRow {
            name: Name::new(c.fqcn.as_ref()),
            key: Name::new(c.fqcn.as_ref()).ascii_lowercase(),
            packed_loc: PackedSymbolLoc::from_symbol_loc(SymbolLoc::Class { file, idx }),
        });
        if let Some(parent) = &c.parent {
            structural_symbols.push(parent.clone());
        }
        structural_symbols.extend(c.interfaces.iter().cloned());
        structural_symbols.extend(c.traits.iter().cloned());
        for prop in c.own_properties.values() {
            if let Some(ty) = &prop.ty {
                push_named_objects(&mut structural_symbols, ty);
            }
        }
        for method in c.own_methods.values() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    push_named_objects(&mut structural_symbols, ty.as_ref());
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                push_named_objects(&mut structural_symbols, rt);
            }
        }
    }
    for (idx, i) in slice.interfaces.iter().enumerate() {
        rows.push(FileDeclRow {
            name: Name::new(i.fqcn.as_ref()),
            key: Name::new(i.fqcn.as_ref()).ascii_lowercase(),
            packed_loc: PackedSymbolLoc::from_symbol_loc(SymbolLoc::Interface { file, idx }),
        });
        structural_symbols.extend(i.extends.iter().cloned());
        for prop in i.own_properties.values() {
            if let Some(ty) = &prop.ty {
                push_named_objects(&mut structural_symbols, ty);
            }
        }
        for method in i.own_methods.values() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    push_named_objects(&mut structural_symbols, ty.as_ref());
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                push_named_objects(&mut structural_symbols, rt);
            }
        }
    }
    for (idx, t) in slice.traits.iter().enumerate() {
        rows.push(FileDeclRow {
            name: Name::new(t.fqcn.as_ref()),
            key: Name::new(t.fqcn.as_ref()).ascii_lowercase(),
            packed_loc: PackedSymbolLoc::from_symbol_loc(SymbolLoc::Trait { file, idx }),
        });
        structural_symbols.extend(t.traits.iter().cloned());
        for prop in t.own_properties.values() {
            if let Some(ty) = &prop.ty {
                push_named_objects(&mut structural_symbols, ty);
            }
        }
        for method in t.own_methods.values() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    push_named_objects(&mut structural_symbols, ty.as_ref());
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                push_named_objects(&mut structural_symbols, rt);
            }
        }
    }
    for (idx, e) in slice.enums.iter().enumerate() {
        rows.push(FileDeclRow {
            name: Name::new(e.fqcn.as_ref()),
            key: Name::new(e.fqcn.as_ref()).ascii_lowercase(),
            packed_loc: PackedSymbolLoc::from_symbol_loc(SymbolLoc::Enum { file, idx }),
        });
        structural_symbols.extend(e.interfaces.iter().cloned());
        structural_symbols.extend(e.traits.iter().cloned());
        for method in e.own_methods.values() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    push_named_objects(&mut structural_symbols, ty.as_ref());
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                push_named_objects(&mut structural_symbols, rt);
            }
        }
    }
    let class_like = class_like_start..rows.len();

    let function_start = rows.len();
    for (idx, f) in slice.functions.iter().enumerate() {
        rows.push(FileDeclRow {
            name: Name::new(f.fqn.as_ref()),
            key: Name::new(f.fqn.as_ref()).ascii_lowercase(),
            packed_loc: PackedSymbolLoc::from_symbol_loc(SymbolLoc::Function { file, idx }),
        });
        for param in f.params.iter() {
            if let Some(ty) = &param.ty {
                push_named_objects(&mut structural_symbols, ty.as_ref());
            }
        }
        if let Some(rt) = f.return_type.as_deref() {
            push_named_objects(&mut structural_symbols, rt);
        }
    }
    let functions = function_start..rows.len();

    let constant_start = rows.len();
    for (idx, (name, _)) in slice.constants.iter().enumerate() {
        rows.push(FileDeclRow {
            name: Name::new(name.as_ref()),
            key: Name::new(name.as_ref()),
            packed_loc: PackedSymbolLoc::from_symbol_loc(SymbolLoc::Constant { file, idx }),
        });
    }
    let constants = constant_start..rows.len();

    structural_symbols.extend(
        slice
            .imports
            .values()
            .map(|fqcn| Arc::<str>::from(fqcn.as_str())),
    );

    FileDeclarationProjection {
        rows,
        class_like,
        functions,
        constants,
        structural_symbols,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SymbolLoc {
    Class { file: SourceFile, idx: usize },
    Interface { file: SourceFile, idx: usize },
    Trait { file: SourceFile, idx: usize },
    Enum { file: SourceFile, idx: usize },
    Function { file: SourceFile, idx: usize },
    Constant { file: SourceFile, idx: usize },
}

impl fmt::Debug for SymbolLoc {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SymbolLoc::Class { idx, .. } => f.debug_tuple("Class").field(idx).finish(),
            SymbolLoc::Interface { idx, .. } => f.debug_tuple("Interface").field(idx).finish(),
            SymbolLoc::Trait { idx, .. } => f.debug_tuple("Trait").field(idx).finish(),
            SymbolLoc::Enum { idx, .. } => f.debug_tuple("Enum").field(idx).finish(),
            SymbolLoc::Function { idx, .. } => f.debug_tuple("Function").field(idx).finish(),
            SymbolLoc::Constant { idx, .. } => f.debug_tuple("Constant").field(idx).finish(),
        }
    }
}

impl SymbolLoc {
    pub fn file(&self) -> SourceFile {
        match self {
            SymbolLoc::Class { file, .. }
            | SymbolLoc::Interface { file, .. }
            | SymbolLoc::Trait { file, .. }
            | SymbolLoc::Enum { file, .. }
            | SymbolLoc::Function { file, .. }
            | SymbolLoc::Constant { file, .. } => *file,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SymbolTier {
    NativeStub = 0,
    UserFile = 1,
    UserStub = 2,
}

#[salsa::input]
pub struct WorkspaceSymbolIndexSingleton {
    pub index: WorkspaceSymbolIndex,
    pub revision: u64,
}

#[derive(Clone, Default)]
pub struct WorkspaceSymbolIndex {
    class_like: Arc<FxHashMap<Name, SymbolLoc>>,
    functions: Arc<FxHashMap<Name, SymbolLoc>>,
    constants: Arc<FxHashMap<Name, SymbolLoc>>,
    class_like_by_short_name: ShortNameIndex,
    class_like_collisions: Arc<FxHashMap<Name, Box<[SymbolLoc]>>>,
    function_collisions: Arc<FxHashMap<Name, Box<[SymbolLoc]>>>,
    constant_collisions: Arc<FxHashMap<Name, Box<[SymbolLoc]>>>,
}

#[derive(Clone, Default)]
pub struct ShortNameIndex {
    ranges: Arc<FxHashMap<Name, Range<u32>>>,
    postings: Arc<[Name]>,
}

impl ShortNameIndex {
    fn from_class_like_keys(keys: impl Iterator<Item = Name>) -> Self {
        let mut pairs: Vec<(Name, Name)> = keys.map(|key| (short_name_key(key), key)).collect();
        pairs.sort_by(|a, b| {
            a.0.as_str()
                .cmp(b.0.as_str())
                .then_with(|| a.1.as_str().cmp(b.1.as_str()))
        });

        let mut ranges = FxHashMap::default();
        let mut postings = Vec::with_capacity(pairs.len());
        let mut idx = 0;
        while idx < pairs.len() {
            let short = pairs[idx].0;
            let start = postings.len() as u32;
            while idx < pairs.len() && pairs[idx].0 == short {
                postings.push(pairs[idx].1);
                idx += 1;
            }
            let end = postings.len() as u32;
            ranges.insert(short, start..end);
        }

        Self {
            ranges: Arc::new(ranges),
            postings: Arc::from(postings),
        }
    }

    pub fn get(&self, short_name: Name) -> &[Name] {
        let Some(range) = self.ranges.get(&short_name) else {
            return &[];
        };
        &self.postings[range.start as usize..range.end as usize]
    }
}

impl PartialEq for ShortNameIndex {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.ranges, &other.ranges) && Arc::ptr_eq(&self.postings, &other.postings)
    }
}

impl WorkspaceSymbolIndex {
    pub fn class_like_len(&self) -> usize {
        self.class_like.len()
    }

    pub fn function_len(&self) -> usize {
        self.functions.len()
    }

    pub fn constant_len(&self) -> usize {
        self.constants.len()
    }

    pub fn class_like_loc(&self, key: Name) -> Option<SymbolLoc> {
        self.class_like.get(&key).copied()
    }

    pub fn function_loc(&self, key: Name) -> Option<SymbolLoc> {
        self.functions.get(&key).copied()
    }

    pub fn constant_loc(&self, key: Name) -> Option<SymbolLoc> {
        self.constants.get(&key).copied()
    }

    pub fn contains_class_like(&self, key: Name) -> bool {
        self.class_like.contains_key(&key)
    }

    pub fn contains_function(&self, key: Name) -> bool {
        self.functions.contains_key(&key)
    }

    pub fn iter_class_likes(&self) -> impl Iterator<Item = (Name, SymbolLoc)> + '_ {
        self.class_like.iter().map(|(k, v)| (*k, *v))
    }

    pub fn class_likes_named(&self, short_name: Name) -> &[Name] {
        self.class_like_by_short_name.get(short_name)
    }

    pub fn class_like_ptr(&self) -> *const FxHashMap<Name, SymbolLoc> {
        Arc::as_ptr(&self.class_like)
    }

    pub(crate) fn class_like_map(&self) -> &FxHashMap<Name, SymbolLoc> {
        &self.class_like
    }

    pub(crate) fn function_map(&self) -> &FxHashMap<Name, SymbolLoc> {
        &self.functions
    }

    pub(crate) fn constant_map(&self) -> &FxHashMap<Name, SymbolLoc> {
        &self.constants
    }

    pub(crate) fn class_like_collisions(&self) -> &FxHashMap<Name, Box<[SymbolLoc]>> {
        &self.class_like_collisions
    }

    pub(crate) fn function_collisions(&self) -> &FxHashMap<Name, Box<[SymbolLoc]>> {
        &self.function_collisions
    }

    pub(crate) fn constant_collisions(&self) -> &FxHashMap<Name, Box<[SymbolLoc]>> {
        &self.constant_collisions
    }
}

impl PartialEq for WorkspaceSymbolIndex {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.class_like, &other.class_like)
            && Arc::ptr_eq(&self.functions, &other.functions)
            && Arc::ptr_eq(&self.constants, &other.constants)
            && Arc::ptr_eq(&self.class_like_collisions, &other.class_like_collisions)
            && Arc::ptr_eq(&self.function_collisions, &other.function_collisions)
            && Arc::ptr_eq(&self.constant_collisions, &other.constant_collisions)
            && self.class_like_by_short_name == other.class_like_by_short_name
    }
}

pub fn build_workspace_symbol_index(
    class_like: FxHashMap<Name, SymbolLoc>,
    functions: FxHashMap<Name, SymbolLoc>,
    constants: FxHashMap<Name, SymbolLoc>,
    class_like_collisions: FxHashMap<Name, Vec<SymbolLoc>>,
    function_collisions: FxHashMap<Name, Vec<SymbolLoc>>,
    constant_collisions: FxHashMap<Name, Vec<SymbolLoc>>,
) -> WorkspaceSymbolIndex {
    fn freeze(map: FxHashMap<Name, Vec<SymbolLoc>>) -> Arc<FxHashMap<Name, Box<[SymbolLoc]>>> {
        Arc::new(
            map.into_iter()
                .filter_map(|(key, locs)| {
                    (locs.len() > 1).then_some((key, locs.into_boxed_slice()))
                })
                .collect(),
        )
    }

    WorkspaceSymbolIndex {
        class_like_by_short_name: ShortNameIndex::from_class_like_keys(class_like.keys().copied()),
        class_like: Arc::new(class_like),
        functions: Arc::new(functions),
        constants: Arc::new(constants),
        class_like_collisions: freeze(class_like_collisions),
        function_collisions: freeze(function_collisions),
        constant_collisions: freeze(constant_collisions),
    }
}

pub fn short_name_key(fqcn_lower: Name) -> Name {
    let s = fqcn_lower.as_str().trim_start_matches('\\');
    match s.rsplit_once('\\') {
        Some((_, short)) => Name::new(short),
        None => Name::new(s),
    }
}

pub fn workspace_index(db: &dyn MirDatabase) -> &WorkspaceSymbolIndex {
    if let Some(s) = db.workspace_symbol_index_singleton() {
        s.index(db)
    } else {
        workspace_symbol_index(db)
    }
}

#[salsa::tracked]
pub fn workspace_symbol_index(db: &dyn MirDatabase) -> WorkspaceSymbolIndex {
    let rev = db
        .workspace_revision()
        .expect("WorkspaceRevision not initialized");
    let _ = rev.revision(db);
    db.note_workspace_index_walk();

    let files = db.all_source_files();
    let mut class_like: FxHashMap<Name, SymbolLoc> = FxHashMap::default();
    let mut functions: FxHashMap<Name, SymbolLoc> = FxHashMap::default();
    let mut constants: FxHashMap<Name, SymbolLoc> = FxHashMap::default();
    let mut class_like_collisions: FxHashMap<Name, Vec<SymbolLoc>> = FxHashMap::default();
    let mut function_collisions: FxHashMap<Name, Vec<SymbolLoc>> = FxHashMap::default();
    let mut constant_collisions: FxHashMap<Name, Vec<SymbolLoc>> = FxHashMap::default();

    let user_stub_set: std::collections::HashSet<_> =
        db.user_stub_source_files().into_iter().collect();
    let (native_stubs, user_files): (Vec<SourceFile>, Vec<SourceFile>) = files
        .into_iter()
        .partition(|f| f.path(db).starts_with("stubs/"));

    for file in &native_stubs {
        let decls = collect_file_declarations(db, *file);
        for decl in decls.class_like() {
            insert_symbol(
                &mut class_like,
                &mut class_like_collisions,
                decl.lookup_key(),
                decl.loc,
                db,
                &user_stub_set,
            );
        }
        for decl in decls.functions() {
            insert_symbol(
                &mut functions,
                &mut function_collisions,
                decl.lookup_key(),
                decl.loc,
                db,
                &user_stub_set,
            );
        }
        for decl in decls.constants() {
            insert_symbol(
                &mut constants,
                &mut constant_collisions,
                decl.lookup_key(),
                decl.loc,
                db,
                &user_stub_set,
            );
        }
    }

    for file in &user_files {
        if user_stub_set.contains(file) {
            continue;
        }
        let decls = collect_file_declarations(db, *file);
        for decl in decls.class_like() {
            insert_symbol(
                &mut class_like,
                &mut class_like_collisions,
                decl.lookup_key(),
                decl.loc,
                db,
                &user_stub_set,
            );
        }
        for decl in decls.functions() {
            insert_symbol(
                &mut functions,
                &mut function_collisions,
                decl.lookup_key(),
                decl.loc,
                db,
                &user_stub_set,
            );
        }
        for decl in decls.constants() {
            insert_symbol(
                &mut constants,
                &mut constant_collisions,
                decl.lookup_key(),
                decl.loc,
                db,
                &user_stub_set,
            );
        }
    }

    for file in &user_stub_set {
        let decls = collect_file_declarations(db, *file);
        for decl in decls.class_like() {
            insert_symbol(
                &mut class_like,
                &mut class_like_collisions,
                decl.lookup_key(),
                decl.loc,
                db,
                &user_stub_set,
            );
        }
        for decl in decls.functions() {
            insert_symbol(
                &mut functions,
                &mut function_collisions,
                decl.lookup_key(),
                decl.loc,
                db,
                &user_stub_set,
            );
        }
        for decl in decls.constants() {
            insert_symbol(
                &mut constants,
                &mut constant_collisions,
                decl.lookup_key(),
                decl.loc,
                db,
                &user_stub_set,
            );
        }
    }

    build_workspace_symbol_index(
        class_like,
        functions,
        constants,
        class_like_collisions,
        function_collisions,
        constant_collisions,
    )
}

fn choose_winner(
    locs: &[SymbolLoc],
    db: &dyn MirDatabase,
    user_stubs: &std::collections::HashSet<SourceFile>,
) -> Option<SymbolLoc> {
    let mut best = None;
    for loc in locs.iter().copied() {
        let tier = symbol_tier(loc, db, user_stubs);
        let replace = match best {
            None => true,
            Some((best_tier, _)) if tier > best_tier => true,
            Some((best_tier, _)) if tier < best_tier => false,
            Some((SymbolTier::NativeStub, _)) => false,
            Some(_) => true,
        };
        if replace {
            best = Some((tier, loc));
        }
    }
    best.map(|(_, loc)| loc)
}

fn symbol_tier(
    loc: SymbolLoc,
    db: &dyn MirDatabase,
    user_stubs: &std::collections::HashSet<SourceFile>,
) -> SymbolTier {
    if user_stubs.contains(&loc.file()) {
        SymbolTier::UserStub
    } else if loc.file().path(db).starts_with("stubs/") {
        SymbolTier::NativeStub
    } else {
        SymbolTier::UserFile
    }
}

pub(crate) fn insert_symbol(
    winners: &mut FxHashMap<Name, SymbolLoc>,
    collisions: &mut FxHashMap<Name, Vec<SymbolLoc>>,
    key: Name,
    loc: SymbolLoc,
    db: &dyn MirDatabase,
    user_stubs: &std::collections::HashSet<SourceFile>,
) {
    if let Some(bucket) = collisions.get_mut(&key) {
        if !bucket.contains(&loc) {
            bucket.push(loc);
        }
        if let Some(winner) = choose_winner(bucket, db, user_stubs) {
            winners.insert(key, winner);
        }
        return;
    }

    match winners.get(&key).copied() {
        None => {
            winners.insert(key, loc);
        }
        Some(existing) if existing == loc => {}
        Some(existing) => {
            let bucket = collisions.entry(key).or_insert_with(|| vec![existing]);
            if !bucket.contains(&loc) {
                bucket.push(loc);
            }
            if let Some(winner) = choose_winner(bucket, db, user_stubs) {
                winners.insert(key, winner);
            }
        }
    }
}

pub(crate) fn remove_symbol(
    winners: &mut FxHashMap<Name, SymbolLoc>,
    collisions: &mut FxHashMap<Name, Vec<SymbolLoc>>,
    key: Name,
    loc: SymbolLoc,
    db: &dyn MirDatabase,
    user_stubs: &std::collections::HashSet<SourceFile>,
) {
    if let Some(bucket) = collisions.get_mut(&key) {
        bucket.retain(|candidate| *candidate != loc);
        match bucket.len() {
            0 => {
                collisions.remove(&key);
                winners.remove(&key);
            }
            1 => {
                let winner = bucket[0];
                collisions.remove(&key);
                winners.insert(key, winner);
            }
            _ => {
                if let Some(winner) = choose_winner(bucket, db, user_stubs) {
                    winners.insert(key, winner);
                }
            }
        }
        return;
    }

    if winners.get(&key).copied() == Some(loc) {
        winners.remove(&key);
    }
}

#[derive(Clone, Default, Debug)]
pub struct GlobalVarMap(pub Arc<FxHashMap<Arc<str>, mir_types::Type>>);

impl PartialEq for GlobalVarMap {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

#[salsa::tracked]
pub fn workspace_global_vars(db: &dyn MirDatabase) -> GlobalVarMap {
    let rev = db
        .workspace_revision()
        .expect("WorkspaceRevision not initialized");
    let _ = rev.revision(db);

    let files = db.all_source_files();
    let mut out: FxHashMap<Arc<str>, mir_types::Type> = FxHashMap::default();
    for file in files.iter() {
        let defs = collect_file_definitions(db, *file);
        for (name, ty) in &defs.slice.global_vars {
            let gname: Arc<str> = Arc::from(name.strip_prefix('$').unwrap_or(name.as_ref()));
            out.entry(gname).or_insert_with(|| ty.clone());
        }
    }
    GlobalVarMap(Arc::new(out))
}

#[cfg(test)]
mod builder_equivalence_tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn tracked_walk_matches_imperative_rebuild() {
        let mut db = crate::db::MirDbStorage::default();
        let native =
            "<?php\nclass Dup {}\nclass StubOnly {}\nfunction dup_fn() {}\nconst DUP = 1;\n";
        let native2 = "<?php\nclass Dup {}\n";
        let user1 =
            "<?php\nclass Dup {}\nclass UserOnly {}\nfunction dup_fn() {}\nconst DUP = 2;\n";
        let user2 = "<?php\nnamespace App;\nclass Dup {}\n";
        let ustub = "<?php\nclass Dup {}\nfunction dup_fn() {}\n";
        db.upsert_source_file_with_durability(
            Arc::from("stubs/std.php"),
            Arc::from(native),
            salsa::Durability::HIGH,
        );
        db.upsert_source_file_with_durability(
            Arc::from("stubs/extra.php"),
            Arc::from(native2),
            salsa::Durability::HIGH,
        );
        db.upsert_source_file_with_durability(
            Arc::from("user1.php"),
            Arc::from(user1),
            salsa::Durability::LOW,
        );
        db.upsert_source_file_with_durability(
            Arc::from("app.php"),
            Arc::from(user2),
            salsa::Durability::LOW,
        );
        db.upsert_source_file_with_durability(
            Arc::from("mystub.php"),
            Arc::from(ustub),
            salsa::Durability::LOW,
        );
        db.register_user_stub_path(Arc::from("mystub.php"));

        let tracked = workspace_symbol_index(&db).clone();
        db.rebuild_workspace_symbol_index();
        let rebuilt = workspace_index(&db).clone();

        fn maps_equal(
            a: impl Iterator<Item = (Name, SymbolLoc)>,
            b: impl Iterator<Item = (Name, SymbolLoc)>,
        ) -> bool {
            let a: FxHashMap<_, _> = a.collect();
            let b: FxHashMap<_, _> = b.collect();
            a == b
        }
        fn collision_map(
            db: &crate::db::MirDbStorage,
            map: &FxHashMap<Name, Box<[SymbolLoc]>>,
        ) -> FxHashMap<Name, Vec<(String, &'static str, usize)>> {
            map.iter()
                .map(|(key, locs)| {
                    let mut rows: Vec<_> = locs
                        .iter()
                        .map(|loc| {
                            let (kind, idx) = match loc {
                                SymbolLoc::Class { idx, .. } => ("class", *idx),
                                SymbolLoc::Interface { idx, .. } => ("interface", *idx),
                                SymbolLoc::Trait { idx, .. } => ("trait", *idx),
                                SymbolLoc::Enum { idx, .. } => ("enum", *idx),
                                SymbolLoc::Function { idx, .. } => ("function", *idx),
                                SymbolLoc::Constant { idx, .. } => ("constant", *idx),
                            };
                            (loc.file().path(db).to_string(), kind, idx)
                        })
                        .collect();
                    rows.sort();
                    (*key, rows)
                })
                .collect()
        }
        assert!(maps_equal(
            tracked.iter_class_likes(),
            rebuilt.iter_class_likes()
        ));
        assert_eq!(tracked.function_len(), rebuilt.function_len());
        assert_eq!(tracked.constant_len(), rebuilt.constant_len());
        let tracked_dup_short: Vec<_> = tracked
            .class_likes_named(Name::new("dup"))
            .iter()
            .map(|key| key.as_str())
            .collect();
        let rebuilt_dup_short: Vec<_> = rebuilt
            .class_likes_named(Name::new("dup"))
            .iter()
            .map(|key| key.as_str())
            .collect();
        assert_eq!(tracked_dup_short, rebuilt_dup_short);
        assert_eq!(rebuilt_dup_short, vec!["app\\dup", "dup"]);
        assert_eq!(
            collision_map(&db, tracked.class_like_collisions()),
            collision_map(&db, rebuilt.class_like_collisions())
        );
        assert_eq!(
            collision_map(&db, tracked.function_collisions()),
            collision_map(&db, rebuilt.function_collisions())
        );
        assert_eq!(
            collision_map(&db, tracked.constant_collisions()),
            collision_map(&db, rebuilt.constant_collisions())
        );

        let dup = tracked.class_like_loc(Name::new("dup")).unwrap();
        assert_eq!(dup.file().path(&db).as_ref(), "mystub.php");
        let dup_fn = tracked.function_loc(Name::new("dup_fn")).unwrap();
        assert_eq!(dup_fn.file().path(&db).as_ref(), "mystub.php");
        let dup_const = tracked.constant_loc(Name::new("DUP")).unwrap();
        assert_eq!(dup_const.file().path(&db).as_ref(), "user1.php");
    }
}

#[cfg(test)]
mod decl_projection_tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn decls_from_slice_matches_tracked_projection() {
        let mut db = crate::db::MirDbStorage::default();
        let text = "<?php\nnamespace App;\n\
            const TOP = 1;\n\
            interface I {}\n\
            trait T {}\n\
            enum E { case A; }\n\
            abstract class C implements I { use T; }\n\
            class D extends C {}\n\
            function f(): void {}\n\
            function g(int $x): int { return $x; }\n";
        let sf = db.upsert_source_file_with_durability(
            Arc::from("proj.php"),
            Arc::from(text),
            salsa::Durability::LOW,
        );

        let tracked = collect_file_declarations(&db, sf);
        let slice = collect_file_definitions(&db, sf).slice.clone();
        let projected = decls_from_slice(&slice, sf);

        let pairs = [
            (
                tracked.class_like().collect::<Vec<_>>(),
                projected.class_like().collect::<Vec<_>>(),
            ),
            (
                tracked.functions().collect::<Vec<_>>(),
                projected.functions().collect::<Vec<_>>(),
            ),
            (
                tracked.constants().collect::<Vec<_>>(),
                projected.constants().collect::<Vec<_>>(),
            ),
        ];
        for (t, p) in pairs {
            assert_eq!(t, p);
        }
        assert_eq!(tracked.class_like_len(), 5, "I, T, E, C, D expected");
    }

    #[test]
    fn file_declarations_compare_by_lookup_keys_only() {
        let mut db = crate::db::MirDbStorage::default();
        let file_a = db.upsert_source_file_with_durability(
            Arc::from("a.php"),
            Arc::from("<?php\n"),
            salsa::Durability::LOW,
        );
        let file_b = db.upsert_source_file_with_durability(
            Arc::from("b.php"),
            Arc::from("<?php\n"),
            salsa::Durability::LOW,
        );

        let decls_a = FileDeclarations {
            file: file_a,
            rows: Arc::from(
                vec![
                    FileDeclRow {
                        name: Name::new("App\\Thing"),
                        key: Name::new("App\\Thing").ascii_lowercase(),
                        packed_loc: PackedSymbolLoc::from_symbol_loc(SymbolLoc::Class {
                            file: file_a,
                            idx: 0,
                        }),
                    },
                    FileDeclRow {
                        name: Name::new("App\\run"),
                        key: Name::new("App\\run").ascii_lowercase(),
                        packed_loc: PackedSymbolLoc::from_symbol_loc(SymbolLoc::Function {
                            file: file_a,
                            idx: 0,
                        }),
                    },
                    FileDeclRow {
                        name: Name::new("THING"),
                        key: Name::new("THING"),
                        packed_loc: PackedSymbolLoc::from_symbol_loc(SymbolLoc::Constant {
                            file: file_a,
                            idx: 0,
                        }),
                    },
                ]
                .into_boxed_slice(),
            ),
            class_like: 0..1,
            functions: 1..2,
            constants: 2..3,
        };
        let decls_b = FileDeclarations {
            file: file_b,
            rows: Arc::from(
                vec![
                    FileDeclRow {
                        name: Name::new("app\\thing"),
                        key: Name::new("app\\thing").ascii_lowercase(),
                        packed_loc: PackedSymbolLoc::from_symbol_loc(SymbolLoc::Trait {
                            file: file_b,
                            idx: 9,
                        }),
                    },
                    FileDeclRow {
                        name: Name::new("APP\\RUN"),
                        key: Name::new("APP\\RUN").ascii_lowercase(),
                        packed_loc: PackedSymbolLoc::from_symbol_loc(SymbolLoc::Function {
                            file: file_b,
                            idx: 4,
                        }),
                    },
                    FileDeclRow {
                        name: Name::new("THING"),
                        key: Name::new("THING"),
                        packed_loc: PackedSymbolLoc::from_symbol_loc(SymbolLoc::Constant {
                            file: file_b,
                            idx: 7,
                        }),
                    },
                ]
                .into_boxed_slice(),
            ),
            class_like: 0..1,
            functions: 1..2,
            constants: 2..3,
        };

        assert_eq!(decls_a, decls_b);
    }
}
