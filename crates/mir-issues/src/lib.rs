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
    /// Emitted by `mir-analyzer/src/call/static_call.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_scope/self_non_static_invocation.phpt`.
    NonStaticSelfCall { class: String, method: String },
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
    /// Emitted by `mir-analyzer/src/batch/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/undefined_class/`.
    UndefinedClass { name: String },
    /// Emitted by `mir-analyzer/src/expr/objects.rs`.
    /// Fixtures: `tests/fixtures/by-kind/undefined_property/`.
    UndefinedProperty { class: String, property: String },
    /// Emitted by `mir-analyzer/src/expr/variables.rs`.
    /// Fixtures: `tests/fixtures/by-kind/undefined_constant/`.
    UndefinedConstant { name: String },
    /// Emitted by `mir-analyzer/src/expr/objects.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_argument/invalid_*_class_const_fetch*.phpt`.
    InaccessibleClassConstant { class: String, constant: String },
    /// Emitted by `mir-analyzer/src/expr/variables.rs`.
    /// Fixtures: `tests/fixtures/by-kind/possibly_undefined_variable/`.
    PossiblyUndefinedVariable { name: String },
    /// Emitted by `mir-analyzer/src/body_analysis/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/undefined_trait/`.
    UndefinedTrait { name: String },
    /// Emitted when `parent::` is used in a class that has no parent.
    /// Fixtures: `tests/fixtures/by-kind/undefined_class/no_parent*.phpt`.
    ParentNotFound,
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
    /// Emitted by `mir-analyzer/src/stmt/flow.rs`.
    /// Fixtures: `tests/fixtures/by-kind/nullable_return_statement/`.
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
    /// Emitted when a function/method tagged `@no-named-arguments` is called with named args.
    /// Fixtures: `tests/fixtures/by-kind/invalid_named_argument/`.
    InvalidNamedArguments { fn_name: String },
    /// Emitted by `mir-analyzer/src/call/args.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_pass_by_reference/`.
    InvalidPassByReference { fn_name: String, param: String },
    /// Emitted by `mir-analyzer/src/expr/objects.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_property_fetch/bad_fetch.phpt`.
    InvalidPropertyFetch { ty: String },
    /// Emitted by `mir-analyzer/src/expr/arrays.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_array_access/`.
    InvalidArrayAccess { ty: String },
    /// Emitted by `mir-analyzer/src/expr/arrays.rs`.
    /// Fixtures: `tests/fixtures/by-kind/possibly_invalid_array_access/`.
    PossiblyInvalidArrayAccess { ty: String },
    /// Emitted by `mir-analyzer/src/expr/assignment.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_array_assignment/`.
    InvalidArrayAssignment { ty: String },
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
    /// Emitted by `mir-analyzer/src/call/static_call.rs`.
    /// Fixtures: `tests/fixtures/by-kind/undefined_method/static_invocation*.phpt`.
    InvalidStaticInvocation { class: String, method: String },
    /// Emitted by `mir-analyzer/src/expr/binary.rs` and `unary.rs` for operations on
    /// non-numeric or non-bitwise-compatible operands.
    /// Fixtures: `tests/fixtures/by-kind/invalid_operand/`.
    InvalidOperand {
        op: String,
        left: String,
        right: String,
    },
    /// Emitted when a union-typed operand has some non-numeric/non-stringifiable members.
    /// Fixtures: `tests/fixtures/by-kind/invalid_operand/`.
    PossiblyInvalidOperand {
        op: String,
        left: String,
        right: String,
    },
    /// Emitted when a divisor operand could be null (potential division by zero).
    /// Fixtures: `tests/fixtures/by-kind/invalid_operand/`.
    PossiblyNullOperand { op: String, ty: String },
    /// Emitted when `yield from` is used with a non-iterable object (no Traversable).
    /// Fixtures: `tests/fixtures/by-kind/invalid_operand/`.
    RawObjectIteration { ty: String },
    /// Emitted when `yield from` might be used with a non-iterable object.
    /// Fixtures: `tests/fixtures/by-kind/invalid_operand/`.
    PossiblyRawObjectIteration { ty: String },
    /// Emitted by `mir-analyzer/src/body_analysis/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/mismatching_docblock_return_type/`.
    MismatchingDocblockReturnType { declared: String, inferred: String },
    /// Emitted by `mir-analyzer/src/body_analysis/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/mismatching_docblock_param_type/`.
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

    /// Emitted by `@trace $var` docblock annotation. Shows inferred type.
    /// Fixtures: `tests/fixtures/by-kind/trace/`.
    Trace { variable: String, type_info: String },

    // --- Array issues -------------------------------------------------------
    /// Emitted by `mir-analyzer/src/expr/arrays.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_array_offset/`.
    InvalidArrayOffset { expected: String, actual: String },
    /// Emitted by `mir-analyzer/src/expr/arrays.rs` when a TKeyedArray is accessed with
    /// a literal key that does not exist in the shape.
    /// Fixtures: `tests/fixtures/by-kind/invalid_array_offset/`.
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
    /// Emitted by `mir-analyzer/src/stmt/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/unnecessary_var_annotation/`.
    UnnecessaryVarAnnotation { var: String },
    /// Emitted by `mir-analyzer/src/stmt/control_flow.rs` and `mir-analyzer/src/expr/conditional.rs`.
    /// Fixtures: `tests/fixtures/by-kind/type_does_not_contain_type/`.
    TypeDoesNotContainType { left: String, right: String },
    /// Emitted by `mir-analyzer/src/stmt/control_flow.rs` and `mir-analyzer/src/expr/conditional.rs`.
    /// Fixtures: `tests/fixtures/by-kind/paradoxical_condition/`.
    ParadoxicalCondition { value: String },
    /// A docblock-declared type makes a subsequent assertion or comparison
    /// impossible (e.g. `assert($a < 4)` on a `@param int<5, max> $a`).
    /// Emitted by `mir-analyzer/src/narrowing.rs`.
    /// Fixtures: `tests/fixtures/by-kind/docblock_type_contradiction/`.
    DocblockTypeContradiction { expr: String, declared: String },
    /// A `===` or `!==` comparison between two types that can never be strictly
    /// equal — e.g. `$int === $string` or `$obj !== null` where `$obj` is a
    /// non-nullable typed value.
    /// Emitted by `mir-analyzer/src/expr/binary.rs`.
    /// Fixtures: `tests/fixtures/by-kind/impossible_identical_comparison/`.
    ImpossibleIdenticalComparison {
        op: String,
        left: String,
        right: String,
    },
    /// A `==` or `!=` comparison between two types that can never be loosely
    /// equal in PHP — e.g. `$obj == null`, `$arr == "foo"`, or a non-empty
    /// array `== false`.  PHP's type-juggling rules make these always false (or
    /// always true for `!=`), which almost certainly indicates a logic bug.
    /// Emitted by `mir-analyzer/src/expr/binary.rs`.
    /// Fixtures: `tests/fixtures/by-kind/impossible_loose_comparison/`.
    ImpossibleLooseComparison {
        op: String,
        left: String,
        right: String,
    },
    /// A `switch`/`match` arm that can never be reached given the subject's
    /// inferred type — most often a `gettype()` arm tested against a string
    /// that `gettype()` never returns (e.g. `case "int"` — it returns
    /// `"integer"`).
    /// Emitted by `mir-analyzer/src/stmt/control_flow.rs` and `mir-analyzer/src/expr/conditional.rs`.
    /// Fixtures: `tests/fixtures/by-kind/unevaluated_code/`.
    UnevaluatedCode { reason: String },

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
    /// Emitted by `mir-analyzer/src/expr/conditional.rs`.
    /// Fixtures: `tests/fixtures/by-kind/unreachable_code/`.
    UnhandledMatchCondition { detail: String },
    /// Emitted by `mir-analyzer/src/dead_code.rs`.
    /// Fixtures: `tests/fixtures/by-kind/unused_method/`.
    UnusedMethod { class: String, method: String },
    /// Emitted by `mir-analyzer/src/dead_code.rs`.
    /// Fixtures: `tests/fixtures/by-kind/unused_property/`.
    UnusedProperty { class: String, property: String },
    /// Emitted by `mir-analyzer/src/dead_code.rs`.
    /// Fixtures: `tests/fixtures/by-kind/unused_function/`.
    UnusedFunction { name: String },
    /// Emitted by `mir-analyzer/src/diagnostics.rs`.
    /// Fixtures: `tests/fixtures/by-kind/unused_foreach_value/`.
    UnusedForeachValue { name: String },
    /// Emitted by `mir-analyzer/src/dead_code.rs`.
    /// Fixtures: `tests/fixtures/by-kind/unused_class/`.
    UnusedClass { class: String },
    /// Emitted by `mir-analyzer/src/batch/mod.rs` when a `@psalm-suppress` /
    /// `@mir-suppress` / `@suppress` annotation does not match any actual issue.
    /// Fixtures: `tests/fixtures/by-kind/unused_suppress/`.
    UnusedSuppress { kind: String },

    /// Emitted by `mir-analyzer/src/call/args/types.rs`.
    /// Fixtures: `tests/fixtures/by-kind/argument_type_coercion/`.
    ArgumentTypeCoercion {
        param: String,
        fn_name: String,
        expected: String,
        actual: String,
    },

    /// Emitted by `mir-analyzer/src/expr/assignment.rs`.
    /// Fixtures: `tests/fixtures/by-kind/property_type_coercion/`.
    PropertyTypeCoercion {
        property: String,
        expected: String,
        actual: String,
    },

    // --- Purity -------------------------------------------------------------
    /// Emitted when a @pure function assigns to a parameter's property.
    ImpurePropertyAssignment { property: String },
    /// Emitted when a @pure function calls an impure method on a parameter.
    ImpureMethodCall { method: String },
    /// Emitted when a @pure function uses a global variable.
    ImpureGlobalVariable { variable: String },
    /// Emitted when a @pure function uses a static variable.
    ImpureStaticVariable { variable: String },
    /// Emitted by `mir-analyzer/src/call/function.rs` when a `@pure` function calls a
    /// non-pure named function.
    /// Fixtures: `tests/fixtures/by-kind/impure_function_call/`.
    ImpureFunctionCall { fn_name: String },
    /// Emitted when a non-constructor method of a `@psalm-immutable` class assigns to a
    /// `$this` property.
    /// Fixtures: `tests/fixtures/by-kind/immutable_property_modification/`.
    ImmutablePropertyModification { property: String },

    // --- Readonly -----------------------------------------------------------
    /// Emitted by `mir-analyzer/src/expr/assignment.rs`.
    /// Fixtures: `tests/fixtures/by-kind/readonly_property_assignment/`.
    ReadonlyPropertyAssignment { class: String, property: String },

    // --- Inheritance --------------------------------------------------------
    /// Emitted by `mir-analyzer/src/class/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/unimplemented_abstract_method/`.
    UnimplementedAbstractMethod { class: String, method: String },
    /// Emitted by `mir-analyzer/src/class/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/unimplemented_interface_method/`.
    UnimplementedInterfaceMethod {
        class: String,
        interface: String,
        method: String,
    },
    /// Emitted by `mir-analyzer/src/class/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/method_signature_mismatch/`.
    MethodSignatureMismatch {
        class: String,
        method: String,
        detail: String,
    },
    /// Emitted by `mir-analyzer/src/class/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/overridden_method_access/`.
    OverriddenMethodAccess { class: String, method: String },
    /// Emitted by `mir-analyzer/src/class/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/overridden_property_access/`.
    OverriddenPropertyAccess { class: String, property: String },
    /// Emitted by `mir-analyzer/src/class/overrides.rs`.
    /// Fixtures: `tests/fixtures/by-kind/property_type_redeclaration_mismatch/`.
    PropertyTypeRedeclarationMismatch {
        class: String,
        property: String,
        expected: String,
        actual: String,
    },
    /// Emitted by `mir-analyzer/src/collector/enum.rs`.
    /// Fixtures: `tests/fixtures/by-kind/backed_enum_case_type_mismatch/`.
    BackedEnumCaseTypeMismatch {
        enum_name: String,
        case_name: String,
        expected: String,
        actual: String,
    },
    /// Emitted by `mir-analyzer/src/call/method.rs`.
    /// Fixtures: `tests/fixtures/by-kind/undefined_method/direct_constructor_call*.phpt`.
    DirectConstructorCall { class: String },
    /// Emitted by `mir-analyzer/src/class/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_extend_class/`.
    InvalidExtendClass { parent: String, child: String },
    /// Emitted by `mir-analyzer/src/class/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/final_method_overridden/`.
    FinalMethodOverridden {
        class: String,
        method: String,
        parent: String,
    },
    /// Emitted by `mir-analyzer/src/expr/objects.rs`.
    /// Fixtures: `tests/fixtures/by-kind/abstract_instantiation/`.
    AbstractInstantiation { class: String },
    /// Emitted by `mir-analyzer/src/call/static_call.rs`.
    /// Fixtures: `tests/fixtures/by-kind/abstract_instantiation/prevent_abstract_method_call.phpt`.
    AbstractMethodCall { class: String, method: String },
    /// Emitted by `mir-analyzer/src/expr/objects.rs`.
    /// Fixtures: `tests/fixtures/by-kind/abstract_instantiation/interface_instantiation.phpt`.
    InterfaceInstantiation { class: String },
    /// Emitted by `mir-analyzer/src/class/mod.rs` when `#[Override]` is declared
    /// but no overridable parent method exists.
    /// Fixtures: `tests/fixtures/by-kind/method_signature_mismatch/`.
    InvalidOverride {
        class: String,
        method: String,
        detail: String,
    },

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
    /// Emitted by `mir-analyzer/src/call/method.rs` when a tainted value reaches a
    /// `@taint-sink llm_prompt` annotated parameter.
    /// Fixtures: `tests/fixtures/by-kind/tainted_llm_prompt/`.
    TaintedLlmPrompt,

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
    /// A method annotated `@if-this-is X<Y>` was called on a receiver whose
    /// type does not satisfy that constraint.
    /// Emitted by `mir-analyzer/src/call/method.rs`.
    /// Fixtures: `tests/fixtures/by-kind/if_this_is_mismatch/`.
    IfThisIsMismatch {
        class: String,
        method: String,
        expected: String,
        actual: String,
    },

    // --- Other --------------------------------------------------------------
    /// Emitted by `mir-analyzer/src/call/function.rs`.
    /// Fixtures: `tests/fixtures/by-kind/deprecated_call/`.
    DeprecatedCall {
        name: String,
        message: Option<Arc<str>>,
    },
    /// Emitted by `mir-analyzer/src/expr/objects.rs`.
    /// Fixtures: `tests/fixtures/by-kind/undefined_property/deprecated_property_*.phpt`.
    DeprecatedProperty {
        class: String,
        property: String,
        message: Option<Arc<str>>,
    },
    /// Emitted by `mir-analyzer/src/expr/objects.rs`.
    /// Fixtures: `tests/fixtures/by-kind/deprecated_call/deprecated_class_const_fetch*.phpt`.
    DeprecatedConstant {
        class: String,
        constant: String,
        message: Option<Arc<str>>,
    },
    /// Emitted by `mir-analyzer/src/class/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/deprecated_interface/`.
    DeprecatedInterface {
        name: String,
        message: Option<Arc<str>>,
    },
    /// Emitted by `mir-analyzer/src/class/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/deprecated_trait/`.
    DeprecatedTrait {
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
    /// Emitted by `mir-analyzer/src/class/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/deprecated_class/`.
    DeprecatedClass {
        name: String,
        message: Option<Arc<str>>,
    },
    /// Emitted by `mir-analyzer/src/call/method.rs`.
    /// Fixtures: `tests/fixtures/by-kind/internal_method/`.
    InternalMethod { class: String, method: String },
    /// Emitted by `mir-analyzer/src/body_analysis/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/missing_return_type/`.
    MissingReturnType { fn_name: String },
    /// Emitted by `mir-analyzer/src/expr/closures.rs`.
    /// Fixtures: `tests/fixtures/by-kind/missing_closure_return_type/`.
    MissingClosureReturnType,
    /// Emitted by `mir-analyzer/src/body_analysis/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/missing_param_type/`.
    MissingParamType { fn_name: String, param: String },
    /// Emitted by `mir-analyzer/src/body_analysis/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/missing_param_type/` (property variants).
    MissingPropertyType { class: String, property: String },
    /// Emitted by `mir-analyzer/src/stmt/flow.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_throw/`.
    InvalidThrow { ty: String },
    /// Emitted by `mir-analyzer/src/stmt/control_flow.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_catch/`.
    InvalidCatch { ty: String },
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
    /// Emitted by `mir-analyzer/src/call/args/types.rs`.
    /// Fixtures: `tests/fixtures/by-kind/mixed_argument/`.
    MixedArgument { param: String, fn_name: String },
    /// Emitted by `mir-analyzer/src/expr/assignment.rs` and `mir-analyzer/src/stmt/control_flow.rs`.
    /// Fixtures: `tests/fixtures/by-kind/mixed_assignment/`.
    MixedAssignment { var: String },
    /// Emitted by `mir-analyzer/src/call/method.rs`.
    /// Fixtures: `tests/fixtures/by-kind/mixed_method_call/`.
    MixedMethodCall { method: String },
    /// Emitted when a PHP reference assignment is used (e.g. `$b = &$arr[$x]`).
    /// Fixtures: `tests/fixtures/by-kind/unsupported_reference_usage/`.
    UnsupportedReferenceUsage,
    /// Emitted when a property is accessed on an interface that has `@seal-properties`
    /// but the property is not declared with `@property`/`@property-read`/`@property-write`.
    /// Fixtures: `tests/fixtures/by-kind/undefined_property/magic_interface_*.phpt`.
    NoInterfaceProperties { property: String },
    /// Emitted when a class referenced only in a docblock (`@return`, `@param`, etc.)
    /// does not exist.
    /// Fixtures: `tests/fixtures/by-kind/mixed_clone/missing_class.phpt`.
    UndefinedDocblockClass { name: String },
    /// Emitted when a class with non-nullable uninitialized properties has no constructor.
    /// Fixtures: `tests/fixtures/by-kind/missing_constructor/`.
    MissingConstructor { class: String },
    /// Emitted by `mir-analyzer/src/call/function.rs` when a dynamic call target is mixed.
    /// Fixtures: `tests/fixtures/by-kind/mixed_function_call/`.
    MixedFunctionCall,
    /// Emitted by `mir-analyzer/src/stmt/flow.rs`.
    /// Fixtures: `tests/fixtures/by-kind/mixed_return_statement/`.
    MixedReturnStatement { declared: String },
    /// Emitted by `mir-analyzer/src/expr/objects.rs`.
    /// Fixtures: `tests/fixtures/by-kind/mixed_property_fetch/`.
    MixedPropertyFetch { property: String },
    /// Emitted by `mir-analyzer/src/expr/assignment.rs`.
    /// Fixtures: `tests/fixtures/by-kind/mixed_property_assignment/`.
    MixedPropertyAssignment { property: String },
    /// Emitted by `mir-analyzer/src/expr/arrays.rs`.
    /// Fixtures: `tests/fixtures/by-kind/mixed_array_access/`.
    MixedArrayAccess,
    /// Emitted by `mir-analyzer/src/expr/arrays.rs`.
    /// Fixtures: `tests/fixtures/by-kind/mixed_array_offset/`.
    MixedArrayOffset,
    /// Emitted by `mir-analyzer/src/expr/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/mixed_clone/`.
    MixedClone,
    /// `clone` of a value that is definitely not an object (e.g. `int`, `string`).
    /// Emitted by `mir-analyzer/src/expr/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/mixed_clone/`.
    InvalidClone { ty: String },
    /// `clone` of a union where some members are not objects (e.g. `int|Exception`).
    /// Emitted by `mir-analyzer/src/expr/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/mixed_clone/`.
    PossiblyInvalidClone { ty: String },
    /// A `__toString` method that does not return a `string`.
    /// Emitted by `mir-analyzer/src/body_analysis/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/implicit_to_string_cast/`.
    InvalidToString { class: String },
    /// Emitted by `mir-analyzer/src/class/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/circular_inheritance/`.
    CircularInheritance { class: String },

    // --- Trait constraints --------------------------------------------------
    /// Emitted by `mir-analyzer/src/body_analysis/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_trait_use/`.
    InvalidTraitUse { trait_name: String, reason: String },
    /// Emitted by `mir-analyzer/src/expr/mod.rs` and `mir-analyzer/src/call/function.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_operand/` (var_dump, shell_exec, backtick).
    ForbiddenCode { message: String },

    // --- Attribute validation -----------------------------------------------
    /// Emitted by `mir-analyzer/src/attributes.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_attribute/`.
    InvalidAttribute { message: String },
    /// Emitted by `mir-analyzer/src/attributes.rs`.
    /// Fixtures: `tests/fixtures/by-kind/undefined_class/missing_attribute_on_*.phpt`.
    UndefinedAttributeClass { name: String },

    // --- Case sensitivity (PHP 8.6 deprecation) -----------------------------
    /// Emitted by `mir-analyzer/src/call/function.rs`.
    /// Fixtures: `tests/fixtures/by-kind/wrong_case_function/`.
    WrongCaseFunction { used: String, canonical: String },
    /// Emitted by `mir-analyzer/src/call/method.rs` and `src/call/static_call.rs`.
    /// Fixtures: `tests/fixtures/by-kind/wrong_case_method/`.
    WrongCaseMethod {
        class: String,
        used: String,
        canonical: String,
    },
    /// Emitted by `mir-analyzer/src/expr/objects.rs` and `src/call/static_call.rs`.
    /// Fixtures: `tests/fixtures/by-kind/wrong_case_class/`.
    WrongCaseClass { used: String, canonical: String },
    /// Emitted by `mir-analyzer/src/body_analysis/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_argument/class_redefinition*.phpt`.
    DuplicateClass { name: String },
    /// Emitted by `mir-analyzer/src/body_analysis/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_argument/interface_redefinition*.phpt`.
    DuplicateInterface { name: String },
    /// Emitted by `mir-analyzer/src/body_analysis/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_argument/trait_redefinition*.phpt`.
    DuplicateTrait { name: String },
    /// Emitted by `mir-analyzer/src/body_analysis/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_argument/enum_redefinition*.phpt`.
    DuplicateEnum { name: String },
    /// Emitted by `mir-analyzer/src/body_analysis/mod.rs`.
    /// Fixtures: `tests/fixtures/by-kind/invalid_argument/function_redefinition*.phpt`.
    DuplicateFunction { name: String },
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
            IssueKind::NonStaticSelfCall { .. }
            | IssueKind::DirectConstructorCall { .. }
            | IssueKind::InvalidScope { .. }
            | IssueKind::UndefinedVariable { .. }
            | IssueKind::UndefinedFunction { .. }
            | IssueKind::UndefinedMethod { .. }
            | IssueKind::UndefinedClass { .. }
            | IssueKind::UndefinedConstant { .. }
            | IssueKind::InaccessibleClassConstant { .. }
            | IssueKind::InvalidReturnType { .. }
            | IssueKind::InvalidArgument { .. }
            | IssueKind::TooFewArguments { .. }
            | IssueKind::TooManyArguments { .. }
            | IssueKind::InvalidNamedArgument { .. }
            | IssueKind::InvalidNamedArguments { .. }
            | IssueKind::InvalidPassByReference { .. }
            | IssueKind::InvalidThrow { .. }
            | IssueKind::InvalidCatch { .. }
            | IssueKind::InvalidStaticInvocation { .. }
            | IssueKind::UnimplementedAbstractMethod { .. }
            | IssueKind::UnimplementedInterfaceMethod { .. }
            | IssueKind::MethodSignatureMismatch { .. }
            | IssueKind::InvalidExtendClass { .. }
            | IssueKind::FinalMethodOverridden { .. }
            | IssueKind::AbstractInstantiation { .. }
            | IssueKind::AbstractMethodCall { .. }
            | IssueKind::InterfaceInstantiation { .. }
            | IssueKind::InvalidOverride { .. }
            | IssueKind::InvalidTemplateParam { .. }
            | IssueKind::ReadonlyPropertyAssignment { .. }
            | IssueKind::ParseError { .. }
            | IssueKind::TaintedInput { .. }
            | IssueKind::TaintedHtml
            | IssueKind::TaintedSql
            | IssueKind::TaintedShell
            | IssueKind::TaintedLlmPrompt
            | IssueKind::CircularInheritance { .. }
            | IssueKind::InvalidTraitUse { .. }
            | IssueKind::UndefinedTrait { .. }
            | IssueKind::InvalidClone { .. }
            | IssueKind::InvalidToString { .. }
            | IssueKind::TypeCheckMismatch { .. }
            | IssueKind::PropertyTypeRedeclarationMismatch { .. }
            | IssueKind::BackedEnumCaseTypeMismatch { .. }
            | IssueKind::ParentNotFound => Severity::Error,
            IssueKind::Trace { .. } => Severity::Info,

            // Warnings (shown at default error level)
            IssueKind::NullArgument { .. }
            | IssueKind::NullPropertyFetch { .. }
            | IssueKind::NullMethodCall { .. }
            | IssueKind::NullArrayAccess
            | IssueKind::NullableReturnStatement { .. }
            | IssueKind::InvalidPropertyFetch { .. }
            | IssueKind::InvalidArrayAccess { .. }
            | IssueKind::InvalidArrayAssignment { .. }
            | IssueKind::InvalidPropertyAssignment { .. }
            | IssueKind::InvalidArrayOffset { .. }
            | IssueKind::NonExistentArrayOffset { .. }
            | IssueKind::PossiblyInvalidArrayOffset { .. }
            | IssueKind::UndefinedProperty { .. }
            | IssueKind::InvalidOperand { .. }
            | IssueKind::OverriddenMethodAccess { .. }
            | IssueKind::OverriddenPropertyAccess { .. }
            | IssueKind::ImplicitToStringCast { .. }
            | IssueKind::ImplicitFloatToIntCast { .. }
            | IssueKind::UnusedVariable { .. }
            | IssueKind::UnusedForeachValue { .. }
            | IssueKind::ImpurePropertyAssignment { .. }
            | IssueKind::ImpureMethodCall { .. }
            | IssueKind::ImpureGlobalVariable { .. }
            | IssueKind::ImpureStaticVariable { .. }
            | IssueKind::ImpureFunctionCall { .. }
            | IssueKind::ImmutablePropertyModification { .. }
            | IssueKind::UnsupportedReferenceUsage
            | IssueKind::ParadoxicalCondition { .. }
            | IssueKind::UnhandledMatchCondition { .. }
            | IssueKind::InvalidStringClass { .. }
            | IssueKind::ImpossibleIdenticalComparison { .. }
            | IssueKind::ImpossibleLooseComparison { .. }
            | IssueKind::ForbiddenCode { .. } => Severity::Warning,

            // PossiblyUndefined: shown at default error level (same as Warning)
            IssueKind::PossiblyUndefinedVariable { .. } => Severity::Warning,

            // Possibly-null / possibly-invalid (only shown in strict mode, level ≥ 7)
            IssueKind::PossiblyNullArgument { .. }
            | IssueKind::PossiblyInvalidArgument { .. }
            | IssueKind::PossiblyNullPropertyFetch { .. }
            | IssueKind::PossiblyNullMethodCall { .. }
            | IssueKind::PossiblyNullArrayAccess
            | IssueKind::PossiblyInvalidArrayAccess { .. }
            | IssueKind::PossiblyInvalidClone { .. }
            | IssueKind::PossiblyInvalidOperand { .. }
            | IssueKind::PossiblyNullOperand { .. }
            | IssueKind::PossiblyRawObjectIteration { .. } => Severity::Info,

            IssueKind::RawObjectIteration { .. } => Severity::Warning,

            // Info
            IssueKind::RedundantCondition { .. }
            | IssueKind::RedundantCast { .. }
            | IssueKind::UnnecessaryVarAnnotation { .. }
            | IssueKind::TypeDoesNotContainType { .. }
            | IssueKind::DocblockTypeContradiction { .. }
            | IssueKind::UnevaluatedCode { .. }
            | IssueKind::IfThisIsMismatch { .. }
            | IssueKind::UnusedParam { .. }
            | IssueKind::UnreachableCode
            | IssueKind::UnusedMethod { .. }
            | IssueKind::UnusedProperty { .. }
            | IssueKind::UnusedFunction { .. }
            | IssueKind::UnusedClass { .. }
            | IssueKind::UnusedSuppress { .. }
            | IssueKind::ArgumentTypeCoercion { .. }
            | IssueKind::PropertyTypeCoercion { .. }
            | IssueKind::DeprecatedCall { .. }
            | IssueKind::DeprecatedProperty { .. }
            | IssueKind::DeprecatedConstant { .. }
            | IssueKind::DeprecatedInterface { .. }
            | IssueKind::DeprecatedTrait { .. }
            | IssueKind::DeprecatedMethodCall { .. }
            | IssueKind::DeprecatedMethod { .. }
            | IssueKind::DeprecatedClass { .. }
            | IssueKind::InternalMethod { .. }
            | IssueKind::MissingReturnType { .. }
            | IssueKind::MissingClosureReturnType
            | IssueKind::MissingParamType { .. }
            | IssueKind::MissingPropertyType { .. }
            | IssueKind::MismatchingDocblockReturnType { .. }
            | IssueKind::MismatchingDocblockParamType { .. }
            | IssueKind::InvalidDocblock { .. }
            | IssueKind::InvalidCast { .. }
            | IssueKind::MixedArgument { .. }
            | IssueKind::MixedAssignment { .. }
            | IssueKind::MixedMethodCall { .. }
            | IssueKind::NoInterfaceProperties { .. }
            | IssueKind::UndefinedDocblockClass { .. }
            | IssueKind::MissingConstructor { .. }
            | IssueKind::MixedFunctionCall
            | IssueKind::MixedReturnStatement { .. }
            | IssueKind::MixedPropertyFetch { .. }
            | IssueKind::MixedPropertyAssignment { .. }
            | IssueKind::MixedArrayAccess
            | IssueKind::MixedArrayOffset
            | IssueKind::MixedClone
            | IssueKind::ShadowedTemplateParam { .. }
            | IssueKind::MissingThrowsDocblock { .. }
            | IssueKind::WrongCaseFunction { .. }
            | IssueKind::WrongCaseMethod { .. }
            | IssueKind::WrongCaseClass { .. }
            | IssueKind::InvalidAttribute { .. }
            | IssueKind::UndefinedAttributeClass { .. } => Severity::Info,
            IssueKind::DuplicateClass { .. }
            | IssueKind::DuplicateInterface { .. }
            | IssueKind::DuplicateTrait { .. }
            | IssueKind::DuplicateEnum { .. }
            | IssueKind::DuplicateFunction { .. } => Severity::Error,
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
            IssueKind::NonStaticSelfCall { .. } => "MIR0216",
            IssueKind::DirectConstructorCall { .. } => "MIR0217",
            IssueKind::InvalidScope { .. } => "MIR0001",
            IssueKind::UndefinedVariable { .. } => "MIR0002",
            IssueKind::UndefinedFunction { .. } => "MIR0003",
            IssueKind::UndefinedMethod { .. } => "MIR0004",
            IssueKind::UndefinedClass { .. } => "MIR0005",
            IssueKind::UndefinedProperty { .. } => "MIR0006",
            IssueKind::UndefinedConstant { .. } => "MIR0007",
            IssueKind::InaccessibleClassConstant { .. } => "MIR0011",
            IssueKind::PossiblyUndefinedVariable { .. } => "MIR0008",
            IssueKind::UndefinedTrait { .. } => "MIR0009",
            IssueKind::ParentNotFound => "MIR0010",

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
            IssueKind::InvalidNamedArguments { .. } => "MIR0224",
            IssueKind::InvalidPassByReference { .. } => "MIR0205",
            IssueKind::InvalidPropertyFetch { .. } => "MIR0218",
            IssueKind::InvalidArrayAccess { .. } => "MIR0219",
            IssueKind::PossiblyInvalidArrayAccess { .. } => "MIR0227",
            IssueKind::InvalidArrayAssignment { .. } => "MIR0220",
            IssueKind::InvalidPropertyAssignment { .. } => "MIR0206",
            IssueKind::InvalidCast { .. } => "MIR0207",
            IssueKind::InvalidStaticInvocation { .. } => "MIR0215",
            IssueKind::InvalidOperand { .. } => "MIR0208",
            IssueKind::PossiblyInvalidOperand { .. } => "MIR0213",
            IssueKind::PossiblyNullOperand { .. } => "MIR0214",
            IssueKind::RawObjectIteration { .. } => "MIR0222",
            IssueKind::PossiblyRawObjectIteration { .. } => "MIR0223",
            IssueKind::MismatchingDocblockReturnType { .. } => "MIR0209",
            IssueKind::MismatchingDocblockParamType { .. } => "MIR0210",
            IssueKind::InvalidStringClass { .. } => "MIR0211",
            IssueKind::TypeCheckMismatch { .. } => "MIR0212",
            IssueKind::Trace { .. } => "MIR0221",
            IssueKind::ArgumentTypeCoercion { .. } => "MIR0225",
            IssueKind::PropertyTypeCoercion { .. } => "MIR0226",

            // Array / offset (0300-0399)
            IssueKind::InvalidArrayOffset { .. } => "MIR0300",
            IssueKind::NonExistentArrayOffset { .. } => "MIR0301",
            IssueKind::PossiblyInvalidArrayOffset { .. } => "MIR0302",

            // Redundancy (0400-0499)
            IssueKind::RedundantCondition { .. } => "MIR0400",
            IssueKind::RedundantCast { .. } => "MIR0401",
            IssueKind::UnnecessaryVarAnnotation { .. } => "MIR0402",
            IssueKind::TypeDoesNotContainType { .. } => "MIR0403",
            IssueKind::ParadoxicalCondition { .. } => "MIR0404",
            IssueKind::UnhandledMatchCondition { .. } => "MIR0405",
            IssueKind::DocblockTypeContradiction { .. } => "MIR0406",
            IssueKind::UnevaluatedCode { .. } => "MIR0407",
            IssueKind::ImpossibleIdenticalComparison { .. } => "MIR0408",
            IssueKind::ImpossibleLooseComparison { .. } => "MIR0409",

            // Dead code (0500-0599)
            IssueKind::UnusedVariable { .. } => "MIR0500",
            IssueKind::UnusedParam { .. } => "MIR0501",
            IssueKind::UnreachableCode => "MIR0502",
            IssueKind::UnusedMethod { .. } => "MIR0503",
            IssueKind::UnusedProperty { .. } => "MIR0504",
            IssueKind::UnusedFunction { .. } => "MIR0505",
            IssueKind::UnusedForeachValue { .. } => "MIR0506",
            IssueKind::UnusedClass { .. } => "MIR0507",
            IssueKind::UnusedSuppress { .. } => "MIR0508",

            // Purity (1700-1799)
            IssueKind::ImpurePropertyAssignment { .. } => "MIR1700",
            IssueKind::ImpureMethodCall { .. } => "MIR1701",
            IssueKind::ImpureGlobalVariable { .. } => "MIR1702",
            IssueKind::ImpureStaticVariable { .. } => "MIR1703",
            IssueKind::ImpureFunctionCall { .. } => "MIR1704",
            IssueKind::ImmutablePropertyModification { .. } => "MIR1705",
            IssueKind::UnsupportedReferenceUsage => "MIR1506",
            IssueKind::NoInterfaceProperties { .. } => "MIR1504",
            IssueKind::UndefinedDocblockClass { .. } => "MIR1505",
            IssueKind::MissingConstructor { .. } => "MIR1507",
            IssueKind::MixedFunctionCall => "MIR1211",
            IssueKind::MixedReturnStatement { .. } => "MIR1212",

            // Readonly (0600-0699)
            IssueKind::ReadonlyPropertyAssignment { .. } => "MIR0600",

            // Inheritance (0700-0799)
            IssueKind::UnimplementedAbstractMethod { .. } => "MIR0700",
            IssueKind::UnimplementedInterfaceMethod { .. } => "MIR0701",
            IssueKind::MethodSignatureMismatch { .. } => "MIR0702",
            IssueKind::OverriddenMethodAccess { .. } => "MIR0703",
            IssueKind::OverriddenPropertyAccess { .. } => "MIR0710",
            IssueKind::PropertyTypeRedeclarationMismatch { .. } => "MIR0712",
            IssueKind::BackedEnumCaseTypeMismatch { .. } => "MIR0713",
            IssueKind::InvalidExtendClass { .. } => "MIR0704",
            IssueKind::FinalMethodOverridden { .. } => "MIR0705",
            IssueKind::AbstractInstantiation { .. } => "MIR0706",
            IssueKind::AbstractMethodCall { .. } => "MIR0711",
            IssueKind::InterfaceInstantiation { .. } => "MIR0709",
            IssueKind::CircularInheritance { .. } => "MIR0707",
            IssueKind::InvalidOverride { .. } => "MIR0708",

            // Security / taint (0800-0899)
            IssueKind::TaintedInput { .. } => "MIR0800",
            IssueKind::TaintedHtml => "MIR0801",
            IssueKind::TaintedSql => "MIR0802",
            IssueKind::TaintedShell => "MIR0803",
            IssueKind::TaintedLlmPrompt => "MIR0804",

            // Generics (0900-0999)
            IssueKind::InvalidTemplateParam { .. } => "MIR0900",
            IssueKind::ShadowedTemplateParam { .. } => "MIR0901",
            IssueKind::IfThisIsMismatch { .. } => "MIR0902",

            // Deprecation / internal (1000-1099)
            IssueKind::DeprecatedCall { .. } => "MIR1000",
            IssueKind::WrongCaseFunction { .. } => "MIR1009",
            IssueKind::WrongCaseMethod { .. } => "MIR1010",
            IssueKind::WrongCaseClass { .. } => "MIR1011",
            IssueKind::DeprecatedProperty { .. } => "MIR1005",
            IssueKind::DeprecatedInterface { .. } => "MIR1006",
            IssueKind::DeprecatedTrait { .. } => "MIR1007",
            IssueKind::DeprecatedConstant { .. } => "MIR1008",
            IssueKind::DeprecatedMethodCall { .. } => "MIR1001",
            IssueKind::DeprecatedMethod { .. } => "MIR1002",
            IssueKind::DeprecatedClass { .. } => "MIR1003",
            IssueKind::InternalMethod { .. } => "MIR1004",

            // Missing types / docblocks (1100-1199)
            IssueKind::MissingReturnType { .. } => "MIR1100",
            IssueKind::MissingParamType { .. } => "MIR1101",
            IssueKind::MissingPropertyType { .. } => "MIR1104",
            IssueKind::MissingClosureReturnType => "MIR1105",
            IssueKind::MissingThrowsDocblock { .. } => "MIR1102",
            IssueKind::InvalidDocblock { .. } => "MIR1103",

            // Mixed (1200-1299)
            IssueKind::MixedArgument { .. } => "MIR1200",
            IssueKind::MixedAssignment { .. } => "MIR1201",
            IssueKind::MixedMethodCall { .. } => "MIR1202",
            IssueKind::MixedPropertyFetch { .. } => "MIR1203",
            IssueKind::MixedPropertyAssignment { .. } => "MIR1208",
            IssueKind::MixedArrayAccess => "MIR1209",
            IssueKind::MixedArrayOffset => "MIR1210",
            IssueKind::MixedClone => "MIR1204",
            IssueKind::InvalidClone { .. } => "MIR1205",
            IssueKind::PossiblyInvalidClone { .. } => "MIR1206",
            IssueKind::InvalidToString { .. } => "MIR1207",

            // Trait (1300-1399)
            IssueKind::InvalidTraitUse { .. } => "MIR1300",
            IssueKind::ForbiddenCode { .. } => "MIR1301",

            // Parse (1400-1499)
            IssueKind::ParseError { .. } => "MIR1400",

            // Attribute (1600-1699)
            IssueKind::InvalidAttribute { .. } => "MIR1600",
            IssueKind::UndefinedAttributeClass { .. } => "MIR1601",
            IssueKind::DuplicateClass { .. } => "MIR1602",
            IssueKind::DuplicateInterface { .. } => "MIR1603",
            IssueKind::DuplicateTrait { .. } => "MIR1604",
            IssueKind::DuplicateEnum { .. } => "MIR1605",
            IssueKind::DuplicateFunction { .. } => "MIR1606",

            // Other (1500-1599)
            IssueKind::InvalidThrow { .. } => "MIR1500",
            IssueKind::InvalidCatch { .. } => "MIR1503",
            IssueKind::ImplicitToStringCast { .. } => "MIR1501",
            IssueKind::ImplicitFloatToIntCast { .. } => "MIR1502",
        }
    }

    /// Returns the default [`Severity`] for a stable issue code (e.g. `"MIR0005"`).
    ///
    /// Useful when a caller holds a bare code string — from config, suppression
    /// annotations, or serialised diagnostics — and needs to recover the severity
    /// without constructing a full [`IssueKind`]. Returns `None` for unknown codes.
    pub fn default_severity_for_code(code: &str) -> Option<Severity> {
        match code {
            // Errors
            "MIR0001" | "MIR0002" | "MIR0003" | "MIR0004" | "MIR0005" | "MIR0007" | "MIR0009"
            | "MIR0010" | "MIR0011" | "MIR0200" | "MIR0201" | "MIR0202" | "MIR0203" | "MIR0204"
            | "MIR0205" | "MIR0212" | "MIR0215" | "MIR0216" | "MIR0217" | "MIR0224" | "MIR0600"
            | "MIR0700" | "MIR0701" | "MIR0702" | "MIR0704" | "MIR0705" | "MIR0706" | "MIR0707"
            | "MIR0708" | "MIR0709" | "MIR0711" | "MIR0712" | "MIR0713" | "MIR0800" | "MIR0801"
            | "MIR0802" | "MIR0803" | "MIR0804" | "MIR0900" | "MIR1205" | "MIR1207" | "MIR1300"
            | "MIR1400" | "MIR1500" | "MIR1503" | "MIR1602" | "MIR1603" | "MIR1604" | "MIR1605"
            | "MIR1606" => Some(Severity::Error),

            // Warnings
            "MIR0006" | "MIR0008" | "MIR0100" | "MIR0101" | "MIR0102" | "MIR0103" | "MIR0109"
            | "MIR0206" | "MIR0208" | "MIR0211" | "MIR0218" | "MIR0219" | "MIR0220" | "MIR0222"
            | "MIR0300" | "MIR0301" | "MIR0302" | "MIR0404" | "MIR0405" | "MIR0408" | "MIR0500"
            | "MIR0506" | "MIR0703" | "MIR0710" | "MIR1301" | "MIR1501" | "MIR1502" | "MIR1700"
            | "MIR1701" | "MIR1702" | "MIR1703" | "MIR1704" | "MIR1705" | "MIR1506" => {
                Some(Severity::Warning)
            }

            // Info
            "MIR0104" | "MIR0105" | "MIR0106" | "MIR0107" | "MIR0108" | "MIR0207" | "MIR0209"
            | "MIR0210" | "MIR0213" | "MIR0214" | "MIR0221" | "MIR0223" | "MIR0400" | "MIR0401"
            | "MIR0402" | "MIR0403" | "MIR0501" | "MIR0502" | "MIR0503" | "MIR0504" | "MIR0505"
            | "MIR0507" | "MIR0508" | "MIR0901" | "MIR1000" | "MIR1001" | "MIR1002" | "MIR1003"
            | "MIR1004" | "MIR1005" | "MIR1006" | "MIR1007" | "MIR1008" | "MIR1009" | "MIR1010"
            | "MIR1011" | "MIR1100" | "MIR1101" | "MIR1102" | "MIR1103" | "MIR1104" | "MIR1105"
            | "MIR1200" | "MIR1201" | "MIR1202" | "MIR1203" | "MIR1204" | "MIR1206" | "MIR1208"
            | "MIR1209" | "MIR1210" | "MIR1211" | "MIR1212" | "MIR1504" | "MIR1505" | "MIR1507"
            | "MIR1600" | "MIR1601" | "MIR0225" | "MIR0226" | "MIR0227" | "MIR0406" | "MIR0407"
            | "MIR0902" => Some(Severity::Info),

            _ => None,
        }
    }

    /// Identifier name used in config and `@psalm-suppress` / `@suppress` annotations.
    pub fn name(&self) -> &'static str {
        match self {
            IssueKind::NonStaticSelfCall { .. } => "NonStaticSelfCall",
            IssueKind::DirectConstructorCall { .. } => "DirectConstructorCall",
            IssueKind::InvalidScope { .. } => "InvalidScope",
            IssueKind::UndefinedVariable { .. } => "UndefinedVariable",
            IssueKind::UndefinedFunction { .. } => "UndefinedFunction",
            IssueKind::UndefinedMethod { .. } => "UndefinedMethod",
            IssueKind::UndefinedClass { .. } => "UndefinedClass",
            IssueKind::UndefinedProperty { .. } => "UndefinedProperty",
            IssueKind::UndefinedConstant { .. } => "UndefinedConstant",
            IssueKind::InaccessibleClassConstant { .. } => "InaccessibleClassConstant",
            IssueKind::PossiblyUndefinedVariable { .. } => "PossiblyUndefinedVariable",
            IssueKind::UndefinedTrait { .. } => "UndefinedTrait",
            IssueKind::ParentNotFound => "ParentNotFound",
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
            IssueKind::InvalidNamedArguments { .. } => "InvalidNamedArguments",
            IssueKind::InvalidPassByReference { .. } => "InvalidPassByReference",
            IssueKind::InvalidPropertyFetch { .. } => "InvalidPropertyFetch",
            IssueKind::InvalidArrayAccess { .. } => "InvalidArrayAccess",
            IssueKind::PossiblyInvalidArrayAccess { .. } => "PossiblyInvalidArrayAccess",
            IssueKind::InvalidArrayAssignment { .. } => "InvalidArrayAssignment",
            IssueKind::InvalidPropertyAssignment { .. } => "InvalidPropertyAssignment",
            IssueKind::InvalidCast { .. } => "InvalidCast",
            IssueKind::InvalidStaticInvocation { .. } => "InvalidStaticInvocation",
            IssueKind::InvalidOperand { .. } => "InvalidOperand",
            IssueKind::PossiblyInvalidOperand { .. } => "PossiblyInvalidOperand",
            IssueKind::PossiblyNullOperand { .. } => "PossiblyNullOperand",
            IssueKind::RawObjectIteration { .. } => "RawObjectIteration",
            IssueKind::PossiblyRawObjectIteration { .. } => "PossiblyRawObjectIteration",
            IssueKind::MismatchingDocblockReturnType { .. } => "MismatchingDocblockReturnType",
            IssueKind::MismatchingDocblockParamType { .. } => "MismatchingDocblockParamType",
            IssueKind::TypeCheckMismatch { .. } => "TypeCheckMismatch",
            IssueKind::DocblockTypeContradiction { .. } => "DocblockTypeContradiction",
            IssueKind::ImpossibleIdenticalComparison { .. } => "ImpossibleIdenticalComparison",
            IssueKind::ImpossibleLooseComparison { .. } => "ImpossibleLooseComparison",
            IssueKind::UnevaluatedCode { .. } => "UnevaluatedCode",
            IssueKind::IfThisIsMismatch { .. } => "IfThisIsMismatch",
            IssueKind::Trace { .. } => "Trace",
            IssueKind::InvalidArrayOffset { .. } => "InvalidArrayOffset",
            IssueKind::NonExistentArrayOffset { .. } => "NonExistentArrayOffset",
            IssueKind::PossiblyInvalidArrayOffset { .. } => "PossiblyInvalidArrayOffset",
            IssueKind::RedundantCondition { .. } => "RedundantCondition",
            IssueKind::RedundantCast { .. } => "RedundantCast",
            IssueKind::UnnecessaryVarAnnotation { .. } => "UnnecessaryVarAnnotation",
            IssueKind::TypeDoesNotContainType { .. } => "TypeDoesNotContainType",
            IssueKind::ParadoxicalCondition { .. } => "ParadoxicalCondition",
            IssueKind::UnhandledMatchCondition { .. } => "UnhandledMatchCondition",
            IssueKind::UnusedVariable { .. } => "UnusedVariable",
            IssueKind::UnusedParam { .. } => "UnusedParam",
            IssueKind::UnreachableCode => "UnreachableCode",
            IssueKind::UnusedMethod { .. } => "UnusedMethod",
            IssueKind::UnusedProperty { .. } => "UnusedProperty",
            IssueKind::UnusedFunction { .. } => "UnusedFunction",
            IssueKind::UnusedForeachValue { .. } => "UnusedForeachValue",
            IssueKind::UnusedClass { .. } => "UnusedClass",
            IssueKind::UnusedSuppress { .. } => "UnusedSuppress",
            IssueKind::ArgumentTypeCoercion { .. } => "ArgumentTypeCoercion",
            IssueKind::PropertyTypeCoercion { .. } => "PropertyTypeCoercion",
            IssueKind::ImpurePropertyAssignment { .. } => "ImpurePropertyAssignment",
            IssueKind::ImpureMethodCall { .. } => "ImpureMethodCall",
            IssueKind::ImpureGlobalVariable { .. } => "ImpureGlobalVariable",
            IssueKind::ImpureStaticVariable { .. } => "ImpureStaticVariable",
            IssueKind::ImpureFunctionCall { .. } => "ImpureFunctionCall",
            IssueKind::ImmutablePropertyModification { .. } => "ImmutablePropertyModification",
            IssueKind::UnsupportedReferenceUsage => "UnsupportedReferenceUsage",
            IssueKind::NoInterfaceProperties { .. } => "NoInterfaceProperties",
            IssueKind::UndefinedDocblockClass { .. } => "UndefinedDocblockClass",
            IssueKind::MissingConstructor { .. } => "MissingConstructor",
            IssueKind::MixedFunctionCall => "MixedFunctionCall",
            IssueKind::MixedReturnStatement { .. } => "MixedReturnStatement",
            IssueKind::UnimplementedAbstractMethod { .. } => "UnimplementedAbstractMethod",
            IssueKind::UnimplementedInterfaceMethod { .. } => "UnimplementedInterfaceMethod",
            IssueKind::MethodSignatureMismatch { .. } => "MethodSignatureMismatch",
            IssueKind::OverriddenMethodAccess { .. } => "OverriddenMethodAccess",
            IssueKind::OverriddenPropertyAccess { .. } => "OverriddenPropertyAccess",
            IssueKind::PropertyTypeRedeclarationMismatch { .. } => {
                "PropertyTypeRedeclarationMismatch"
            }
            IssueKind::BackedEnumCaseTypeMismatch { .. } => "BackedEnumCaseTypeMismatch",
            IssueKind::InvalidExtendClass { .. } => "InvalidExtendClass",
            IssueKind::FinalMethodOverridden { .. } => "FinalMethodOverridden",
            IssueKind::AbstractInstantiation { .. } => "AbstractInstantiation",
            IssueKind::AbstractMethodCall { .. } => "AbstractMethodCall",
            IssueKind::InterfaceInstantiation { .. } => "InterfaceInstantiation",
            IssueKind::InvalidOverride { .. } => "InvalidOverride",
            IssueKind::ReadonlyPropertyAssignment { .. } => "ReadonlyPropertyAssignment",
            IssueKind::InvalidTemplateParam { .. } => "InvalidTemplateParam",
            IssueKind::ShadowedTemplateParam { .. } => "ShadowedTemplateParam",
            IssueKind::TaintedInput { .. } => "TaintedInput",
            IssueKind::TaintedHtml => "TaintedHtml",
            IssueKind::TaintedSql => "TaintedSql",
            IssueKind::TaintedShell => "TaintedShell",
            IssueKind::TaintedLlmPrompt => "TaintedLlmPrompt",
            IssueKind::DeprecatedCall { .. } => "DeprecatedCall",
            IssueKind::DeprecatedProperty { .. } => "DeprecatedProperty",
            IssueKind::DeprecatedConstant { .. } => "DeprecatedConstant",
            IssueKind::DeprecatedInterface { .. } => "DeprecatedInterface",
            IssueKind::DeprecatedTrait { .. } => "DeprecatedTrait",
            IssueKind::DeprecatedMethodCall { .. } => "DeprecatedMethodCall",
            IssueKind::DeprecatedMethod { .. } => "DeprecatedMethod",
            IssueKind::DeprecatedClass { .. } => "DeprecatedClass",
            IssueKind::InternalMethod { .. } => "InternalMethod",
            IssueKind::MissingReturnType { .. } => "MissingReturnType",
            IssueKind::MissingClosureReturnType => "MissingClosureReturnType",
            IssueKind::MissingParamType { .. } => "MissingParamType",
            IssueKind::MissingPropertyType { .. } => "MissingPropertyType",
            IssueKind::InvalidThrow { .. } => "InvalidThrow",
            IssueKind::InvalidCatch { .. } => "InvalidCatch",
            IssueKind::MissingThrowsDocblock { .. } => "MissingThrowsDocblock",
            IssueKind::ImplicitToStringCast { .. } => "ImplicitToStringCast",
            IssueKind::ImplicitFloatToIntCast { .. } => "ImplicitFloatToIntCast",
            IssueKind::ParseError { .. } => "ParseError",
            IssueKind::InvalidDocblock { .. } => "InvalidDocblock",
            IssueKind::MixedArgument { .. } => "MixedArgument",
            IssueKind::MixedAssignment { .. } => "MixedAssignment",
            IssueKind::MixedMethodCall { .. } => "MixedMethodCall",
            IssueKind::MixedPropertyFetch { .. } => "MixedPropertyFetch",
            IssueKind::MixedPropertyAssignment { .. } => "MixedPropertyAssignment",
            IssueKind::MixedArrayAccess => "MixedArrayAccess",
            IssueKind::MixedArrayOffset => "MixedArrayOffset",
            IssueKind::MixedClone => "MixedClone",
            IssueKind::InvalidClone { .. } => "InvalidClone",
            IssueKind::PossiblyInvalidClone { .. } => "PossiblyInvalidClone",
            IssueKind::InvalidToString { .. } => "InvalidToString",
            IssueKind::CircularInheritance { .. } => "CircularInheritance",
            IssueKind::InvalidTraitUse { .. } => "InvalidTraitUse",
            IssueKind::ForbiddenCode { .. } => "ForbiddenCode",
            IssueKind::WrongCaseFunction { .. } => "WrongCaseFunction",
            IssueKind::WrongCaseMethod { .. } => "WrongCaseMethod",
            IssueKind::WrongCaseClass { .. } => "WrongCaseClass",
            IssueKind::InvalidAttribute { .. } => "InvalidAttribute",
            IssueKind::UndefinedAttributeClass { .. } => "UndefinedAttributeClass",
            IssueKind::DuplicateClass { .. } => "DuplicateClass",
            IssueKind::DuplicateInterface { .. } => "DuplicateInterface",
            IssueKind::DuplicateTrait { .. } => "DuplicateTrait",
            IssueKind::DuplicateEnum { .. } => "DuplicateEnum",
            IssueKind::DuplicateFunction { .. } => "DuplicateFunction",
        }
    }

    /// Human-readable message for this issue.
    pub fn message(&self) -> String {
        match self {
            IssueKind::NonStaticSelfCall { class, method } => {
                format!("Non-static method {class}::{method}() cannot be called statically")
            }
            IssueKind::DirectConstructorCall { class } => {
                format!("Cannot call constructor of {class} directly")
            }
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
            IssueKind::InaccessibleClassConstant { class, constant } => {
                format!("Cannot access constant {class}::{constant}")
            }
            IssueKind::PossiblyUndefinedVariable { name } => {
                format!("Variable ${name} might not be defined")
            }
            IssueKind::UndefinedTrait { name } => format!("Trait {name} does not exist"),
            IssueKind::ParentNotFound => {
                "Cannot use parent:: when current class has no parent".to_string()
            }
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
            IssueKind::InvalidNamedArguments { fn_name } => {
                format!("{}() does not accept named arguments", fn_name)
            }
            IssueKind::InvalidPassByReference { fn_name, param } => {
                format!(
                    "Argument ${} of {}() must be passed by reference",
                    param, fn_name
                )
            }
            IssueKind::InvalidPropertyFetch { ty } => {
                format!("Cannot fetch property on non-object type '{ty}'")
            }
            IssueKind::InvalidArrayAccess { ty } => {
                format!("Cannot use [] operator on non-array type '{ty}'")
            }
            IssueKind::PossiblyInvalidArrayAccess { ty } => {
                format!("Possibly invalid array access: '{ty}' might not support []")
            }
            IssueKind::InvalidArrayAssignment { ty } => {
                format!("Cannot use [] assignment on non-array type '{ty}'")
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
            IssueKind::InvalidStaticInvocation { class, method } => {
                format!("Non-static method {class}::{method}() cannot be called statically")
            }
            IssueKind::InvalidOperand { op, left, right } => {
                format!("Operator '{op}' not supported between '{left}' and '{right}'")
            }
            IssueKind::PossiblyInvalidOperand { op, left, right } => {
                format!("Operator '{op}' might not be supported between '{left}' and '{right}'")
            }
            IssueKind::PossiblyNullOperand { op, ty } => {
                format!("Operator '{op}' operand '{ty}' might be null")
            }
            IssueKind::RawObjectIteration { ty } => {
                format!("Cannot iterate over non-iterable object '{ty}'")
            }
            IssueKind::PossiblyRawObjectIteration { ty } => {
                format!("Cannot iterate over possibly non-iterable object '{ty}'")
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
            IssueKind::Trace {
                variable,
                type_info,
            } => {
                format!("Type of ${variable} is {type_info}")
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
            IssueKind::ParadoxicalCondition { value } => {
                format!("Value {value} is duplicated; this branch can never be reached")
            }
            IssueKind::UnhandledMatchCondition { detail } => {
                format!("Unhandled match condition: {detail}")
            }
            IssueKind::DocblockTypeContradiction { expr, declared } => {
                format!("Type '{declared}' makes '{expr}' impossible — this can never hold")
            }
            IssueKind::ImpossibleIdenticalComparison { op, left, right } => {
                let result = if op == "===" { "false" } else { "true" };
                format!("'{op}' between '{left}' and '{right}' is always {result} — these types can never be identical")
            }
            IssueKind::ImpossibleLooseComparison { op, left, right } => {
                let result = if op == "==" { "false" } else { "true" };
                format!("'{op}' between '{left}' and '{right}' is always {result} — these types can never be loosely equal")
            }
            IssueKind::UnevaluatedCode { reason } => {
                format!("Unevaluated code: {reason}")
            }
            IssueKind::IfThisIsMismatch {
                class,
                method,
                expected,
                actual,
            } => {
                format!(
                    "Cannot call {class}::{method}() — @if-this-is requires $this to be '{expected}', but it is '{actual}'"
                )
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
            IssueKind::UnusedForeachValue { name } => {
                format!("Foreach value ${name} is never read")
            }
            IssueKind::UnusedClass { class } => {
                format!("Class {class} is never referenced")
            }
            IssueKind::UnusedSuppress { kind } => {
                format!("Suppress annotation for '{kind}' is never used")
            }
            IssueKind::ArgumentTypeCoercion {
                param,
                fn_name,
                expected,
                actual,
            } => {
                format!("Argument ${param} of {fn_name}() expects '{expected}', got '{actual}' — coercion may fail at runtime")
            }
            IssueKind::PropertyTypeCoercion {
                property,
                expected,
                actual,
            } => {
                format!("Property ${property} expects '{expected}', cannot assign '{actual}' — coercion may fail at runtime")
            }
            IssueKind::ImpurePropertyAssignment { property } => {
                format!("Assigning to property {property} of a parameter in a @pure function")
            }
            IssueKind::ImpureMethodCall { method } => {
                format!("Calling impure method {method}() in a @pure function")
            }
            IssueKind::ImpureGlobalVariable { variable } => {
                format!("Using global variable ${variable} in a @pure function")
            }
            IssueKind::ImpureStaticVariable { variable } => {
                format!("Using static variable ${variable} in a @pure function")
            }
            IssueKind::ImpureFunctionCall { fn_name } => {
                format!("Calling impure function {fn_name}() in a @pure function")
            }
            IssueKind::ImmutablePropertyModification { property } => {
                format!("Assigning to property {property} of $this in a @psalm-immutable class")
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
            IssueKind::OverriddenPropertyAccess { class, property } => {
                format!("Property {class}::${property} overrides with less visibility")
            }
            IssueKind::PropertyTypeRedeclarationMismatch {
                class,
                property,
                expected,
                actual,
            } => {
                format!(
                    "Type of {class}::${property} must be {expected} (as in parent class), {actual} given"
                )
            }
            IssueKind::BackedEnumCaseTypeMismatch {
                enum_name,
                case_name,
                expected,
                actual,
            } => {
                format!(
                    "Backed enum case {enum_name}::{case_name} has value of type {actual}, but backing type is {expected}"
                )
            }
            IssueKind::ReadonlyPropertyAssignment { class, property } => {
                format!(
                    "Cannot assign to readonly property {class}::${property} outside of constructor"
                )
            }
            IssueKind::InvalidExtendClass { parent, child } => {
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
            IssueKind::AbstractMethodCall { class, method } => {
                format!("Cannot call abstract method {class}::{method}()")
            }
            IssueKind::InterfaceInstantiation { class } => {
                format!("Cannot instantiate interface {class}")
            }
            IssueKind::InvalidOverride {
                class,
                method,
                detail,
            } => {
                format!("Method {class}::{method}() has #[Override] but {detail}")
            }

            IssueKind::TaintedInput { sink } => format!("Tainted input reaching sink '{sink}'"),
            IssueKind::TaintedHtml => "Tainted HTML output — possible XSS".to_string(),
            IssueKind::TaintedSql => "Tainted SQL query — possible SQL injection".to_string(),
            IssueKind::TaintedShell => {
                "Tainted shell command — possible command injection".to_string()
            }
            IssueKind::TaintedLlmPrompt => {
                "Tainted LLM prompt — possible prompt injection".to_string()
            }

            IssueKind::DeprecatedCall { name, message } => {
                let base = format!("Call to deprecated function {name}");
                append_deprecation_message(base, message)
            }
            IssueKind::DeprecatedProperty {
                class,
                property,
                message,
            } => {
                let base = format!("Property {class}::${property} is deprecated");
                append_deprecation_message(base, message)
            }
            IssueKind::DeprecatedConstant {
                class,
                constant,
                message,
            } => {
                let base = format!("Constant {class}::{constant} is deprecated");
                append_deprecation_message(base, message)
            }
            IssueKind::DeprecatedInterface { name, message } => {
                let base = format!("Interface {name} is deprecated");
                append_deprecation_message(base, message)
            }
            IssueKind::DeprecatedTrait { name, message } => {
                let base = format!("Trait {name} is deprecated");
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
            IssueKind::MissingClosureReturnType => {
                "Closure has no return type annotation".to_string()
            }
            IssueKind::MissingParamType { fn_name, param } => {
                format!("Parameter ${param} of {fn_name}() has no type annotation")
            }
            IssueKind::MissingPropertyType { class, property } => {
                format!("Property {class}::${property} has no type annotation")
            }
            IssueKind::InvalidThrow { ty } => {
                format!("Thrown type '{ty}' does not extend Throwable")
            }
            IssueKind::InvalidCatch { ty } => {
                format!("Caught type '{ty}' does not extend Throwable")
            }
            IssueKind::MissingThrowsDocblock { class } => {
                format!("Exception {class} is thrown but not declared in @throws")
            }
            IssueKind::ImplicitToStringCast { class } => {
                format!("Class {class} is implicitly cast to string")
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
            IssueKind::UnsupportedReferenceUsage => {
                "Reference assignment is not supported".to_string()
            }
            IssueKind::NoInterfaceProperties { property } => {
                format!("Property ${property} is not defined on sealed interface")
            }
            IssueKind::UndefinedDocblockClass { name } => {
                format!("Docblock type '{name}' does not exist")
            }
            IssueKind::MissingConstructor { class } => {
                format!("Class {class} has uninitialized properties but no constructor")
            }
            IssueKind::MixedFunctionCall => "Cannot call mixed type as a function".to_string(),
            IssueKind::MixedReturnStatement { declared } => {
                format!("Cannot return a mixed type from function with declared return type '{declared}'")
            }
            IssueKind::MixedPropertyFetch { property } => {
                format!("Property ${property} fetched on mixed type")
            }
            IssueKind::MixedPropertyAssignment { property } => {
                format!("Property ${property} assigned on mixed type")
            }
            IssueKind::MixedArrayAccess => "Array access on mixed type".to_string(),
            IssueKind::MixedArrayOffset => "Mixed type used as array offset".to_string(),
            IssueKind::MixedClone => "cannot clone mixed".to_string(),
            IssueKind::InvalidClone { ty } => format!("cannot clone non-object {ty}"),
            IssueKind::PossiblyInvalidClone { ty } => {
                format!("cannot clone possibly non-object {ty}")
            }
            IssueKind::InvalidToString { class } => {
                format!("Method {class}::__toString() must return a string")
            }
            IssueKind::CircularInheritance { class } => {
                format!("Class {class} has a circular inheritance chain")
            }
            IssueKind::InvalidTraitUse { trait_name, reason } => {
                format!("Trait {trait_name} used incorrectly: {reason}")
            }
            IssueKind::WrongCaseFunction { used, canonical } => {
                format!("Function name '{used}' has incorrect casing; use '{canonical}'")
            }
            IssueKind::WrongCaseMethod {
                class,
                used,
                canonical,
            } => {
                format!("Method name '{class}::{used}' has incorrect casing; use '{canonical}'")
            }
            IssueKind::WrongCaseClass { used, canonical } => {
                format!("Class name '{used}' has incorrect casing; use '{canonical}'")
            }
            IssueKind::InvalidAttribute { message } => message.clone(),
            IssueKind::UndefinedAttributeClass { name } => {
                format!("Attribute class {name} does not exist")
            }
            IssueKind::ForbiddenCode { message } => message.clone(),
            IssueKind::DuplicateClass { name } => {
                format!("Class {name} has already been defined")
            }
            IssueKind::DuplicateInterface { name } => {
                format!("Interface {name} has already been defined")
            }
            IssueKind::DuplicateTrait { name } => {
                format!("Trait {name} has already been defined")
            }
            IssueKind::DuplicateEnum { name } => {
                format!("Enum {name} has already been defined")
            }
            IssueKind::DuplicateFunction { name } => {
                format!("Function {name}() has already been defined")
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

    /// Like `into_issues` but keeps suppressed issues (with `suppressed = true`)
    /// so callers that need to detect unused suppressions can see which issues
    /// were silenced. File-level suppressions are also marked `suppressed = true`
    /// rather than dropped.
    pub fn into_all_issues(self) -> Vec<Issue> {
        self.issues
            .into_iter()
            .map(|mut i| {
                if self.file_suppressions.contains(&i.kind.name().to_string()) {
                    i.suppressed = true;
                }
                i
            })
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
            IssueKind::NonStaticSelfCall {
                class: s(),
                method: s(),
            },
            IssueKind::DirectConstructorCall { class: s() },
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
            IssueKind::InaccessibleClassConstant {
                class: s(),
                constant: s(),
            },
            IssueKind::PossiblyUndefinedVariable { name: s() },
            IssueKind::UndefinedTrait { name: s() },
            IssueKind::ParentNotFound,
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
            IssueKind::InvalidNamedArguments { fn_name: s() },
            IssueKind::InvalidPassByReference {
                fn_name: s(),
                param: s(),
            },
            IssueKind::InvalidPropertyFetch { ty: s() },
            IssueKind::InvalidArrayAccess { ty: s() },
            IssueKind::PossiblyInvalidArrayAccess { ty: s() },
            IssueKind::InvalidArrayAssignment { ty: s() },
            IssueKind::InvalidPropertyAssignment {
                property: s(),
                expected: s(),
                actual: s(),
            },
            IssueKind::InvalidCast { from: s(), to: s() },
            IssueKind::InvalidStaticInvocation {
                class: s(),
                method: s(),
            },
            IssueKind::InvalidOperand {
                op: s(),
                left: s(),
                right: s(),
            },
            IssueKind::PossiblyInvalidOperand {
                op: s(),
                left: s(),
                right: s(),
            },
            IssueKind::PossiblyNullOperand { op: s(), ty: s() },
            IssueKind::RawObjectIteration { ty: s() },
            IssueKind::PossiblyRawObjectIteration { ty: s() },
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
            IssueKind::Trace {
                variable: s(),
                type_info: s(),
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
            IssueKind::UnhandledMatchCondition { detail: s() },
            IssueKind::UnusedMethod {
                class: s(),
                method: s(),
            },
            IssueKind::UnusedProperty {
                class: s(),
                property: s(),
            },
            IssueKind::UnusedFunction { name: s() },
            IssueKind::UnusedForeachValue { name: s() },
            IssueKind::UnusedClass { class: s() },
            IssueKind::UnusedSuppress { kind: s() },
            IssueKind::ArgumentTypeCoercion {
                param: s(),
                fn_name: s(),
                expected: s(),
                actual: s(),
            },
            IssueKind::PropertyTypeCoercion {
                property: s(),
                expected: s(),
                actual: s(),
            },
            IssueKind::ImpurePropertyAssignment { property: s() },
            IssueKind::ImpureMethodCall { method: s() },
            IssueKind::ImpureGlobalVariable { variable: s() },
            IssueKind::ImpureStaticVariable { variable: s() },
            IssueKind::ImpureFunctionCall { fn_name: s() },
            IssueKind::ImmutablePropertyModification { property: s() },
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
            IssueKind::OverriddenPropertyAccess {
                class: s(),
                property: s(),
            },
            IssueKind::PropertyTypeRedeclarationMismatch {
                class: s(),
                property: s(),
                expected: s(),
                actual: s(),
            },
            IssueKind::BackedEnumCaseTypeMismatch {
                enum_name: s(),
                case_name: s(),
                expected: s(),
                actual: s(),
            },
            IssueKind::InvalidExtendClass {
                parent: s(),
                child: s(),
            },
            IssueKind::FinalMethodOverridden {
                class: s(),
                method: s(),
                parent: s(),
            },
            IssueKind::AbstractInstantiation { class: s() },
            IssueKind::AbstractMethodCall {
                class: s(),
                method: s(),
            },
            IssueKind::InterfaceInstantiation { class: s() },
            IssueKind::InvalidOverride {
                class: s(),
                method: s(),
                detail: s(),
            },
            IssueKind::CircularInheritance { class: s() },
            IssueKind::TaintedInput { sink: s() },
            IssueKind::TaintedHtml,
            IssueKind::TaintedSql,
            IssueKind::TaintedShell,
            IssueKind::TaintedLlmPrompt,
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
            IssueKind::DeprecatedProperty {
                class: s(),
                property: s(),
                message: None,
            },
            IssueKind::DeprecatedConstant {
                class: s(),
                constant: s(),
                message: None,
            },
            IssueKind::DeprecatedInterface {
                name: s(),
                message: None,
            },
            IssueKind::DeprecatedTrait {
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
            IssueKind::MissingClosureReturnType,
            IssueKind::MissingParamType {
                fn_name: s(),
                param: s(),
            },
            IssueKind::MissingPropertyType {
                class: s(),
                property: s(),
            },
            IssueKind::MissingThrowsDocblock { class: s() },
            IssueKind::InvalidDocblock { message: s() },
            IssueKind::MixedArgument {
                param: s(),
                fn_name: s(),
            },
            IssueKind::MixedAssignment { var: s() },
            IssueKind::MixedMethodCall { method: s() },
            IssueKind::UnsupportedReferenceUsage,
            IssueKind::NoInterfaceProperties { property: s() },
            IssueKind::UndefinedDocblockClass { name: s() },
            IssueKind::MissingConstructor { class: s() },
            IssueKind::MixedFunctionCall,
            IssueKind::MixedReturnStatement { declared: s() },
            IssueKind::MixedPropertyFetch { property: s() },
            IssueKind::MixedPropertyAssignment { property: s() },
            IssueKind::MixedArrayAccess,
            IssueKind::MixedArrayOffset,
            IssueKind::MixedClone,
            IssueKind::InvalidClone { ty: s() },
            IssueKind::PossiblyInvalidClone { ty: s() },
            IssueKind::InvalidToString { class: s() },
            IssueKind::InvalidTraitUse {
                trait_name: s(),
                reason: s(),
            },
            IssueKind::ParseError { message: s() },
            IssueKind::InvalidThrow { ty: s() },
            IssueKind::InvalidCatch { ty: s() },
            IssueKind::ImplicitToStringCast { class: s() },
            IssueKind::ImplicitFloatToIntCast { from: s() },
            IssueKind::WrongCaseFunction {
                used: s(),
                canonical: s(),
            },
            IssueKind::WrongCaseMethod {
                class: s(),
                used: s(),
                canonical: s(),
            },
            IssueKind::WrongCaseClass {
                used: s(),
                canonical: s(),
            },
            IssueKind::InvalidAttribute { message: s() },
            IssueKind::UndefinedAttributeClass { name: s() },
            IssueKind::ForbiddenCode { message: s() },
            IssueKind::DuplicateClass { name: s() },
            IssueKind::DuplicateInterface { name: s() },
            IssueKind::DuplicateTrait { name: s() },
            IssueKind::DuplicateEnum { name: s() },
            IssueKind::DuplicateFunction { name: s() },
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

    #[test]
    fn default_severity_for_code_round_trips() {
        for kind in one_of_each() {
            let code = kind.code();
            assert_eq!(
                IssueKind::default_severity_for_code(code),
                Some(kind.default_severity()),
                "severity mismatch for {code} (variant {})",
                kind.name(),
            );
        }
    }

    #[test]
    fn default_severity_for_code_unknown_returns_none() {
        assert_eq!(IssueKind::default_severity_for_code("MIR9999"), None);
        assert_eq!(IssueKind::default_severity_for_code(""), None);
        assert_eq!(IssueKind::default_severity_for_code("mir0001"), None);
    }

    /// Guards against forgetting to add a new variant to `one_of_each()`.
    /// If you add a variant, add it to `one_of_each()` *and* bump this count.
    #[test]
    fn one_of_each_has_every_variant() {
        // If this assertion fires after you added a new variant, also add it
        // to `one_of_each()` so the uniqueness and shape tests cover it.
        assert_eq!(one_of_each().len(), 142);
    }
}
