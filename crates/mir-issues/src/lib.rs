use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Severity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Only shown with `--show-info`
    Info,
    /// Warnings — shown at default level
    Warning,
    /// Errors — always shown; non-zero exit code
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

// ---------------------------------------------------------------------------
// Location
// ---------------------------------------------------------------------------

pub use mir_types::Location;

// ---------------------------------------------------------------------------
// IssueKind
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum IssueKind {
    // --- Undefined ----------------------------------------------------------
    /// Emitted by `mir-analyzer/src/expr/variables.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_scope/`.
    InvalidScope {
        /// `true` when inside a class but in a static method; `false` when outside a class.
        in_class: bool,
    },
    /// Emitted by `mir-analyzer/src/expr/variables.rs`.
    /// Fixtures: `tests/fixtures/by-kind/undefined_variable/`.
    UndefinedVariable { name: String },
    /// Emitted by `mir-analyzer/src/call/function.rs`.
    /// Fixtures: `tests/fixtures/by-kind/undefined_function/`.
    UndefinedFunction { name: String },
    /// Emitted by `mir-analyzer/src/call/static_call.rs`.
    /// Fixtures: `tests/fixtures/by-kind/undefined_method/`.
    UndefinedMethod { class: String, method: String },
    /// Emitted by `mir-analyzer/src/batch.rs`.
    /// Fixtures: `tests/fixtures/by-kind/undefined_class/`.
    UndefinedClass { name: String },
    /// Emitted by `mir-analyzer/src/expr/objects.rs`.
    /// Fixtures: `tests/fixtures/by-kind/undefined_property/`.
    UndefinedProperty { class: String, property: String },
    /// Emitted by `mir-analyzer/src/expr/variables.rs`.
    /// Fixtures: `tests/fixtures/by-kind/undefined_constant/`.
    UndefinedConstant { name: String },
    /// Emitted by `mir-analyzer/src/expr/variables.rs`.
    /// Fixtures: `tests/fixtures/by-kind/possibly_undefined_variable/`.
    PossiblyUndefinedVariable { name: String },
    /// Emitted by `mir-analyzer/src/body_analysis.rs`.
    /// Fixtures: `tests/fixtures/by-kind/undefined_trait/`.
    UndefinedTrait { name: String },
    /// Emitted by `mir-analyzer/src/expr/objects.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_string_class/`.
    InvalidStringClass { actual: String },

    // --- Nullability --------------------------------------------------------
    /// Emitted by `mir-analyzer/src/call/args.rs`.
    /// Fixtures: `tests/fixtures/by-kind/null_argument/`.
    NullArgument { param: String, fn_name: String },
    /// Emitted by `mir-analyzer/src/expr/objects.rs`.
    /// Fixtures: `tests/fixtures/by-kind/null_property_fetch/`.
    NullPropertyFetch { property: String },
    /// Emitted by `mir-analyzer/src/call/method.rs`.
    /// Fixtures: `tests/fixtures/by-kind/null_method_call/`.
    NullMethodCall { method: String },
    /// Emitted by `mir-analyzer/src/expr/arrays.rs`.
    /// Fixtures: `tests/fixtures/by-kind/null_array_access/`.
    NullArrayAccess,
    /// Emitted by `mir-analyzer/src/call/args.rs`.
    /// Fixtures: `tests/fixtures/by-kind/possibly_null_argument/`.
    PossiblyNullArgument { param: String, fn_name: String },
    /// Emitted by `mir-analyzer/src/call/args.rs`.
    /// Fixtures: `tests/fixtures/by-kind/possibly_invalid_argument/`.
    PossiblyInvalidArgument {
        param: String,
        fn_name: String,
        expected: String,
        actual: String,
    },
    /// Emitted by `mir-analyzer/src/expr/objects.rs`.
    /// Fixtures: `tests/fixtures/by-kind/possibly_null_property_fetch/`.
    PossiblyNullPropertyFetch { property: String },
    /// Emitted by `mir-analyzer/src/call/method.rs`.
    /// Fixtures: `tests/fixtures/by-kind/possibly_null_method_call/`.
    PossiblyNullMethodCall { method: String },
    /// Emitted by `mir-analyzer/src/expr/arrays.rs`.
    /// Fixtures: `tests/fixtures/by-kind/possibly_null_array_access/`.
    PossiblyNullArrayAccess,
    /// Not yet emitted. Fixtures: `tests/fixtures/by-kind/nullable_return_statement/` (planned).
    NullableReturnStatement { expected: String, actual: String },

    // --- Type mismatches ----------------------------------------------------
    /// Emitted by `mir-analyzer/src/stmt/flow.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_return_type/`.
    InvalidReturnType { expected: String, actual: String },
    /// Emitted by `mir-analyzer/src/call/args.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_argument/`.
    InvalidArgument {
        param: String,
        fn_name: String,
        expected: String,
        actual: String,
    },
    /// Emitted by `mir-analyzer/src/call/callable.rs`.
    /// Fixtures: `tests/fixtures/by-kind/too_few_arguments/`.
    TooFewArguments {
        fn_name: String,
        expected: usize,
        actual: usize,
    },
    /// Emitted by `mir-analyzer/src/call/function.rs`.
    /// Fixtures: `tests/fixtures/by-kind/too_many_arguments/`.
    TooManyArguments {
        fn_name: String,
        expected: usize,
        actual: usize,
    },
    /// Emitted by `mir-analyzer/src/call/args.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_named_argument/`.
    InvalidNamedArgument { fn_name: String, name: String },
    /// Emitted by `mir-analyzer/src/call/args.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_pass_by_reference/`.
    InvalidPassByReference { fn_name: String, param: String },
    /// Emitted by `mir-analyzer/src/expr/assignment.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_property_assignment/`.
    InvalidPropertyAssignment {
        property: String,
        expected: String,
        actual: String,
    },
    /// Emitted by `mir-analyzer/src/expr/casts.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_cast/`.
    InvalidCast { from: String, to: String },
    /// Not yet emitted. Fixtures: `tests/fixtures/by-kind/invalid_operand/` (planned).
    InvalidOperand {
        op: String,
        left: String,
        right: String,
    },
    /// Not yet emitted. Fixtures: `tests/fixtures/by-kind/mismatching_docblock_return_type/` (planned).
    MismatchingDocblockReturnType { declared: String, inferred: String },
    /// Not yet emitted. Fixtures: `tests/fixtures/by-kind/mismatching_docblock_param_type/` (planned).
    MismatchingDocblockParamType {
        param: String,
        declared: String,
        inferred: String,
    },
    /// Emitted by `mir-analyzer/src/stmt/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/type_check_mismatch/`.
    TypeCheckMismatch {
        var: String,
        expected: String,
        actual: String,
    },

    // --- Array issues -------------------------------------------------------
    /// Not yet emitted. Fixtures: `tests/fixtures/by-kind/invalid_array_offset/` (planned).
    InvalidArrayOffset { expected: String, actual: String },
    /// Not yet emitted. No fixtures yet.
    NonExistentArrayOffset { key: String },
    /// Emitted by `mir-analyzer/src/expr/assignment.rs`.
    /// Fixtures: `tests/fixtures/by-kind/possibly_invalid_array_offset/`.
    PossiblyInvalidArrayOffset { expected: String, actual: String },

    // --- Redundancy ---------------------------------------------------------
    /// Emitted by `mir-analyzer/src/stmt/control_flow.rs`.
    /// Fixtures: `tests/fixtures/by-kind/redundant_condition/`.
    RedundantCondition { ty: String },
    /// Emitted by `mir-analyzer/src/expr/casts.rs`.
    /// Fixtures: `tests/fixtures/by-kind/redundant_cast/`.
    RedundantCast { from: String, to: String },
    /// Not yet emitted. Fixtures: `tests/fixtures/by-kind/unnecessary_var_annotation/` (planned).
    UnnecessaryVarAnnotation { var: String },
    /// Not yet emitted. Fixtures: `tests/fixtures/by-kind/type_does_not_contain_type/` (planned).
    TypeDoesNotContainType { left: String, right: String },

    // --- Dead code ----------------------------------------------------------
    /// Emitted by `mir-analyzer/src/diagnostics.rs`.
    /// Fixtures: `tests/fixtures/by-kind/unused_variable/`.
    UnusedVariable { name: String },
    /// Emitted by `mir-analyzer/src/diagnostics.rs`.
    /// Fixtures: `tests/fixtures/by-kind/unused_param/`.
    UnusedParam { name: String },
    /// Emitted by `mir-analyzer/src/stmt/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/unreachable_code/`.
    UnreachableCode,
    /// Emitted by `mir-analyzer/src/dead_code.rs`.
    /// Fixtures: `tests/fixtures/by-kind/unused_method/`.
    UnusedMethod { class: String, method: String },
    /// Emitted by `mir-analyzer/src/dead_code.rs`.
    /// Fixtures: `tests/fixtures/by-kind/unused_property/`.
    UnusedProperty { class: String, property: String },
    /// Emitted by `mir-analyzer/src/dead_code.rs`.
    /// Fixtures: `tests/fixtures/by-kind/unused_function/`.
    UnusedFunction { name: String },

    // --- Readonly -----------------------------------------------------------
    /// Emitted by `mir-analyzer/src/expr/assignment.rs`.
    /// Fixtures: `tests/fixtures/by-kind/readonly_property_assignment/`.
    ReadonlyPropertyAssignment { class: String, property: String },

    // --- Inheritance --------------------------------------------------------
    /// Emitted by `mir-analyzer/src/class.rs`.
    /// Fixtures: `tests/fixtures/by-kind/unimplemented_abstract_method/`.
    UnimplementedAbstractMethod { class: String, method: String },
    /// Emitted by `mir-analyzer/src/class.rs`.
    /// Fixtures: `tests/fixtures/by-kind/unimplemented_interface_method/`.
    UnimplementedInterfaceMethod {
        class: String,
        interface: String,
        method: String,
    },
    /// Emitted by `mir-analyzer/src/class.rs`.
    /// Fixtures: `tests/fixtures/by-kind/method_signature_mismatch/`.
    MethodSignatureMismatch {
        class: String,
        method: String,
        detail: String,
    },
    /// Emitted by `mir-analyzer/src/class.rs`.
    /// Fixtures: `tests/fixtures/by-kind/overridden_method_access/`.
    OverriddenMethodAccess { class: String, method: String },
    /// Emitted by `mir-analyzer/src/class.rs`.
    /// Fixtures: `tests/fixtures/by-kind/final_class_extended/`.
    FinalClassExtended { parent: String, child: String },
    /// Emitted by `mir-analyzer/src/class.rs`.
    /// Fixtures: `tests/fixtures/by-kind/final_method_overridden/`.
    FinalMethodOverridden {
        class: String,
        method: String,
        parent: String,
    },
    /// Emitted by `mir-analyzer/src/expr/objects.rs`.
    /// Fixtures: `tests/fixtures/by-kind/abstract_instantiation/`.
    AbstractInstantiation { class: String },

    // --- Security (taint) ---------------------------------------------------
    /// Not yet emitted (generic taint sink; specific sinks use `TaintedHtml`, `TaintedSql`, `TaintedShell`).
    /// No fixtures yet.
    TaintedInput { sink: String },
    /// Emitted by `mir-analyzer/src/call/function.rs`.
    /// Fixtures: `tests/fixtures/by-kind/tainted_html/`.
    TaintedHtml,
    /// Emitted by `mir-analyzer/src/call/function.rs`.
    /// Fixtures: `tests/fixtures/by-kind/tainted_sql/`.
    TaintedSql,
    /// Emitted by `mir-analyzer/src/call/function.rs`.
    /// Fixtures: `tests/fixtures/by-kind/tainted_shell/`.
    TaintedShell,

    // --- Generics -----------------------------------------------------------
    /// Emitted by `mir-analyzer/src/call/function.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_template_param/`.
    InvalidTemplateParam {
        name: String,
        expected_bound: String,
        actual: String,
    },
    /// Emitted by `mir-analyzer/src/call/method.rs`.
    /// Fixtures: `tests/fixtures/by-kind/shadowed_template_param/`.
    ShadowedTemplateParam { name: String },

    // --- Other --------------------------------------------------------------
    /// Emitted by `mir-analyzer/src/call/function.rs`.
    /// Fixtures: `tests/fixtures/by-kind/deprecated_call/`.
    DeprecatedCall {
        name: String,
        message: Option<Arc<str>>,
    },
    /// Emitted by `mir-analyzer/src/call/method.rs`.
    /// Fixtures: `tests/fixtures/by-kind/deprecated_method_call/`.
    DeprecatedMethodCall {
        class: String,
        method: String,
        message: Option<Arc<str>>,
    },
    /// Emitted by `mir-analyzer/src/call/method.rs`.
    /// Fixtures: `tests/fixtures/by-kind/deprecated_method/`.
    DeprecatedMethod {
        class: String,
        method: String,
        message: Option<Arc<str>>,
    },
    /// Emitted by `mir-analyzer/src/class.rs`.
    /// Fixtures: `tests/fixtures/by-kind/deprecated_class/`.
    DeprecatedClass {
        name: String,
        message: Option<Arc<str>>,
    },
    /// Emitted by `mir-analyzer/src/call/method.rs`.
    /// Fixtures: `tests/fixtures/by-kind/internal_method/`.
    InternalMethod { class: String, method: String },
    /// Not yet emitted. Fixtures: `tests/fixtures/by-kind/missing_return_type/` (planned).
    MissingReturnType { fn_name: String },
    /// Not yet emitted. Fixtures: `tests/fixtures/by-kind/missing_param_type/` (planned).
    MissingParamType { fn_name: String, param: String },
    /// Emitted by `mir-analyzer/src/stmt/flow.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_throw/`.
    InvalidThrow { ty: String },
    /// Emitted by `mir-analyzer/src/stmt/flow.rs`.
    /// Fixtures: `tests/fixtures/by-kind/missing_throws_docblock/`.
    MissingThrowsDocblock { class: String },
    /// Emitted by `mir-analyzer/src/stmt/expressions.rs`.
    /// Fixtures: `tests/fixtures/by-kind/implicit_to_string_cast/`.
    ImplicitToStringCast { class: String },
    /// Emitted by `mir-analyzer/src/call/args.rs`.
    /// Fixtures: `tests/fixtures/by-kind/implicit_float_to_int_cast/`.
    ImplicitFloatToIntCast { from: String },
    /// Emitted by `mir-analyzer/src/parser/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/parse_error/`.
    ParseError { message: String },
    /// Emitted by `mir-analyzer/src/collector/annotation.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_docblock/`.
    InvalidDocblock { message: String },
    /// Not yet emitted. Fixtures: `tests/fixtures/by-kind/mixed_argument/` (planned).
    MixedArgument { param: String, fn_name: String },
    /// Not yet emitted. Fixtures: `tests/fixtures/by-kind/mixed_assignment/` (planned).
    MixedAssignment { var: String },
    /// Emitted by `mir-analyzer/src/call/method.rs`.
    /// Fixtures: `tests/fixtures/by-kind/mixed_method_call/`.
    MixedMethodCall { method: String },
    /// Not yet emitted. Fixtures: `tests/fixtures/by-kind/mixed_property_fetch/` (planned).
    MixedPropertyFetch { property: String },
    /// Emitted by `mir-analyzer/src/expr/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/mixed_clone/`.
    MixedClone,
    /// Emitted by `mir-analyzer/src/class.rs`.
    /// Fixtures: `tests/fixtures/by-kind/circular_inheritance/`.
    CircularInheritance { class: String },

    // --- Trait constraints --------------------------------------------------
    /// Emitted by `mir-analyzer/src/body_analysis.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_trait_use/`.
    InvalidTraitUse { trait_name: String, reason: String },
}

fn append_deprecation_message(base: String, message: &Option<Arc<str>>) -> String {
    match message.as_deref().filter(|m| !m.is_empty()) {
        Some(msg) => format!("{base}: {msg}"),
        None => base,
    }
}

impl IssueKind {
    /// Default severity for this issue kind.
    pub fn default_severity(&self) -> Severity {
        match self {
            // Errors (always blocking)
            IssueKind::InvalidScope { .. }
            | IssueKind::UndefinedVariable { .. }
            | IssueKind::UndefinedFunction { .. }
            | IssueKind::UndefinedMethod { .. }
            | IssueKind::UndefinedClass { .. }
            | IssueKind::UndefinedConstant { .. }
            | IssueKind::InvalidReturnType { .. }
            | IssueKind::InvalidArgument { .. }
            | IssueKind::TooFewArguments { .. }
            | IssueKind::TooManyArguments { .. }
            | IssueKind::InvalidNamedArgument { .. }
            | IssueKind::InvalidPassByReference { .. }
            | IssueKind::InvalidThrow { .. }
            | IssueKind::UnimplementedAbstractMethod { .. }
            | IssueKind::UnimplementedInterfaceMethod { .. }
            | IssueKind::MethodSignatureMismatch { .. }
            | IssueKind::FinalClassExtended { .. }
            | IssueKind::FinalMethodOverridden { .. }
            | IssueKind::AbstractInstantiation { .. }
            | IssueKind::InvalidTemplateParam { .. }
            | IssueKind::ReadonlyPropertyAssignment { .. }
            | IssueKind::ParseError { .. }
            | IssueKind::TaintedInput { .. }
            | IssueKind::TaintedHtml
            | IssueKind::TaintedSql
            | IssueKind::TaintedShell
            | IssueKind::CircularInheritance { .. }
            | IssueKind::InvalidTraitUse { .. }
            | IssueKind::UndefinedTrait { .. }
            | IssueKind::TypeCheckMismatch { .. } => Severity::Error,

            // Warnings (shown at default error level)
            IssueKind::NullArgument { .. }
            | IssueKind::NullPropertyFetch { .. }
            | IssueKind::NullMethodCall { .. }
            | IssueKind::NullArrayAccess
            | IssueKind::NullableReturnStatement { .. }
            | IssueKind::InvalidPropertyAssignment { .. }
            | IssueKind::InvalidArrayOffset { .. }
            | IssueKind::NonExistentArrayOffset { .. }
            | IssueKind::PossiblyInvalidArrayOffset { .. }
            | IssueKind::UndefinedProperty { .. }
            | IssueKind::InvalidOperand { .. }
            | IssueKind::OverriddenMethodAccess { .. }
            | IssueKind::ImplicitToStringCast { .. }
            | IssueKind::ImplicitFloatToIntCast { .. }
            | IssueKind::UnusedVariable { .. }
            | IssueKind::InvalidStringClass { .. } => Severity::Warning,

            // PossiblyUndefined: shown at default error level (same as Warning)
            IssueKind::PossiblyUndefinedVariable { .. } => Severity::Warning,

            // Possibly-null / possibly-invalid (only shown in strict mode, level ≥ 7)
            IssueKind::PossiblyNullArgument { .. }
            | IssueKind::PossiblyInvalidArgument { .. }
            | IssueKind::PossiblyNullPropertyFetch { .. }
            | IssueKind::PossiblyNullMethodCall { .. }
            | IssueKind::PossiblyNullArrayAccess => Severity::Info,

            // Info
            IssueKind::RedundantCondition { .. }
            | IssueKind::RedundantCast { .. }
            | IssueKind::UnnecessaryVarAnnotation { .. }
            | IssueKind::TypeDoesNotContainType { .. }
            | IssueKind::UnusedParam { .. }
            | IssueKind::UnreachableCode
            | IssueKind::UnusedMethod { .. }
            | IssueKind::UnusedProperty { .. }
            | IssueKind::UnusedFunction { .. }
            | IssueKind::DeprecatedCall { .. }
            | IssueKind::DeprecatedMethodCall { .. }
            | IssueKind::DeprecatedMethod { .. }
            | IssueKind::DeprecatedClass { .. }
            | IssueKind::InternalMethod { .. }
            | IssueKind::MissingReturnType { .. }
            | IssueKind::MissingParamType { .. }
            | IssueKind::MismatchingDocblockReturnType { .. }
            | IssueKind::MismatchingDocblockParamType { .. }
            | IssueKind::InvalidDocblock { .. }
            | IssueKind::InvalidCast { .. }
            | IssueKind::MixedArgument { .. }
            | IssueKind::MixedAssignment { .. }
            | IssueKind::MixedMethodCall { .. }
            | IssueKind::MixedPropertyFetch { .. }
            | IssueKind::MixedClone
            | IssueKind::ShadowedTemplateParam { .. }
            | IssueKind::MissingThrowsDocblock { .. } => Severity::Info,
        }
    }

    /// Stable error code (e.g. `"MIR0005"`).
    ///
    /// Codes are assigned in bands by category and are part of the public API:
    /// once a code ships, it must never be reused for a different issue kind.
    /// New variants take the next free slot in their band; obsolete variants
    /// retire their code (the slot stays burnt). Bands have headroom for growth.
    ///
    /// Bands:
    ///
    /// | Range         | Category                        |
    /// |---------------|---------------------------------|
    /// | 0001 – 0099   | Undefined symbols               |
    /// | 0100 – 0199   | Nullability                     |
    /// | 0200 – 0299   | Type mismatches                 |
    /// | 0300 – 0399   | Array / offset                  |
    /// | 0400 – 0499   | Redundancy                      |
    /// | 0500 – 0599   | Dead code                       |
    /// | 0600 – 0699   | Readonly                        |
    /// | 0700 – 0799   | Inheritance                     |
    /// | 0800 – 0899   | Security (taint)                |
    /// | 0900 – 0999   | Generics                        |
    /// | 1000 – 1099   | Deprecation / internal          |
    /// | 1100 – 1199   | Missing types / docblocks       |
    /// | 1200 – 1299   | Mixed                           |
    /// | 1300 – 1399   | Trait                           |
    /// | 1400 – 1499   | Parse                           |
    /// | 1500 – 1599   | Other                           |
    pub fn code(&self) -> &'static str {
        match self {
            // Undefined (0001-0099)
            IssueKind::InvalidScope { .. } => "MIR0001",
            IssueKind::UndefinedVariable { .. } => "MIR0002",
            IssueKind::UndefinedFunction { .. } => "MIR0003",
            IssueKind::UndefinedMethod { .. } => "MIR0004",
            IssueKind::UndefinedClass { .. } => "MIR0005",
            IssueKind::UndefinedProperty { .. } => "MIR0006",
            IssueKind::UndefinedConstant { .. } => "MIR0007",
            IssueKind::PossiblyUndefinedVariable { .. } => "MIR0008",
            IssueKind::UndefinedTrait { .. } => "MIR0009",

            // Nullability (0100-0199)
            IssueKind::NullArgument { .. } => "MIR0100",
            IssueKind::NullPropertyFetch { .. } => "MIR0101",
            IssueKind::NullMethodCall { .. } => "MIR0102",
            IssueKind::NullArrayAccess => "MIR0103",
            IssueKind::PossiblyNullArgument { .. } => "MIR0104",
            IssueKind::PossiblyInvalidArgument { .. } => "MIR0105",
            IssueKind::PossiblyNullPropertyFetch { .. } => "MIR0106",
            IssueKind::PossiblyNullMethodCall { .. } => "MIR0107",
            IssueKind::PossiblyNullArrayAccess => "MIR0108",
            IssueKind::NullableReturnStatement { .. } => "MIR0109",

            // Type mismatches (0200-0299)
            IssueKind::InvalidReturnType { .. } => "MIR0200",
            IssueKind::InvalidArgument { .. } => "MIR0201",
            IssueKind::TooFewArguments { .. } => "MIR0202",
            IssueKind::TooManyArguments { .. } => "MIR0203",
            IssueKind::InvalidNamedArgument { .. } => "MIR0204",
            IssueKind::InvalidPassByReference { .. } => "MIR0205",
            IssueKind::InvalidPropertyAssignment { .. } => "MIR0206",
            IssueKind::InvalidCast { .. } => "MIR0207",
            IssueKind::InvalidOperand { .. } => "MIR0208",
            IssueKind::MismatchingDocblockReturnType { .. } => "MIR0209",
            IssueKind::MismatchingDocblockParamType { .. } => "MIR0210",
            IssueKind::InvalidStringClass { .. } => "MIR0211",
            IssueKind::TypeCheckMismatch { .. } => "MIR0212",

            // Array / offset (0300-0399)
            IssueKind::InvalidArrayOffset { .. } => "MIR0300",
            IssueKind::NonExistentArrayOffset { .. } => "MIR0301",
            IssueKind::PossiblyInvalidArrayOffset { .. } => "MIR0302",

            // Redundancy (0400-0499)
            IssueKind::RedundantCondition { .. } => "MIR0400",
            IssueKind::RedundantCast { .. } => "MIR0401",
            IssueKind::UnnecessaryVarAnnotation { .. } => "MIR0402",
            IssueKind::TypeDoesNotContainType { .. } => "MIR0403",

            // Dead code (0500-0599)
            IssueKind::UnusedVariable { .. } => "MIR0500",
            IssueKind::UnusedParam { .. } => "MIR0501",
            IssueKind::UnreachableCode => "MIR0502",
            IssueKind::UnusedMethod { .. } => "MIR0503",
            IssueKind::UnusedProperty { .. } => "MIR0504",
            IssueKind::UnusedFunction { .. } => "MIR0505",

            // Readonly (0600-0699)
            IssueKind::ReadonlyPropertyAssignment { .. } => "MIR0600",

            // Inheritance (0700-0799)
            IssueKind::UnimplementedAbstractMethod { .. } => "MIR0700",
            IssueKind::UnimplementedInterfaceMethod { .. } => "MIR0701",
            IssueKind::MethodSignatureMismatch { .. } => "MIR0702",
            IssueKind::OverriddenMethodAccess { .. } => "MIR0703",
            IssueKind::FinalClassExtended { .. } => "MIR0704",
            IssueKind::FinalMethodOverridden { .. } => "MIR0705",
            IssueKind::AbstractInstantiation { .. } => "MIR0706",
            IssueKind::CircularInheritance { .. } => "MIR0707",

            // Security / taint (0800-0899)
            IssueKind::TaintedInput { .. } => "MIR0800",
            IssueKind::TaintedHtml => "MIR0801",
            IssueKind::TaintedSql => "MIR0802",
            IssueKind::TaintedShell => "MIR0803",

            // Generics (0900-0999)
            IssueKind::InvalidTemplateParam { .. } => "MIR0900",
            IssueKind::ShadowedTemplateParam { .. } => "MIR0901",

            // Deprecation / internal (1000-1099)
            IssueKind::DeprecatedCall { .. } => "MIR1000",
            IssueKind::DeprecatedMethodCall { .. } => "MIR1001",
            IssueKind::DeprecatedMethod { .. } => "MIR1002",
            IssueKind::DeprecatedClass { .. } => "MIR1003",
            IssueKind::InternalMethod { .. } => "MIR1004",

            // Missing types / docblocks (1100-1199)
            IssueKind::MissingReturnType { .. } => "MIR1100",
            IssueKind::MissingParamType { .. } => "MIR1101",
            IssueKind::MissingThrowsDocblock { .. } => "MIR1102",
            IssueKind::InvalidDocblock { .. } => "MIR1103",

            // Mixed (1200-1299)
            IssueKind::MixedArgument { .. } => "MIR1200",
            IssueKind::MixedAssignment { .. } => "MIR1201",
            IssueKind::MixedMethodCall { .. } => "MIR1202",
            IssueKind::MixedPropertyFetch { .. } => "MIR1203",
            IssueKind::MixedClone => "MIR1204",

            // Trait (1300-1399)
            IssueKind::InvalidTraitUse { .. } => "MIR1300",

            // Parse (1400-1499)
            IssueKind::ParseError { .. } => "MIR1400",

            // Other (1500-1599)
            IssueKind::InvalidThrow { .. } => "MIR1500",
            IssueKind::ImplicitToStringCast { .. } => "MIR1501",
            IssueKind::ImplicitFloatToIntCast { .. } => "MIR1502",
        }
    }

    /// Identifier name used in config and `@psalm-suppress` / `@suppress` annotations.
    pub fn name(&self) -> &'static str {
        match self {
            IssueKind::InvalidScope { .. } => "InvalidScope",
            IssueKind::UndefinedVariable { .. } => "UndefinedVariable",
            IssueKind::UndefinedFunction { .. } => "UndefinedFunction",
            IssueKind::UndefinedMethod { .. } => "UndefinedMethod",
            IssueKind::UndefinedClass { .. } => "UndefinedClass",
            IssueKind::UndefinedProperty { .. } => "UndefinedProperty",
            IssueKind::UndefinedConstant { .. } => "UndefinedConstant",
            IssueKind::PossiblyUndefinedVariable { .. } => "PossiblyUndefinedVariable",
            IssueKind::UndefinedTrait { .. } => "UndefinedTrait",
            IssueKind::InvalidStringClass { .. } => "InvalidStringClass",
            IssueKind::NullArgument { .. } => "NullArgument",
            IssueKind::NullPropertyFetch { .. } => "NullPropertyFetch",
            IssueKind::NullMethodCall { .. } => "NullMethodCall",
            IssueKind::NullArrayAccess => "NullArrayAccess",
            IssueKind::PossiblyNullArgument { .. } => "PossiblyNullArgument",
            IssueKind::PossiblyInvalidArgument { .. } => "PossiblyInvalidArgument",
            IssueKind::PossiblyNullPropertyFetch { .. } => "PossiblyNullPropertyFetch",
            IssueKind::PossiblyNullMethodCall { .. } => "PossiblyNullMethodCall",
            IssueKind::PossiblyNullArrayAccess => "PossiblyNullArrayAccess",
            IssueKind::NullableReturnStatement { .. } => "NullableReturnStatement",
            IssueKind::InvalidReturnType { .. } => "InvalidReturnType",
            IssueKind::InvalidArgument { .. } => "InvalidArgument",
            IssueKind::TooFewArguments { .. } => "TooFewArguments",
            IssueKind::TooManyArguments { .. } => "TooManyArguments",
            IssueKind::InvalidNamedArgument { .. } => "InvalidNamedArgument",
            IssueKind::InvalidPassByReference { .. } => "InvalidPassByReference",
            IssueKind::InvalidPropertyAssignment { .. } => "InvalidPropertyAssignment",
            IssueKind::InvalidCast { .. } => "InvalidCast",
            IssueKind::InvalidOperand { .. } => "InvalidOperand",
            IssueKind::MismatchingDocblockReturnType { .. } => "MismatchingDocblockReturnType",
            IssueKind::MismatchingDocblockParamType { .. } => "MismatchingDocblockParamType",
            IssueKind::TypeCheckMismatch { .. } => "TypeCheckMismatch",
            IssueKind::InvalidArrayOffset { .. } => "InvalidArrayOffset",
            IssueKind::NonExistentArrayOffset { .. } => "NonExistentArrayOffset",
            IssueKind::PossiblyInvalidArrayOffset { .. } => "PossiblyInvalidArrayOffset",
            IssueKind::RedundantCondition { .. } => "RedundantCondition",
            IssueKind::RedundantCast { .. } => "RedundantCast",
            IssueKind::UnnecessaryVarAnnotation { .. } => "UnnecessaryVarAnnotation",
            IssueKind::TypeDoesNotContainType { .. } => "TypeDoesNotContainType",
            IssueKind::UnusedVariable { .. } => "UnusedVariable",
            IssueKind::UnusedParam { .. } => "UnusedParam",
            IssueKind::UnreachableCode => "UnreachableCode",
            IssueKind::UnusedMethod { .. } => "UnusedMethod",
            IssueKind::UnusedProperty { .. } => "UnusedProperty",
            IssueKind::UnusedFunction { .. } => "UnusedFunction",
            IssueKind::UnimplementedAbstractMethod { .. } => "UnimplementedAbstractMethod",
            IssueKind::UnimplementedInterfaceMethod { .. } => "UnimplementedInterfaceMethod",
            IssueKind::MethodSignatureMismatch { .. } => "MethodSignatureMismatch",
            IssueKind::OverriddenMethodAccess { .. } => "OverriddenMethodAccess",
            IssueKind::FinalClassExtended { .. } => "FinalClassExtended",
            IssueKind::FinalMethodOverridden { .. } => "FinalMethodOverridden",
            IssueKind::AbstractInstantiation { .. } => "AbstractInstantiation",
            IssueKind::ReadonlyPropertyAssignment { .. } => "ReadonlyPropertyAssignment",
            IssueKind::InvalidTemplateParam { .. } => "InvalidTemplateParam",
            IssueKind::ShadowedTemplateParam { .. } => "ShadowedTemplateParam",
            IssueKind::TaintedInput { .. } => "TaintedInput",
            IssueKind::TaintedHtml => "TaintedHtml",
            IssueKind::TaintedSql => "TaintedSql",
            IssueKind::TaintedShell => "TaintedShell",
            IssueKind::DeprecatedCall { .. } => "DeprecatedCall",
            IssueKind::DeprecatedMethodCall { .. } => "DeprecatedMethodCall",
            IssueKind::DeprecatedMethod { .. } => "DeprecatedMethod",
            IssueKind::DeprecatedClass { .. } => "DeprecatedClass",
            IssueKind::InternalMethod { .. } => "InternalMethod",
            IssueKind::MissingReturnType { .. } => "MissingReturnType",
            IssueKind::MissingParamType { .. } => "MissingParamType",
            IssueKind::InvalidThrow { .. } => "InvalidThrow",
            IssueKind::MissingThrowsDocblock { .. } => "MissingThrowsDocblock",
            IssueKind::ImplicitToStringCast { .. } => "ImplicitToStringCast",
            IssueKind::ImplicitFloatToIntCast { .. } => "ImplicitFloatToIntCast",
            IssueKind::ParseError { .. } => "ParseError",
            IssueKind::InvalidDocblock { .. } => "InvalidDocblock",
            IssueKind::MixedArgument { .. } => "MixedArgument",
            IssueKind::MixedAssignment { .. } => "MixedAssignment",
            IssueKind::MixedMethodCall { .. } => "MixedMethodCall",
            IssueKind::MixedPropertyFetch { .. } => "MixedPropertyFetch",
            IssueKind::MixedClone => "MixedClone",
            IssueKind::CircularInheritance { .. } => "CircularInheritance",
            IssueKind::InvalidTraitUse { .. } => "InvalidTraitUse",
        }
    }

    /// Human-readable message for this issue.
    pub fn message(&self) -> String {
        match self {
            IssueKind::InvalidScope { in_class } => {
                if *in_class {
                    "$this cannot be used in a static method".to_string()
                } else {
                    "$this cannot be used outside of a class".to_string()
                }
            }
            IssueKind::UndefinedVariable { name } => format!("Variable ${name} is not defined"),
            IssueKind::UndefinedFunction { name } => format!("Function {name}() is not defined"),
            IssueKind::UndefinedMethod { class, method } => {
                format!("Method {class}::{method}() does not exist")
            }
            IssueKind::UndefinedClass { name } => format!("Class {name} does not exist"),
            IssueKind::UndefinedProperty { class, property } => {
                format!("Property {class}::${property} does not exist")
            }
            IssueKind::UndefinedConstant { name } => format!("Constant {name} is not defined"),
            IssueKind::PossiblyUndefinedVariable { name } => {
                format!("Variable ${name} might not be defined")
            }
            IssueKind::UndefinedTrait { name } => format!("Trait {name} does not exist"),
            IssueKind::InvalidStringClass { actual } => {
                format!("Dynamic class instantiation requires string or class-string type, got '{actual}'")
            }

            IssueKind::NullArgument { param, fn_name } => {
                format!("Argument ${param} of {fn_name}() cannot be null")
            }
            IssueKind::NullPropertyFetch { property } => {
                format!("Cannot access property ${property} on null")
            }
            IssueKind::NullMethodCall { method } => {
                format!("Cannot call method {method}() on null")
            }
            IssueKind::NullArrayAccess => "Cannot access array on null".to_string(),
            IssueKind::PossiblyNullArgument { param, fn_name } => {
                format!("Argument ${param} of {fn_name}() might be null")
            }
            IssueKind::PossiblyInvalidArgument {
                param,
                fn_name,
                expected,
                actual,
            } => {
                format!("Argument ${param} of {fn_name}() expects '{expected}', possibly different type '{actual}' provided")
            }
            IssueKind::PossiblyNullPropertyFetch { property } => {
                format!("Cannot access property ${property} on possibly null value")
            }
            IssueKind::PossiblyNullMethodCall { method } => {
                format!("Cannot call method {method}() on possibly null value")
            }
            IssueKind::PossiblyNullArrayAccess => {
                "Cannot access array on possibly null value".to_string()
            }
            IssueKind::NullableReturnStatement { expected, actual } => {
                format!("Return type '{actual}' is not compatible with declared '{expected}'")
            }

            IssueKind::InvalidReturnType { expected, actual } => {
                format!("Return type '{actual}' is not compatible with declared '{expected}'")
            }
            IssueKind::InvalidArgument {
                param,
                fn_name,
                expected,
                actual,
            } => {
                format!("Argument ${param} of {fn_name}() expects '{expected}', got '{actual}'")
            }
            IssueKind::TooFewArguments {
                fn_name,
                expected,
                actual,
            } => {
                format!(
                    "Too few arguments for {}(): expected {}, got {}",
                    fn_name, expected, actual
                )
            }
            IssueKind::TooManyArguments {
                fn_name,
                expected,
                actual,
            } => {
                format!(
                    "Too many arguments for {}(): expected {}, got {}",
                    fn_name, expected, actual
                )
            }
            IssueKind::InvalidNamedArgument { fn_name, name } => {
                format!("{}() has no parameter named ${}", fn_name, name)
            }
            IssueKind::InvalidPassByReference { fn_name, param } => {
                format!(
                    "Argument ${} of {}() must be passed by reference",
                    param, fn_name
                )
            }
            IssueKind::InvalidPropertyAssignment {
                property,
                expected,
                actual,
            } => {
                format!("Property ${property} expects '{expected}', cannot assign '{actual}'")
            }
            IssueKind::InvalidCast { from, to } => {
                format!("Cannot cast '{from}' to '{to}'")
            }
            IssueKind::InvalidOperand { op, left, right } => {
                format!("Operator '{op}' not supported between '{left}' and '{right}'")
            }
            IssueKind::MismatchingDocblockReturnType { declared, inferred } => {
                format!("Docblock return type '{declared}' does not match inferred '{inferred}'")
            }
            IssueKind::MismatchingDocblockParamType {
                param,
                declared,
                inferred,
            } => {
                format!(
                    "Docblock type '{declared}' for ${param} does not match inferred '{inferred}'"
                )
            }
            IssueKind::TypeCheckMismatch {
                var,
                expected,
                actual,
            } => {
                format!("Type of ${var} is expected to be {expected}, got {actual}")
            }

            IssueKind::InvalidArrayOffset { expected, actual } => {
                format!("Array offset expects '{expected}', got '{actual}'")
            }
            IssueKind::NonExistentArrayOffset { key } => {
                format!("Array offset '{key}' does not exist")
            }
            IssueKind::PossiblyInvalidArrayOffset { expected, actual } => {
                format!("Array offset might be invalid: expects '{expected}', got '{actual}'")
            }

            IssueKind::RedundantCondition { ty } => {
                format!("Condition is always true/false for type '{ty}'")
            }
            IssueKind::RedundantCast { from, to } => {
                format!("Casting '{from}' to '{to}' is redundant")
            }
            IssueKind::UnnecessaryVarAnnotation { var } => {
                format!("@var annotation for ${var} is unnecessary")
            }
            IssueKind::TypeDoesNotContainType { left, right } => {
                format!("Type '{left}' can never contain type '{right}'")
            }

            IssueKind::UnusedVariable { name } => format!("Variable ${name} is never read"),
            IssueKind::UnusedParam { name } => format!("Parameter ${name} is never used"),
            IssueKind::UnreachableCode => "Unreachable code detected".to_string(),
            IssueKind::UnusedMethod { class, method } => {
                format!("Private method {class}::{method}() is never called")
            }
            IssueKind::UnusedProperty { class, property } => {
                format!("Private property {class}::${property} is never read")
            }
            IssueKind::UnusedFunction { name } => {
                format!("Function {name}() is never called")
            }

            IssueKind::UnimplementedAbstractMethod { class, method } => {
                format!("Class {class} must implement abstract method {method}()")
            }
            IssueKind::UnimplementedInterfaceMethod {
                class,
                interface,
                method,
            } => {
                format!("Class {class} must implement {interface}::{method}() from interface")
            }
            IssueKind::MethodSignatureMismatch {
                class,
                method,
                detail,
            } => {
                format!("Method {class}::{method}() signature mismatch: {detail}")
            }
            IssueKind::OverriddenMethodAccess { class, method } => {
                format!("Method {class}::{method}() overrides with less visibility")
            }
            IssueKind::ReadonlyPropertyAssignment { class, property } => {
                format!(
                    "Cannot assign to readonly property {class}::${property} outside of constructor"
                )
            }
            IssueKind::FinalClassExtended { parent, child } => {
                format!("Class {child} cannot extend final class {parent}")
            }
            IssueKind::InvalidTemplateParam {
                name,
                expected_bound,
                actual,
            } => {
                format!(
                    "Template type '{name}' inferred as '{actual}' does not satisfy bound '{expected_bound}'"
                )
            }
            IssueKind::ShadowedTemplateParam { name } => {
                format!(
                    "Method template parameter '{name}' shadows class-level template parameter with the same name"
                )
            }
            IssueKind::FinalMethodOverridden {
                class,
                method,
                parent,
            } => {
                format!("Method {class}::{method}() cannot override final method from {parent}")
            }
            IssueKind::AbstractInstantiation { class } => {
                format!("Cannot instantiate abstract class {class}")
            }

            IssueKind::TaintedInput { sink } => format!("Tainted input reaching sink '{sink}'"),
            IssueKind::TaintedHtml => "Tainted HTML output — possible XSS".to_string(),
            IssueKind::TaintedSql => "Tainted SQL query — possible SQL injection".to_string(),
            IssueKind::TaintedShell => {
                "Tainted shell command — possible command injection".to_string()
            }

            IssueKind::DeprecatedCall { name, message } => {
                let base = format!("Call to deprecated function {name}");
                append_deprecation_message(base, message)
            }
            IssueKind::DeprecatedMethodCall {
                class,
                method,
                message,
            } => {
                let base = format!("Call to deprecated method {class}::{method}");
                append_deprecation_message(base, message)
            }
            IssueKind::DeprecatedMethod {
                class,
                method,
                message,
            } => {
                let base = format!("Method {class}::{method}() is deprecated");
                append_deprecation_message(base, message)
            }
            IssueKind::DeprecatedClass { name, message } => {
                let base = format!("Class {name} is deprecated");
                append_deprecation_message(base, message)
            }
            IssueKind::InternalMethod { class, method } => {
                format!("Method {class}::{method}() is marked @internal")
            }
            IssueKind::MissingReturnType { fn_name } => {
                format!("Function {fn_name}() has no return type annotation")
            }
            IssueKind::MissingParamType { fn_name, param } => {
                format!("Parameter ${param} of {fn_name}() has no type annotation")
            }
            IssueKind::InvalidThrow { ty } => {
                format!("Thrown type '{ty}' does not extend Throwable")
            }
            IssueKind::MissingThrowsDocblock { class } => {
                format!("Exception {class} is thrown but not declared in @throws")
            }
            IssueKind::ImplicitToStringCast { class } => {
                format!("Class {class} does not implement __toString()")
            }
            IssueKind::ImplicitFloatToIntCast { from } => {
                format!("Implicit cast from {from} to int truncates the fractional part")
            }
            IssueKind::ParseError { message } => format!("Parse error: {message}"),
            IssueKind::InvalidDocblock { message } => format!("Invalid docblock: {message}"),
            IssueKind::MixedArgument { param, fn_name } => {
                format!("Argument ${param} of {fn_name}() is mixed")
            }
            IssueKind::MixedAssignment { var } => {
                format!("Variable ${var} is assigned a mixed type")
            }
            IssueKind::MixedMethodCall { method } => {
                format!("Method {method}() called on mixed type")
            }
            IssueKind::MixedPropertyFetch { property } => {
                format!("Property ${property} fetched on mixed type")
            }
            IssueKind::MixedClone => "cannot clone mixed".to_string(),
            IssueKind::CircularInheritance { class } => {
                format!("Class {class} has a circular inheritance chain")
            }
            IssueKind::InvalidTraitUse { trait_name, reason } => {
                format!("Trait {trait_name} used incorrectly: {reason}")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Issue
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issue {
    pub kind: IssueKind,
    pub severity: Severity,
    pub location: Location,
    pub snippet: Option<String>,
    pub suppressed: bool,
}

impl Issue {
    pub fn new(kind: IssueKind, location: Location) -> Self {
        let severity = kind.default_severity();
        Self {
            severity,
            kind,
            location,
            snippet: None,
            suppressed: false,
        }
    }

    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }

    pub fn suppress(mut self) -> Self {
        self.suppressed = true;
        self
    }
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sev = match self.severity {
            Severity::Error => "error".red().to_string(),
            Severity::Warning => "warning".yellow().to_string(),
            Severity::Info => "info".blue().to_string(),
        };
        write!(
            f,
            "{} {}[{}] {}: {}",
            self.location.bright_black(),
            sev,
            self.kind.code().bright_black(),
            self.kind.name().bold(),
            self.kind.message()
        )
    }
}

// ---------------------------------------------------------------------------
// IssueBuffer — collects issues for a single file pass
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct IssueBuffer {
    issues: Vec<Issue>,
    seen: HashSet<(&'static str, Arc<str>, u32, u16)>,
    /// Issue names suppressed at the file level (from `@psalm-suppress` / `@suppress` on the file docblock)
    file_suppressions: Vec<String>,
}

impl IssueBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, issue: Issue) {
        let key = (
            issue.kind.name(),
            issue.location.file.clone(),
            issue.location.line,
            issue.location.col_start,
        );
        if self.seen.insert(key) {
            self.issues.push(issue);
        }
    }

    pub fn add_suppression(&mut self, name: impl Into<String>) {
        self.file_suppressions.push(name.into());
    }

    /// Consume the buffer and return unsuppressed issues.
    pub fn into_issues(self) -> Vec<Issue> {
        self.issues
            .into_iter()
            .filter(|i| !i.suppressed)
            .filter(|i| !self.file_suppressions.contains(&i.kind.name().to_string()))
            .collect()
    }

    /// Mark all issues added since index `from` as suppressed if their issue
    /// name appears in `suppressions`. Used for `@psalm-suppress` / `@suppress` on statements.
    pub fn suppress_range(&mut self, from: usize, suppressions: &[String]) {
        if suppressions.is_empty() {
            return;
        }
        for issue in self.issues[from..].iter_mut() {
            if suppressions.iter().any(|s| s == issue.kind.name()) {
                issue.suppressed = true;
            }
        }
    }

    /// Current number of buffered issues. Use before analyzing a statement to
    /// get the `from` index for `suppress_range`.
    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }

    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn len(&self) -> usize {
        self.issues.len()
    }

    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| !i.suppressed && i.severity == Severity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| !i.suppressed && i.severity == Severity::Warning)
            .count()
    }
}

#[cfg(test)]
mod code_tests {
    use super::*;
    use std::collections::HashSet;

    /// Returns one instance of every `IssueKind` variant.
    ///
    /// Updating `IssueKind` without updating this list will compile (it's a
    /// regular `Vec`), but `codes_cover_every_variant` will catch the omission
    /// — the test below asserts the count matches the exhaustive `code()` arm.
    fn one_of_each() -> Vec<IssueKind> {
        let s = || String::new();
        vec![
            IssueKind::InvalidScope { in_class: false },
            IssueKind::UndefinedVariable { name: s() },
            IssueKind::UndefinedFunction { name: s() },
            IssueKind::UndefinedMethod {
                class: s(),
                method: s(),
            },
            IssueKind::UndefinedClass { name: s() },
            IssueKind::UndefinedProperty {
                class: s(),
                property: s(),
            },
            IssueKind::UndefinedConstant { name: s() },
            IssueKind::PossiblyUndefinedVariable { name: s() },
            IssueKind::NullArgument {
                param: s(),
                fn_name: s(),
            },
            IssueKind::NullPropertyFetch { property: s() },
            IssueKind::NullMethodCall { method: s() },
            IssueKind::NullArrayAccess,
            IssueKind::PossiblyNullArgument {
                param: s(),
                fn_name: s(),
            },
            IssueKind::PossiblyInvalidArgument {
                param: s(),
                fn_name: s(),
                expected: s(),
                actual: s(),
            },
            IssueKind::PossiblyNullPropertyFetch { property: s() },
            IssueKind::PossiblyNullMethodCall { method: s() },
            IssueKind::PossiblyNullArrayAccess,
            IssueKind::NullableReturnStatement {
                expected: s(),
                actual: s(),
            },
            IssueKind::InvalidReturnType {
                expected: s(),
                actual: s(),
            },
            IssueKind::InvalidArgument {
                param: s(),
                fn_name: s(),
                expected: s(),
                actual: s(),
            },
            IssueKind::TooFewArguments {
                fn_name: s(),
                expected: 0,
                actual: 0,
            },
            IssueKind::TooManyArguments {
                fn_name: s(),
                expected: 0,
                actual: 0,
            },
            IssueKind::InvalidNamedArgument {
                fn_name: s(),
                name: s(),
            },
            IssueKind::InvalidPassByReference {
                fn_name: s(),
                param: s(),
            },
            IssueKind::InvalidPropertyAssignment {
                property: s(),
                expected: s(),
                actual: s(),
            },
            IssueKind::InvalidCast { from: s(), to: s() },
            IssueKind::InvalidOperand {
                op: s(),
                left: s(),
                right: s(),
            },
            IssueKind::MismatchingDocblockReturnType {
                declared: s(),
                inferred: s(),
            },
            IssueKind::MismatchingDocblockParamType {
                param: s(),
                declared: s(),
                inferred: s(),
            },
            IssueKind::TypeCheckMismatch {
                var: s(),
                expected: s(),
                actual: s(),
            },
            IssueKind::InvalidArrayOffset {
                expected: s(),
                actual: s(),
            },
            IssueKind::NonExistentArrayOffset { key: s() },
            IssueKind::PossiblyInvalidArrayOffset {
                expected: s(),
                actual: s(),
            },
            IssueKind::RedundantCondition { ty: s() },
            IssueKind::RedundantCast { from: s(), to: s() },
            IssueKind::UnnecessaryVarAnnotation { var: s() },
            IssueKind::TypeDoesNotContainType {
                left: s(),
                right: s(),
            },
            IssueKind::UnusedVariable { name: s() },
            IssueKind::UnusedParam { name: s() },
            IssueKind::UnreachableCode,
            IssueKind::UnusedMethod {
                class: s(),
                method: s(),
            },
            IssueKind::UnusedProperty {
                class: s(),
                property: s(),
            },
            IssueKind::UnusedFunction { name: s() },
            IssueKind::ReadonlyPropertyAssignment {
                class: s(),
                property: s(),
            },
            IssueKind::UnimplementedAbstractMethod {
                class: s(),
                method: s(),
            },
            IssueKind::UnimplementedInterfaceMethod {
                class: s(),
                interface: s(),
                method: s(),
            },
            IssueKind::MethodSignatureMismatch {
                class: s(),
                method: s(),
                detail: s(),
            },
            IssueKind::OverriddenMethodAccess {
                class: s(),
                method: s(),
            },
            IssueKind::FinalClassExtended {
                parent: s(),
                child: s(),
            },
            IssueKind::FinalMethodOverridden {
                class: s(),
                method: s(),
                parent: s(),
            },
            IssueKind::AbstractInstantiation { class: s() },
            IssueKind::CircularInheritance { class: s() },
            IssueKind::TaintedInput { sink: s() },
            IssueKind::TaintedHtml,
            IssueKind::TaintedSql,
            IssueKind::TaintedShell,
            IssueKind::InvalidTemplateParam {
                name: s(),
                expected_bound: s(),
                actual: s(),
            },
            IssueKind::ShadowedTemplateParam { name: s() },
            IssueKind::DeprecatedCall {
                name: s(),
                message: None,
            },
            IssueKind::DeprecatedMethodCall {
                class: s(),
                method: s(),
                message: None,
            },
            IssueKind::DeprecatedMethod {
                class: s(),
                method: s(),
                message: None,
            },
            IssueKind::DeprecatedClass {
                name: s(),
                message: None,
            },
            IssueKind::InternalMethod {
                class: s(),
                method: s(),
            },
            IssueKind::MissingReturnType { fn_name: s() },
            IssueKind::MissingParamType {
                fn_name: s(),
                param: s(),
            },
            IssueKind::MissingThrowsDocblock { class: s() },
            IssueKind::InvalidDocblock { message: s() },
            IssueKind::MixedArgument {
                param: s(),
                fn_name: s(),
            },
            IssueKind::MixedAssignment { var: s() },
            IssueKind::MixedMethodCall { method: s() },
            IssueKind::MixedPropertyFetch { property: s() },
            IssueKind::MixedClone,
            IssueKind::InvalidTraitUse {
                trait_name: s(),
                reason: s(),
            },
            IssueKind::ParseError { message: s() },
            IssueKind::InvalidThrow { ty: s() },
            IssueKind::ImplicitToStringCast { class: s() },
            IssueKind::ImplicitFloatToIntCast { from: s() },
        ]
    }

    #[test]
    fn codes_have_expected_shape() {
        for kind in one_of_each() {
            let code = kind.code();
            assert!(
                code.len() == 7
                    && code.starts_with("MIR")
                    && code[3..].chars().all(|c| c.is_ascii_digit()),
                "code {code:?} for {} does not match MIR####",
                kind.name(),
            );
        }
    }

    #[test]
    fn codes_are_unique() {
        let kinds = one_of_each();
        let mut seen: HashSet<&'static str> = HashSet::new();
        for kind in &kinds {
            assert!(
                seen.insert(kind.code()),
                "duplicate code {} (variant {})",
                kind.code(),
                kind.name(),
            );
        }
    }

    #[test]
    fn display_includes_code() {
        let issue = Issue::new(
            IssueKind::UndefinedClass {
                name: "Foo".to_string(),
            },
            Location {
                file: Arc::from("src/x.php"),
                line: 1,
                line_end: 1,
                col_start: 0,
                col_end: 3,
            },
        );
        // Strip ANSI escape sequences so the assertion isn't dependent on
        // owo-colors' tty detection.
        let raw = format!("{issue}");
        let stripped: String = {
            let mut out = String::new();
            let mut chars = raw.chars();
            while let Some(c) = chars.next() {
                if c == '\u{1b}' {
                    for c2 in chars.by_ref() {
                        if c2 == 'm' {
                            break;
                        }
                    }
                } else {
                    out.push(c);
                }
            }
            out
        };
        assert!(
            stripped.contains("error[MIR0005] UndefinedClass:"),
            "Display output missing code/name segment: {stripped:?}",
        );
    }

    /// Guards against forgetting to add a new variant to `one_of_each()`.
    /// If you add a variant, add it to `one_of_each()` *and* bump this count.
    #[test]
    fn one_of_each_has_every_variant() {
        // 77 = current variant count. If this assertion fires after you added
        // a new variant, also add it to `one_of_each()` so the uniqueness
        // and shape tests cover it.
        assert_eq!(one_of_each().len(), 77);
    }
}
