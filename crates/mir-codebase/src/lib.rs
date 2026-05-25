pub mod file_id;
pub mod storage;

pub use file_id::{FileId, FileIdMap};
pub use storage::{
    deduplicate_params_in_slice, wrap_param_type, ClassDef, ConstantDef, EnumCaseDef, EnumDef,
    FnParam, FunctionDef, InterfaceDef, MethodDef, PropertyDef, StubSlice, TemplateParam, TraitDef,
    Visibility,
};
