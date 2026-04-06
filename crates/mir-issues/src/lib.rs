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
    pub col_start: u16,
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

    // --- Other --------------------------------------------------------------
    DeprecatedCall {
        name: String,
    },
    DeprecatedMethodCall {
        class: String,
        method: String,
    },
    DeprecatedMethod {
        class: String,
        method: String,
    },
    DeprecatedClass {
        name: String,
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
}

impl IssueKind {
    /// Default severity for this issue kind.
    pub fn default_severity(&self) -> Severity {
        match self {
            // Errors (always blocking)
            IssueKind::UndefinedVariable { .. }
            | IssueKind::UndefinedFunction { .. }
            | IssueKind::UndefinedMethod { .. }
            | IssueKind::UndefinedClass { .. }
            | IssueKind::UndefinedConstant { .. }
            | IssueKind::InvalidReturnType { .. }
            | IssueKind::InvalidArgument { .. }
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
            | IssueKind::TaintedShell => Severity::Error,

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

            // Possibly-null / possibly-undefined (only shown in strict mode, level ≥ 7)
            IssueKind::PossiblyUndefinedVariable { .. }
            | IssueKind::PossiblyNullArgument { .. }
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
            | IssueKind::MixedPropertyFetch { .. } => Severity::Info,
        }
    }

    /// Identifier name used in config and `@psalm-suppress` / `@suppress` annotations.
    pub fn name(&self) -> &'static str {
        match self {
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
        }
    }

    /// Human-readable message for this issue.
    pub fn message(&self) -> String {
        match self {
            IssueKind::UndefinedVariable { name } => format!("Variable ${} is not defined", name),
            IssueKind::UndefinedFunction { name } => format!("Function {}() is not defined", name),
            IssueKind::UndefinedMethod { class, method } => {
                format!("Method {}::{}() does not exist", class, method)
            }
            IssueKind::UndefinedClass { name } => format!("Class {} does not exist", name),
            IssueKind::UndefinedProperty { class, property } => {
                format!("Property {}::${} does not exist", class, property)
            }
            IssueKind::UndefinedConstant { name } => format!("Constant {} is not defined", name),
            IssueKind::PossiblyUndefinedVariable { name } => {
                format!("Variable ${} might not be defined", name)
            }

            IssueKind::NullArgument { param, fn_name } => {
                format!("Argument ${} of {}() cannot be null", param, fn_name)
            }
            IssueKind::NullPropertyFetch { property } => {
                format!("Cannot access property ${} on null", property)
            }
            IssueKind::NullMethodCall { method } => {
                format!("Cannot call method {}() on null", method)
            }
            IssueKind::NullArrayAccess => "Cannot access array on null".to_string(),
            IssueKind::PossiblyNullArgument { param, fn_name } => {
                format!("Argument ${} of {}() might be null", param, fn_name)
            }
            IssueKind::PossiblyNullPropertyFetch { property } => {
                format!(
                    "Cannot access property ${} on possibly null value",
                    property
                )
            }
            IssueKind::PossiblyNullMethodCall { method } => {
                format!("Cannot call method {}() on possibly null value", method)
            }
            IssueKind::PossiblyNullArrayAccess => {
                "Cannot access array on possibly null value".to_string()
            }
            IssueKind::NullableReturnStatement { expected, actual } => {
                format!(
                    "Return type '{}' is not compatible with declared '{}'",
                    actual, expected
                )
            }

            IssueKind::InvalidReturnType { expected, actual } => {
                format!(
                    "Return type '{}' is not compatible with declared '{}'",
                    actual, expected
                )
            }
            IssueKind::InvalidArgument {
                param,
                fn_name,
                expected,
                actual,
            } => {
                format!(
                    "Argument ${} of {}() expects '{}', got '{}'",
                    param, fn_name, expected, actual
                )
            }
            IssueKind::InvalidPropertyAssignment {
                property,
                expected,
                actual,
            } => {
                format!(
                    "Property ${} expects '{}', cannot assign '{}'",
                    property, expected, actual
                )
            }
            IssueKind::InvalidCast { from, to } => {
                format!("Cannot cast '{}' to '{}'", from, to)
            }
            IssueKind::InvalidOperand { op, left, right } => {
                format!(
                    "Operator '{}' not supported between '{}' and '{}'",
                    op, left, right
                )
            }
            IssueKind::MismatchingDocblockReturnType { declared, inferred } => {
                format!(
                    "Docblock return type '{}' does not match inferred '{}'",
                    declared, inferred
                )
            }
            IssueKind::MismatchingDocblockParamType {
                param,
                declared,
                inferred,
            } => {
                format!(
                    "Docblock type '{}' for ${} does not match inferred '{}'",
                    declared, param, inferred
                )
            }

            IssueKind::InvalidArrayOffset { expected, actual } => {
                format!("Array offset expects '{}', got '{}'", expected, actual)
            }
            IssueKind::NonExistentArrayOffset { key } => {
                format!("Array offset '{}' does not exist", key)
            }
            IssueKind::PossiblyInvalidArrayOffset { expected, actual } => {
                format!(
                    "Array offset might be invalid: expects '{}', got '{}'",
                    expected, actual
                )
            }

            IssueKind::RedundantCondition { ty } => {
                format!("Condition is always true/false for type '{}'", ty)
            }
            IssueKind::RedundantCast { from, to } => {
                format!("Casting '{}' to '{}' is redundant", from, to)
            }
            IssueKind::UnnecessaryVarAnnotation { var } => {
                format!("@var annotation for ${} is unnecessary", var)
            }
            IssueKind::TypeDoesNotContainType { left, right } => {
                format!("Type '{}' can never contain type '{}'", left, right)
            }

            IssueKind::UnusedVariable { name } => format!("Variable ${} is never read", name),
            IssueKind::UnusedParam { name } => format!("Parameter ${} is never used", name),
            IssueKind::UnreachableCode => "Unreachable code detected".to_string(),
            IssueKind::UnusedMethod { class, method } => {
                format!("Private method {}::{}() is never called", class, method)
            }
            IssueKind::UnusedProperty { class, property } => {
                format!("Private property {}::${} is never read", class, property)
            }
            IssueKind::UnusedFunction { name } => {
                format!("Function {}() is never called", name)
            }

            IssueKind::UnimplementedAbstractMethod { class, method } => {
                format!(
                    "Class {} must implement abstract method {}()",
                    class, method
                )
            }
            IssueKind::UnimplementedInterfaceMethod {
                class,
                interface,
                method,
            } => {
                format!(
                    "Class {} must implement {}::{}() from interface",
                    class, interface, method
                )
            }
            IssueKind::MethodSignatureMismatch {
                class,
                method,
                detail,
            } => {
                format!(
                    "Method {}::{}() signature mismatch: {}",
                    class, method, detail
                )
            }
            IssueKind::OverriddenMethodAccess { class, method } => {
                format!(
                    "Method {}::{}() overrides with less visibility",
                    class, method
                )
            }
            IssueKind::ReadonlyPropertyAssignment { class, property } => {
                format!(
                    "Cannot assign to readonly property {}::${} outside of constructor",
                    class, property
                )
            }
            IssueKind::FinalClassExtended { parent, child } => {
                format!("Class {} cannot extend final class {}", child, parent)
            }
            IssueKind::InvalidTemplateParam {
                name,
                expected_bound,
                actual,
            } => {
                format!(
                    "Template type '{}' inferred as '{}' does not satisfy bound '{}'",
                    name, actual, expected_bound
                )
            }
            IssueKind::FinalMethodOverridden {
                class,
                method,
                parent,
            } => {
                format!(
                    "Method {}::{}() cannot override final method from {}",
                    class, method, parent
                )
            }

            IssueKind::TaintedInput { sink } => format!("Tainted input reaching sink '{}'", sink),
            IssueKind::TaintedHtml => "Tainted HTML output — possible XSS".to_string(),
            IssueKind::TaintedSql => "Tainted SQL query — possible SQL injection".to_string(),
            IssueKind::TaintedShell => {
                "Tainted shell command — possible command injection".to_string()
            }

            IssueKind::DeprecatedCall { name } => {
                format!("Call to deprecated function {}", name)
            }
            IssueKind::DeprecatedMethodCall { class, method } => {
                format!("Call to deprecated method {}::{}", class, method)
            }
            IssueKind::DeprecatedMethod { class, method } => {
                format!("Method {}::{}() is deprecated", class, method)
            }
            IssueKind::DeprecatedClass { name } => format!("Class {} is deprecated", name),
            IssueKind::InternalMethod { class, method } => {
                format!("Method {}::{}() is marked @internal", class, method)
            }
            IssueKind::MissingReturnType { fn_name } => {
                format!("Function {}() has no return type annotation", fn_name)
            }
            IssueKind::MissingParamType { fn_name, param } => {
                format!(
                    "Parameter ${} of {}() has no type annotation",
                    param, fn_name
                )
            }
            IssueKind::InvalidThrow { ty } => {
                format!("Thrown type '{}' does not extend Throwable", ty)
            }
            IssueKind::MissingThrowsDocblock { class } => {
                format!("Exception {} is thrown but not declared in @throws", class)
            }
            IssueKind::ParseError { message } => format!("Parse error: {}", message),
            IssueKind::InvalidDocblock { message } => format!("Invalid docblock: {}", message),
            IssueKind::MixedArgument { param, fn_name } => {
                format!("Argument ${} of {}() is mixed", param, fn_name)
            }
            IssueKind::MixedAssignment { var } => {
                format!("Variable ${} is assigned a mixed type", var)
            }
            IssueKind::MixedMethodCall { method } => {
                format!("Method {}() called on mixed type", method)
            }
            IssueKind::MixedPropertyFetch { property } => {
                format!("Property ${} fetched on mixed type", property)
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
    /// Issue names suppressed at the file level (from `@psalm-suppress` / `@suppress` on the file docblock)
    file_suppressions: Vec<String>,
}

impl IssueBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, issue: Issue) {
        // Deduplicate: skip if the same issue (kind + location) was already added.
        if self.issues.iter().any(|existing| {
            existing.kind.name() == issue.kind.name()
                && existing.location.file == issue.location.file
                && existing.location.line == issue.location.line
                && existing.location.col_start == issue.location.col_start
        }) {
            return;
        }
        self.issues.push(issue);
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
