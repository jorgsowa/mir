pub mod definitions;
pub mod file_id;

pub use definitions::{
    deduplicate_params_in_slice, wrap_param_type, ClassDef, ConstantDef, DeclaredParam,
    EnumCaseDef, EnumDef, FunctionDef, InterfaceDef, MethodDef, PropertyDef, StubSlice,
    TemplateParam, TraitDef, Visibility,
};
pub use file_id::{FileId, FileIdMap};
pub use mir_types::Location;
