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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub file: Arc<str>,
    pub line: u32,
    /// Last line of the issue range (inclusive, 1-based). Equal to `line` for single-line issues.
    pub line_end: u32,
    /// 0-based Unicode char-count (code-point) column of the issue start.
    pub col_start: u16,
    /// 0-based Unicode char-count (code-point) column of the issue end (exclusive).
    pub col_end: u16,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.col_start)
    }
}

// ---------------------------------------------------------------------------
// IssueKind
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum IssueKind {
    // --- Undefined ----------------------------------------------------------
    InvalidScope {
        /// `true` when inside a class but in a static method; `false` when outside a class.
        in_class: bool,
    },
    UndefinedVariable {
        name: String,
    },
    UndefinedFunction {
        name: String,
    },
    UndefinedMethod {
        class: String,
        method: String,
    },
    UndefinedClass {
        name: String,
    },
    UndefinedProperty {
        class: String,
        property: String,
    },
    UndefinedConstant {
        name: String,
    },
    PossiblyUndefinedVariable {
        name: String,
    },

    // --- Nullability --------------------------------------------------------
    NullArgument {
        param: String,
        fn_name: String,
    },
    NullPropertyFetch {
        property: String,
    },
    NullMethodCall {
        method: String,
    },
    NullArrayAccess,
    PossiblyNullArgument {
        param: String,
        fn_name: String,
    },
    PossiblyNullPropertyFetch {
        property: String,
    },
    PossiblyNullMethodCall {
        method: String,
    },
    PossiblyNullArrayAccess,
    NullableReturnStatement {
        expected: String,
        actual: String,
    },

    // --- Type mismatches ----------------------------------------------------
    InvalidReturnType {
        expected: String,
        actual: String,
    },
    InvalidArgument {
        param: String,
        fn_name: String,
        expected: String,
        actual: String,
    },
    TooFewArguments {
        fn_name: String,
        expected: usize,
        actual: usize,
    },
    TooManyArguments {
        fn_name: String,
        expected: usize,
        actual: usize,
    },
    InvalidNamedArgument {
        fn_name: String,
        name: String,
    },
    InvalidPassByReference {
        fn_name: String,
        param: String,
    },
    InvalidPropertyAssignment {
        property: String,
        expected: String,
        actual: String,
    },
    InvalidCast {
        from: String,
        to: String,
    },
    InvalidOperand {
        op: String,
        left: String,
        right: String,
    },
    MismatchingDocblockReturnType {
        declared: String,
        inferred: String,
    },
    MismatchingDocblockParamType {
        param: String,
        declared: String,
        inferred: String,
    },

    // --- Array issues -------------------------------------------------------
    InvalidArrayOffset {
        expected: String,
        actual: String,
    },
    NonExistentArrayOffset {
        key: String,
    },
    PossiblyInvalidArrayOffset {
        expected: String,
        actual: String,
    },

    // --- Redundancy ---------------------------------------------------------
    RedundantCondition {
        ty: String,
    },
    RedundantCast {
        from: String,
        to: String,
    },
    UnnecessaryVarAnnotation {
        var: String,
    },
    TypeDoesNotContainType {
        left: String,
        right: String,
    },

    // --- Dead code ----------------------------------------------------------
    UnusedVariable {
        name: String,
    },
    UnusedParam {
        name: String,
    },
    UnreachableCode,
    UnusedMethod {
        class: String,
        method: String,
    },
    UnusedProperty {
        class: String,
        property: String,
    },
    UnusedFunction {
        name: String,
    },

    // --- Readonly -----------------------------------------------------------
    ReadonlyPropertyAssignment {
        class: String,
        property: String,
    },

    // --- Inheritance --------------------------------------------------------
    UnimplementedAbstractMethod {
        class: String,
        method: String,
    },
    UnimplementedInterfaceMethod {
        class: String,
        interface: String,
        method: String,
    },
    MethodSignatureMismatch {
        class: String,
        method: String,
        detail: String,
    },
    OverriddenMethodAccess {
        class: String,
        method: String,
    },
    FinalClassExtended {
        parent: String,
        child: String,
    },
    FinalMethodOverridden {
        class: String,
        method: String,
        parent: String,
    },

    // --- Security (taint) ---------------------------------------------------
    TaintedInput {
        sink: String,
    },
    TaintedHtml,
    TaintedSql,
    TaintedShell,

    // --- Generics -----------------------------------------------------------
    InvalidTemplateParam {
        name: String,
        expected_bound: String,
        actual: String,
    },
    ShadowedTemplateParam {
        name: String,
    },

    // --- Other --------------------------------------------------------------
    DeprecatedCall {
        name: String,
        message: Option<Arc<str>>,
    },
    DeprecatedMethodCall {
        class: String,
        method: String,
        message: Option<Arc<str>>,
    },
    DeprecatedMethod {
        class: String,
        method: String,
        message: Option<Arc<str>>,
    },
    DeprecatedClass {
        name: String,
        message: Option<Arc<str>>,
    },
    InternalMethod {
        class: String,
        method: String,
    },
    MissingReturnType {
        fn_name: String,
    },
    MissingParamType {
        fn_name: String,
        param: String,
    },
    InvalidThrow {
        ty: String,
    },
    MissingThrowsDocblock {
        class: String,
    },
    ParseError {
        message: String,
    },
    InvalidDocblock {
        message: String,
    },
    MixedArgument {
        param: String,
        fn_name: String,
    },
    MixedAssignment {
        var: String,
    },
    MixedMethodCall {
        method: String,
    },
    MixedPropertyFetch {
        property: String,
    },
    CircularInheritance {
        class: String,
    },

    // --- Trait constraints --------------------------------------------------
    InvalidTraitUse {
        trait_name: String,
        reason: String,
    },
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
            | IssueKind::InvalidTemplateParam { .. }
            | IssueKind::ReadonlyPropertyAssignment { .. }
            | IssueKind::ParseError { .. }
            | IssueKind::TaintedInput { .. }
            | IssueKind::TaintedHtml
            | IssueKind::TaintedSql
            | IssueKind::TaintedShell
            | IssueKind::CircularInheritance { .. }
            | IssueKind::InvalidTraitUse { .. } => Severity::Error,

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
            | IssueKind::MissingThrowsDocblock { .. }
            | IssueKind::UnusedVariable { .. } => Severity::Warning,

            // PossiblyUndefined: shown at default error level (same as Warning)
            IssueKind::PossiblyUndefinedVariable { .. } => Severity::Warning,

            // Possibly-null (only shown in strict mode, level ≥ 7)
            IssueKind::PossiblyNullArgument { .. }
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
            | IssueKind::ShadowedTemplateParam { .. } => Severity::Info,
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
            IssueKind::NullArgument { .. } => "NullArgument",
            IssueKind::NullPropertyFetch { .. } => "NullPropertyFetch",
            IssueKind::NullMethodCall { .. } => "NullMethodCall",
            IssueKind::NullArrayAccess => "NullArrayAccess",
            IssueKind::PossiblyNullArgument { .. } => "PossiblyNullArgument",
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
            IssueKind::ParseError { .. } => "ParseError",
            IssueKind::InvalidDocblock { .. } => "InvalidDocblock",
            IssueKind::MixedArgument { .. } => "MixedArgument",
            IssueKind::MixedAssignment { .. } => "MixedAssignment",
            IssueKind::MixedMethodCall { .. } => "MixedMethodCall",
            IssueKind::MixedPropertyFetch { .. } => "MixedPropertyFetch",
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            "{} {} {}: {}",
            self.location.bright_black(),
            sev,
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
