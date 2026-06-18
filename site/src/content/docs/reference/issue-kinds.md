---
title: Issue Kinds
description: All issue types mir can emit, grouped by category.
---

All issue types mir can emit, grouped by category.

## Undefined

| Kind | Description |
|------|-------------|
| `InvalidScope` | `$this` used outside a class or inside a static method. |
| `UndefinedVariable` | A variable is used before it has been assigned. |
| `UndefinedFunction` | A call targets a function that does not exist in the codebase or stubs. |
| `UndefinedMethod` | A method is called on a type that does not declare that method. |
| `UndefinedClass` | A reference is made to a class or interface that does not exist. |
| `UndefinedProperty` | A property is accessed that is not declared on the class. |
| `UndefinedConstant` | A reference is made to a constant that has not been defined. |
| `PossiblyUndefinedVariable` | A variable is only assigned in some branches and may be unset on other paths. |
| `UndefinedTrait` | A trait is used that does not exist in the codebase or stubs. |
| `ParentNotFound` | Use of `parent::` in a class that has no parent. |
| `InaccessibleClassConstant` | Access to a private or protected class constant from an incompatible scope. |

## Nullability

| Kind | Description |
|------|-------------|
| `NullArgument` | null is passed to a parameter that does not accept null. |
| `NullPropertyFetch` | A property is accessed on a value that is possibly null. |
| `NullMethodCall` | A method is called on a value that is possibly null. |
| `NullArrayAccess` | Array access is performed on a value that is possibly null. |
| `PossiblyNullArgument` | An argument is possibly `null` but the parameter type doesn't admit `null`. |
| `PossiblyInvalidArgument` | An argument may not match the parameter type on some code paths. |
| `PossiblyNullPropertyFetch` | A property is fetched from a value that may be `null`. |
| `PossiblyNullMethodCall` | A method is called on a value that may be `null`. |
| `PossiblyNullArrayAccess` | Array access on a value that may be `null`. |
| `NullableReturnStatement` | A nullable value is returned from a function with a non-nullable return type. |

## Type mismatches

| Kind | Description |
|------|-------------|
| `InvalidReturnType` | The returned value does not match the declared return type. |
| `InvalidArgument` | An argument's type does not match the parameter's declared type. |
| `TooFewArguments` | A call provides fewer arguments than the function requires. |
| `TooManyArguments` | A call provides more arguments than the function accepts. |
| `InvalidNamedArgument` | A named argument does not correspond to any parameter of the callable. |
| `InvalidPassByReference` | A by-reference parameter receives an expression that cannot be referenced. |
| `InvalidPropertyAssignment` | A value assigned to a property is incompatible with the property's declared type. |
| `InvalidCast` | A cast from an array or object to a scalar type always produces a meaningless result. |
| `InvalidOperand` | An operator is applied to incompatible types. |
| `MismatchingDocblockReturnType` | The @return docblock type conflicts with the native return type hint. |
| `MismatchingDocblockParamType` | A docblock `@param` type doesn't match the inferred type from the function body. |
| `InvalidStringClass` | A string used as a class name does not resolve to a known class. |
| `TypeCheckMismatch` | A `@mir-check` assertion in a test fixture does not match the inferred type. |
| `PossiblyInvalidOperand` | Operator applied to a union type with some incompatible members. |
| `PossiblyNullOperand` | Operand could be `null`, making the operation potentially unsafe. |
| `InvalidStaticInvocation` | Non-static method called with `::` syntax. |
| `NonStaticSelfCall` | `self::method()` called when the method is not static. |
| `DirectConstructorCall` | Direct call to `__construct()` outside of a constructor chain. |
| `InvalidPropertyFetch` | Property access on a non-object type. |
| `InvalidArrayAccess` | Array-style access on a non-array, non-string, non-`ArrayAccess` type. |
| `InvalidArrayAssignment` | Array-style assignment on a non-array type. |
| `Trace` | Internal/debug output of the `@trace` annotation, showing the inferred type of an expression. Not a project diagnostic. |
| `RawObjectIteration` | An object that does not implement `Traversable` is iterated (e.g. via `foreach` or `yield from`). |
| `PossiblyRawObjectIteration` | A value that *might* be a non-iterable object is iterated. |
| `InvalidNamedArguments` | Named arguments passed to a function or method tagged `@no-named-arguments`. |
| `ArgumentTypeCoercion` | An argument's type had to be widened/coerced to match the parameter's declared type. |
| `PropertyTypeCoercion` | A value assigned to a property had to be coerced to the property's declared type. |
| `PossiblyInvalidArrayAccess` | Array access on a value that is only sometimes offset-accessible. |

## Array / offset

| Kind | Description |
|------|-------------|
| `InvalidArrayOffset` | An array is accessed with a key of the wrong type. |
| `NonExistentArrayOffset` | An array is accessed with a key that is known not to exist. |
| `PossiblyInvalidArrayOffset` | An array is accessed with a key that might not exist. |

## Redundancy

| Kind | Description |
|------|-------------|
| `RedundantCondition` | A condition is always true or always false based on the known types. |
| `RedundantCast` | A value is cast to a type it already has. |
| `UnnecessaryVarAnnotation` | A @var annotation matches the type that mir already infers. |
| `TypeDoesNotContainType` | A type check can never be true because the type does not include the tested type. |
| `ParadoxicalCondition` | Condition that directly contradicts itself (always false). |
| `UnhandledMatchCondition` | A `match` has no arm for some possible subject value and no `default` arm. |
| `DocblockTypeContradiction` | A docblock-declared type makes a later assertion or comparison impossible (e.g. `assert($a < 4)` on `@param int<5, max> $a`). |
| `UnevaluatedCode` | A `switch`/`match` arm can never be reached given the subject's inferred type (e.g. `case "int"` for `gettype()`, which returns `"integer"`). |

## Dead code

| Kind | Description |
|------|-------------|
| `UnusedVariable` | A variable is assigned but never read. |
| `UnusedParam` | A function parameter is never referenced in the function body. |
| `UnreachableCode` | Code appears after an unconditional return, throw, or exit. |
| `UnusedMethod` | A private method is never called within the class. |
| `UnusedProperty` | A private property is never read within the class. |
| `UnusedFunction` | A function is defined but never called. |
| `UnusedForeachValue` | Foreach value variable assigned but never read. |
| `UnusedClass` | A class is declared but never referenced anywhere in the analyzed code. |
| `UnusedSuppress` | A suppression annotation (`@psalm-suppress` / `@mir-suppress` / `@suppress`) did not match any issue. |

## Readonly

| Kind | Description |
|------|-------------|
| `ReadonlyPropertyAssignment` | A readonly property is assigned after the constructor. |

## Inheritance

| Kind | Description |
|------|-------------|
| `UnimplementedAbstractMethod` | A concrete class does not implement an abstract method from its parent. |
| `UnimplementedInterfaceMethod` | A class does not implement a method required by an interface it declares. |
| `MethodSignatureMismatch` | An overriding method has a signature incompatible with the parent's. |
| `OverriddenMethodAccess` | An overriding method reduces the visibility of the parent method. |
| `InvalidExtendClass` | A class extends a `final` class or a class annotated `@final`. |
| `FinalMethodOverridden` | A subclass overrides a method declared as final in the parent. |
| `AbstractInstantiation` | An abstract class is being instantiated with `new`. |
| `CircularInheritance` | A class participates in a circular `extends`/`implements`/`use` chain. |
| `InvalidOverride` | Method declared `#[Override]` does not actually override a parent method. |
| `InterfaceInstantiation` | Instantiating an interface with `new`. |
| `OverriddenPropertyAccess` | An overriding property reduces the visibility of the parent property. |
| `AbstractMethodCall` | An abstract method is invoked where no concrete implementation exists. |

## Security (taint)

| Kind | Description |
|------|-------------|
| `TaintedInput` | Tainted user input flows to a sensitive sink (generic taint sink). |
| `TaintedHtml` | User-controlled input reaches an HTML output sink without sanitization. |
| `TaintedSql` | User-controlled input reaches a SQL sink without parameterization. |
| `TaintedShell` | User-controlled input reaches a shell execution sink without escaping. |
| `TaintedLlmPrompt` | Tainted input reaches a `@taint-sink llm_prompt` parameter without sanitization. |

## Generics

| Kind | Description |
|------|-------------|
| `InvalidTemplateParam` | A template argument violates the declared bounds of the type parameter. |
| `ShadowedTemplateParam` | A template parameter shadows a name from an outer scope (class template hidden by method template). |
| `IfThisIsMismatch` | A method annotated `@if-this-is X<Y>` was called on a receiver whose type does not satisfy that constraint. |

## Deprecation / internal

| Kind | Description |
|------|-------------|
| `DeprecatedCall` | A function marked `@deprecated` is called. |
| `DeprecatedMethodCall` | A method marked `@deprecated` is called on an instance. |
| `DeprecatedMethod` | A method declaration is itself marked @deprecated (reported on the method definition, not its call sites). |
| `DeprecatedClass` | A class marked @deprecated is being instantiated. |
| `InternalMethod` | A method marked @internal is called from outside its package. |
| `DeprecatedProperty` | Access to a `@deprecated` property. |
| `DeprecatedInterface` | Implementing a `@deprecated` interface. |
| `DeprecatedTrait` | Using a `@deprecated` trait. |
| `DeprecatedConstant` | Reference to a `@deprecated` constant. |
| `WrongCaseFunction` | Function name casing does not match its declaration. |
| `WrongCaseMethod` | Method name casing does not match its declaration. |
| `WrongCaseClass` | Class, interface, or enum name casing does not match its declaration. |

## Missing types / docblocks

| Kind | Description |
|------|-------------|
| `MissingReturnType` | A function or method has no declared return type hint. |
| `MissingParamType` | A parameter has no declared type hint. |
| `MissingThrowsDocblock` | A function throws an exception that is not declared in its @throws docblock. |
| `InvalidDocblock` | A docblock contains a malformed or unrecognised annotation. |
| `MissingPropertyType` | A class property has no declared type. |
| `MissingClosureReturnType` | A closure or arrow function has no declared return type. |

## Mixed

| Kind | Description |
|------|-------------|
| `MixedArgument` | An argument with type `mixed` is passed to a parameter with a more specific type. |
| `MixedAssignment` | A value with type `mixed` is assigned, hiding its concrete type. |
| `MixedMethodCall` | A method is called on a value of type `mixed`. |
| `MixedPropertyFetch` | A property is fetched from a value of type `mixed`. |
| `MixedClone` | `clone` is applied to a value of type `mixed`. |
| `InvalidClone` | Cloning a non-object type. |
| `PossiblyInvalidClone` | Cloning a value whose type might not be an object. |
| `InvalidToString` | Using an object in a string context without a `__toString` method. |
| `MixedPropertyAssignment` | A `mixed` value is assigned to a property, hiding its concrete type. |
| `MixedArrayAccess` | Array access produced a value of type `mixed`. |
| `MixedArrayOffset` | An array is indexed with an offset of type `mixed`. |
| `MixedFunctionCall` | A dynamic call target has type `mixed`. |
| `MixedReturnStatement` | A `mixed` value is returned from a function with a typed return. |

## Trait

| Kind | Description |
|------|-------------|
| `InvalidTraitUse` | A trait is used in a way that violates its declared constraints. |
| `ForbiddenCode` | Use of a forbidden construct such as `var_dump`, `shell_exec`, or the backtick operator. |

## Parse

| Kind | Description |
|------|-------------|
| `ParseError` | A PHP file could not be parsed. |

## Other

| Kind | Description |
|------|-------------|
| `InvalidThrow` | A value that does not implement Throwable is thrown. |
| `ImplicitToStringCast` | An object without `__toString` or `Stringable` is implicitly coerced to a string (e.g. in string concatenation). |
| `ImplicitFloatToIntCast` | A float is implicitly cast to an int, potentially losing precision. |
| `InvalidCatch` | Catching a type that is not `Throwable`. |
| `NoInterfaceProperties` | A property is accessed on an interface that seals properties and does not declare it via `@property`. |
| `UndefinedDocblockClass` | A class referenced only in a docblock does not exist. |
| `UnsupportedReferenceUsage` | A PHP reference assignment is used in a form mir cannot model precisely (e.g. `$b = &$arr[$x]`). |
| `MissingConstructor` | A class has non-nullable, uninitialized typed properties but no constructor to initialize them. |

## Attributes

| Kind | Description |
|------|-------------|
| `InvalidAttribute` | `#[Attribute]` usage violates target restrictions or argument constraints. |
| `UndefinedAttributeClass` | Attribute class referenced with `#[...]` does not exist. |
| `DuplicateClass` | Two classes with the same fully-qualified name exist in the codebase. |
| `DuplicateInterface` | An interface with the same name is declared more than once. |
| `DuplicateTrait` | A trait with the same name is declared more than once. |
| `DuplicateEnum` | An enum with the same name is declared more than once. |
| `DuplicateFunction` | A function with the same name is declared more than once. |

## Purity

| Kind | Description |
|------|-------------|
| `ImpurePropertyAssignment` | A function marked `@pure` assigns to a property. |
| `ImpureMethodCall` | A `@pure` function calls an impure method. |
| `ImpureGlobalVariable` | A `@pure` function reads or writes a global variable. |
| `ImpureStaticVariable` | A `@pure` function uses a static variable. |
| `ImpureFunctionCall` | A `@pure` function calls a non-pure function. |
