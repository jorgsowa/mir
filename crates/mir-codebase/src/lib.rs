pub mod storage;
pub mod codebase;

pub use codebase::Codebase;
pub use storage::{
    ClassStorage, MethodStorage, FunctionStorage, PropertyStorage, ConstantStorage,
    InterfaceStorage, TraitStorage, EnumStorage, EnumCaseStorage,
    Visibility, FnParam, TemplateParam,
};
