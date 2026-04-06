pub mod codebase;
pub mod storage;

pub use codebase::Codebase;
pub use storage::{
    ClassStorage, ConstantStorage, EnumCaseStorage, EnumStorage, FnParam, FunctionStorage,
    InterfaceStorage, MethodStorage, PropertyStorage, TemplateParam, TraitStorage, Visibility,
};
