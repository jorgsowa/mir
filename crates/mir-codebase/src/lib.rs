pub mod codebase;
pub mod interner;
pub mod storage;

pub use codebase::{codebase_from_parts, Codebase, CodebaseBuilder};
pub use storage::{
    ClassStorage, ConstantStorage, EnumCaseStorage, EnumStorage, FnParam, FunctionStorage,
    InterfaceStorage, MethodStorage, PropertyStorage, StubSlice, TemplateParam, TraitStorage,
    Visibility,
};
