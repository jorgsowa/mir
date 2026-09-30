//! Psalm's per-issue `ERROR_LEVEL` for the issue kinds mir shares with it.
//! Kinds absent from the table (always-error, opt-in feature, or mir-only) are unaffected by the level.

/// Psalm error level (1–8) at which `name` stops being reported as an error.
/// An issue is downgraded to info when the configured level is above this value.
pub fn psalm_error_level(name: &str) -> Option<u8> {
    Some(match name {
        "MixedArgument"
        | "MixedArrayAccess"
        | "MixedArrayOffset"
        | "MixedAssignment"
        | "MixedClone"
        | "MixedFunctionCall"
        | "MixedMethodCall"
        | "MixedPropertyAssignment"
        | "MixedPropertyFetch"
        | "MixedReturnStatement"
        | "PossiblyNullOperand"
        | "Trace" => 1,
        "DeprecatedCall"
        | "DeprecatedClass"
        | "DeprecatedConstant"
        | "DeprecatedInterface"
        | "DeprecatedMethod"
        | "DeprecatedMethodCall"
        | "DeprecatedProperty"
        | "DeprecatedTrait"
        | "DirectConstructorCall"
        | "DocblockTypeContradiction"
        | "InvalidStringClass"
        | "MissingClosureReturnType"
        | "MissingConstructor"
        | "MissingParamType"
        | "MissingPropertyType"
        | "MissingReturnType"
        | "RawObjectIteration"
        | "UnsupportedReferenceUsage" => 2,
        "ArgumentTypeCoercion"
        | "PossiblyInvalidArgument"
        | "PossiblyInvalidArrayAccess"
        | "PossiblyInvalidArrayOffset"
        | "PossiblyInvalidClone"
        | "PossiblyInvalidOperand"
        | "PossiblyNullArgument"
        | "PossiblyNullArrayAccess"
        | "PossiblyNullMethodCall"
        | "PossiblyNullPropertyFetch"
        | "PossiblyUndefinedVariable"
        | "PropertyTypeCoercion" => 3,
        "ForbiddenCode"
        | "IfThisIsMismatch"
        | "ImplicitToStringCast"
        | "InternalMethod"
        | "InvalidDocblock"
        | "InvalidDocblockType"
        | "InvalidOperand"
        | "InvalidToString"
        | "MismatchingDocblockParamType"
        | "MismatchingDocblockReturnType"
        | "NoInterfaceProperties"
        | "RedundantCast"
        | "RedundantCondition"
        | "TooManyArguments"
        | "TypeDoesNotContainType" => 4,
        "NullableReturnStatement" => 5,
        "InvalidArgument"
        | "InvalidArrayAccess"
        | "InvalidArrayAssignment"
        | "InvalidArrayOffset"
        | "InvalidCast"
        | "InvalidCatch"
        | "InvalidClone"
        | "InvalidNamedArgument"
        | "InvalidPropertyAssignment"
        | "InvalidPropertyFetch"
        | "InvalidReturnType"
        | "InvalidTemplateParam"
        | "NullArgument"
        | "UndefinedMethod"
        | "UndefinedProperty" => 6,
        "AbstractInstantiation"
        | "InvalidOverride"
        | "MethodSignatureMismatch"
        | "OverriddenMethodAccess"
        | "UnhandledMatchCondition" => 7,
        _ => return None,
    })
}
