---
title: Issue Kinds
description: All issue types mir can emit, grouped by category.
---

All issue types mir can emit, grouped by category.

## Undefined

| Kind | Description |
|------|-------------|
| `InvalidScope` | Use of `$this` outside a class or inside a static method |
| `UndefinedVariable` | Variable used before assignment |
| `PossiblyUndefinedVariable` | Variable only assigned in some branches |
| `UndefinedFunction` | Call to a function not found in codebase or stubs |
| `UndefinedMethod` | Call to a method not found on the type |
| `UndefinedClass` | Reference to a class or interface that doesn't exist |
| `UndefinedTrait` | Reference to a trait that doesn't exist |
| `UndefinedProperty` | Access to a property not declared on the class |
| `UndefinedConstant` | Reference to an undefined constant |

## Nullability

| Kind | Description |
|------|-------------|
| `NullArgument` | Passing `null` to a non-nullable parameter |
| `NullPropertyFetch` | Property access on a `null` value |
| `NullMethodCall` | Method call on a `null` value |
| `NullArrayAccess` | Array access on a `null` value |
| `PossiblyNullArgument` | Argument may be `null` where non-nullable is expected |
| `PossiblyNullPropertyFetch` | Property access on a possibly-null value |
| `PossiblyNullMethodCall` | Method call on a possibly-null value |
| `PossiblyNullArrayAccess` | Array access on a possibly-null value |
| `PossiblyInvalidArgument` | Argument may be an incompatible type |
| `NullableReturnStatement` | Returning `null` from a non-nullable return type |

## Type mismatches

| Kind | Description |
|------|-------------|
| `InvalidReturnType` | Return value doesn't match declared return type |
| `InvalidArgument` | Argument type doesn't match parameter type |
| `TooFewArguments` | Call provides fewer arguments than required |
| `TooManyArguments` | Call provides more arguments than accepted |
| `InvalidNamedArgument` | Named argument does not match a callable parameter |
| `InvalidPassByReference` | By-reference parameter receives a non-referenceable expression |
| `InvalidPropertyAssignment` | Assigned value incompatible with property type |
| `InvalidCast` | Cast from an array or object to a scalar type that always produces a meaningless result |
| `InvalidOperand` | Operator applied to incompatible types |
| `MismatchingDocblockReturnType` | Docblock return type conflicts with native type hint |
| `MismatchingDocblockParamType` | Docblock param type conflicts with native type hint |

## Array

| Kind | Description |
|------|-------------|
| `InvalidArrayOffset` | Array accessed with a key of the wrong type |
| `NonExistentArrayOffset` | Array accessed with a key known not to exist |
| `PossiblyInvalidArrayOffset` | Array accessed with a key that might be the wrong type or might not exist |

## Redundancy

| Kind | Description |
|------|-------------|
| `RedundantCondition` | Condition that is always true or always false |
| `RedundantCast` | Cast to a type the value already has |
| `UnnecessaryVarAnnotation` | `@var` annotation that matches the inferred type |
| `TypeDoesNotContainType` | Type check that can never be true |

## Dead code

| Kind | Description |
|------|-------------|
| `UnusedVariable` | Variable assigned but never read |
| `UnusedParam` | Parameter never used in the function body |
| `UnusedMethod` | Private method never called |
| `UnusedProperty` | Private property never read |
| `UnusedFunction` | Function never called |
| `UnreachableCode` | Code after an unconditional `return`/`throw`/`exit` |

## Inheritance

| Kind | Description |
|------|-------------|
| `UnimplementedAbstractMethod` | Abstract method not implemented in concrete class |
| `UnimplementedInterfaceMethod` | Interface method not implemented |
| `MethodSignatureMismatch` | Override has incompatible signature |
| `OverriddenMethodAccess` | Override reduces visibility |
| `FinalClassExtended` | Extending a `final` class |
| `FinalMethodOverridden` | Overriding a `final` method |
| `AbstractInstantiation` | Instantiating an abstract class with `new` |
| `CircularInheritance` | Class or interface inherits from itself |

## Security (taint)

| Kind | Description |
|------|-------------|
| `TaintedInput` | Tainted input reaching a sink |
| `TaintedHtml` | Tainted input reaches an HTML sink without sanitization |
| `TaintedSql` | Tainted input reaches a SQL sink without sanitization |
| `TaintedShell` | Tainted input reaches a shell sink without sanitization |

## Generics

| Kind | Description |
|------|-------------|
| `InvalidTemplateParam` | Template argument violates declared bounds |
| `ShadowedTemplateParam` | `@template` declaration shadows a template param from an outer scope |

## Other

| Kind | Description |
|------|-------------|
| `DeprecatedCall` | Call to a `@deprecated` function |
| `DeprecatedMethodCall` | Call to a `@deprecated` method |
| `DeprecatedMethod` | Method is marked `@deprecated` |
| `DeprecatedClass` | Instantiation of a `@deprecated` class |
| `InternalMethod` | Call to an `@internal` method from outside its package |
| `InvalidThrow` | `throw` of a non-`Throwable` value |
| `MissingThrowsDocblock` | Thrown exception not declared in `@throws` |
| `ImplicitToStringCast` | Object implicitly cast to `string` without a `__toString` method or `Stringable` interface |
| `ImplicitFloatToIntCast` | `float` implicitly narrowed to `int` with possible data loss |
| `MissingReturnType` | Function or method has no declared return type |
| `MissingParamType` | Parameter has no declared type |
| `ReadonlyPropertyAssignment` | Assignment to a `readonly` property after construction |
| `InvalidTraitUse` | Incompatible or conflicting trait use |
| `ParseError` | File could not be parsed |
| `InvalidDocblock` | Malformed or unrecognisable docblock annotation |

## Mixed

| Kind | Description |
|------|-------------|
| `MixedArgument` | Argument has type `mixed` where a typed value is expected |
| `MixedAssignment` | Assigning a `mixed` value to a typed variable |
| `MixedMethodCall` | Method call on a `mixed` value |
| `MixedPropertyFetch` | Property access on a `mixed` value |
| `MixedClone` | Cloning a `mixed` value |
