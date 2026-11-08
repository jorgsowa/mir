---
title: Docblock Annotations
description: Docblock tags mir reads for types, generics, assertions, purity, taint and dead code.
---

Native type declarations are always checked. Docblock tags add what PHP's type
syntax can't express, such as generics, array shapes and assertions.

Unless a table says otherwise, every tag also works with a `@psalm-` or
`@phpstan-` prefix (`@psalm-param`, `@phpstan-return`), so annotations written
for other analyzers are read as they are.

## Types

| Tag | Effect |
|-----|--------|
| `@param Type $name` | Parameter type |
| `@return Type` | Return type |
| `@var Type` / `@var Type $name` | Property, constant or variable type |
| `@param-out Type $name` | Type of a by-reference parameter after the call |
| `@throws ClassName` | Exception the function can throw |

A docblock type must be compatible with the native declaration; a contradiction
is reported as `MismatchingDocblockParamType` or `MismatchingDocblockReturnType`.

## Type aliases

Only the prefixed forms are supported.

| Tag | Effect |
|-----|--------|
| `@psalm-type Name = Type` / `@phpstan-type` | Declares an alias usable in the class's docblocks |
| `@psalm-import-type Name from Class` / `@phpstan-import-type` | Imports an alias from another class, optionally `as Other` |

## Generics

| Tag | Effect |
|-----|--------|
| `@template T` | Declares a type parameter |
| `@template T of Bound` | Type parameter constrained to `Bound` |
| `@template T = Default` | Type parameter with a default |
| `@template-covariant T` / `@template-contravariant T` | Type parameter with variance |
| `@extends Parent<Type>` | Binds the parent class's type parameters (also `@template-extends`) |
| `@implements Interface<Type>` | Binds an interface's type parameters (also `@template-implements`) |
| `@use Trait<Type>` | Binds a trait's type parameters (also `@template-use`) |

## Assertions and narrowing

| Tag | Effect |
|-----|--------|
| `@assert Type $x` | After the call, `$x` is `Type` |
| `@assert-if-true Type $x` | When the call returns `true`, `$x` is `Type` |
| `@assert-if-false Type $x` | When the call returns `false`, `$x` is `Type` |
| `@self-out Type` | After the call, `$this` is `Type` (also `@this-out`) |
| `@if-this-is Type` | The method may only be called when `$this` is `Type` |
| `@param-closure-this Type $callback` | Type of `$this` inside the closure passed as `$callback` |

Assertions accept negation (`!null`), properties (`$this->prop`) and array
elements. A function whose body is a single `instanceof` check narrows its
argument without any annotation.

## Magic members

| Tag | Effect |
|-----|--------|
| `@property Type $name` | Declares a magic property readable and writable through `__get` / `__set` |
| `@property-read Type $name` | Read-only magic property |
| `@property-write Type $name` | Write-only magic property |
| `@method [static] ReturnType name(params)` | Declares a magic method handled by `__call` / `__callStatic` |
| `@mixin ClassName` | Forwards unknown method calls and property fetches to `ClassName` |

## Class constraints

Only the prefixed forms are supported.

| Tag | Effect |
|-----|--------|
| `@psalm-require-extends Class` / `@phpstan-require-extends` | Classes using this trait or implementing this interface must extend `Class` |
| `@psalm-require-implements Interface` / `@phpstan-require-implements` | Classes using this trait must implement `Interface` |

## Purity and immutability

| Tag | Effect |
|-----|--------|
| `@pure` | The function has no side effects and depends only on its arguments; impure operations inside it are reported |
| `@mutation-free` | The method doesn't change any state |
| `@external-mutation-free` | The method changes only `$this` |
| `@impure` | The function has side effects, even if it looks pure |
| `@immutable` | No property of the class changes after construction |
| `@readonly` | The property (or every property of the class) is assigned only once |

Violations are reported as `Impure*` issues and
`ImmutablePropertyModification`.

## Taint analysis

Each tag also accepts the `@psalm-` prefix.

| Tag | Effect |
|-----|--------|
| `@taint-source [kind]` | The return value is user-controlled input |
| `@taint-sink <kind> $param` | Tainted data passed to `$param` is reported (`html`, `sql`, `shell`, `header`, `cookie`, `file`, `include`, `ssrf`, `ldap`, `xpath`, `callable`, `unserialize`, `eval`, `llm_prompt`) |
| `@taint-escape <kind>` | The return value is safe for sinks of `<kind>` |
| `@taint-unescape <kind>` | The return value is dangerous again for `<kind>` sinks when its input was tainted |
| `@psalm-flow ($a, $b) -> return` | The listed parameters' taint flows into the return value of a method without a body (also `@taint-flow`) |

Without annotations, request data is tainted wherever it is read: `$_GET`,
`$_POST`, `$_COOKIE`, `$_FILES`, `$_REQUEST`, client-controlled `$_SERVER`
entries, `php://input`, and the request objects of Symfony, Laravel and PSR-7.

Function and method bodies are summarized automatically: a call's result carries
the taint of the arguments its body returns, and an argument that reaches a sink
inside the callee is reported at the call.

## Deprecation, visibility and dead code

| Tag | Effect |
|-----|--------|
| `@deprecated` | Uses are reported as `DeprecatedMethod`, `DeprecatedClass`, `DeprecatedProperty`, … |
| `@internal` | Calls from outside the top-level namespace are reported as `InternalMethod` (also `@psalm-internal`) |
| `@api` | Exempts the symbol from unused-code reports (also `@psalm-api`) |
| `@no-named-arguments` | Calls with named arguments are reported as `InvalidNamedArguments` (unprefixed only) |
| `@dataProvider name` | Marks the provider method as used |
| `@inheritDoc` | Inherits the parent's docblock types (also `{@inheritDoc}`) |

## Debugging

| Tag | Effect |
|-----|--------|
| `@trace $var` | Reports the inferred type of `$var` at this point as a `Trace` issue |

## Type syntax

mir understands the PHPDoc type syntax shared by PHP analyzers:

```text
int|string                       union
Foo&Bar                          intersection
?string                          nullable (string|null)
'a'|'b', 1|2, true               literal types
array<int, string>               array with key and value types
list<string>                     list (sequential integer keys from 0)
non-empty-array<T>, non-empty-list<T>
array{id: int, name?: string}    array shape (name is optional)
object{id: int}                  object shape
Foo<T>                           generic class
class-string<T>                  name of a class that is T or a subtype
key-of<T>, value-of<T>           keys or values of an array type or enum
int<0, max>, positive-int        integer ranges
non-empty-string, numeric-string, lowercase-string, literal-string
callable(int): string            callable signature
Closure(int): string             closure signature
($x is int ? string : bool)      conditional return type
never, void, mixed, null         special types
```
