# Issue Kinds

All issue types mir can emit, grouped by category.

## Undefined

| Kind | Description |
|------|-------------|
| `UndefinedVariable` | Variable used before assignment |
| `PossiblyUndefinedVariable` | Variable only assigned in some branches |
| `UndefinedFunction` | Call to a function not found in codebase or stubs |
| `UndefinedMethod` | Call to a method not found on the type |
| `UndefinedClass` | Reference to a class/interface/trait that doesn't exist |
| `UndefinedProperty` | Access to a property not declared on the class |
| `UndefinedConstant` | Reference to an undefined constant |

## Nullability

| Kind | Description |
|------|-------------|
| `NullArgument` | Passing `null` to a non-nullable parameter |
| `NullPropertyFetch` | Property access on a possibly-null value |
| `NullMethodCall` | Method call on a possibly-null value |
| `NullArrayAccess` | Array access on a possibly-null value |
| `PossiblyNull*` | Nullable variants of the above |
| `NullableReturnStatement` | Returning `null` from a non-nullable return type |

## Type mismatches

| Kind | Description |
|------|-------------|
| `InvalidReturnType` | Return value doesn't match declared return type |
| `InvalidArgument` | Argument type doesn't match parameter type |
| `InvalidPropertyAssignment` | Assigned value incompatible with property type |
| `InvalidCast` | Explicit cast that can never succeed |
| `InvalidOperand` | Operator applied to incompatible types |
| `MismatchingDocblockReturnType` | Docblock return type conflicts with native type hint |

## Array

| Kind | Description |
|------|-------------|
| `InvalidArrayOffset` | Array accessed with a key of the wrong type |
| `NonExistentArrayOffset` | Array accessed with a key known not to exist |
| `PossiblyInvalidArrayOffset` | Array accessed with a key that might not exist |
| `PossiblyInvalidArrayAccess` | Array access on a value that might not be an array |

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

## Security (taint)

| Kind | Description |
|------|-------------|
| `TaintedHtml` | User input reaches an HTML sink without sanitization |
| `TaintedSql` | User input reaches a SQL sink without sanitization |
| `TaintedShell` | User input reaches a shell sink without sanitization |

## Generics

| Kind | Description |
|------|-------------|
| `InvalidTemplateParam` | Template argument violates declared bounds |

## Other

| Kind | Description |
|------|-------------|
| `DeprecatedMethod` | Call to a `@deprecated` method |
| `DeprecatedClass` | Instantiation of a `@deprecated` class |
| `InternalMethod` | Call to an `@internal` method from outside its package |
| `InvalidThrow` | `throw` of a non-`Throwable` value |
| `MissingThrowsDocblock` | Thrown exception not declared in `@throws` |
| `ReadonlyPropertyAssignment` | Assignment to a `readonly` property after construction |
| `ParseError` | File could not be parsed |
| `InvalidDocblock` | Malformed or unrecognisable docblock annotation |
