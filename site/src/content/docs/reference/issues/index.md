---
title: Issue Kinds
description: All issue types mir can emit, grouped by category.
sidebar:
  order: 0
---

All issue types mir can emit. Select an issue for details.

| Category | Issues |
|----------|--------|
| [Undefined](./undefined/undefined-variable/) | UndefinedVariable, PossiblyUndefinedVariable, UndefinedFunction, UndefinedMethod, UndefinedClass, UndefinedProperty, UndefinedConstant |
| [Nullability](./nullability/null-argument/) | NullArgument, NullPropertyFetch, NullMethodCall, NullArrayAccess, PossiblyNull\*, NullableReturnStatement |
| [Type Mismatches](./type-mismatches/invalid-return-type/) | InvalidReturnType, InvalidArgument, TooFewArguments, TooManyArguments, InvalidNamedArgument, InvalidPassByReference, InvalidPropertyAssignment, InvalidCast, InvalidOperand, MismatchingDocblockReturnType |
| [Array](./array/invalid-array-offset/) | InvalidArrayOffset, NonExistentArrayOffset, PossiblyInvalidArrayOffset, PossiblyInvalidArrayAccess |
| [Redundancy](./redundancy/redundant-condition/) | RedundantCondition, RedundantCast, UnnecessaryVarAnnotation, TypeDoesNotContainType |
| [Dead Code](./dead-code/unused-variable/) | UnusedVariable, UnusedParam, UnusedMethod, UnusedProperty, UnusedFunction, UnreachableCode |
| [Inheritance](./inheritance/unimplemented-abstract-method/) | UnimplementedAbstractMethod, UnimplementedInterfaceMethod, MethodSignatureMismatch, OverriddenMethodAccess, FinalClassExtended, FinalMethodOverridden |
| [Security](./security/tainted-html/) | TaintedHtml, TaintedSql, TaintedShell |
| [Generics](./generics/invalid-template-param/) | InvalidTemplateParam |
| [Other](./other/deprecated-method/) | DeprecatedMethod, DeprecatedClass, InternalMethod, InvalidThrow, MissingThrowsDocblock, ReadonlyPropertyAssignment, ParseError, InvalidDocblock |
