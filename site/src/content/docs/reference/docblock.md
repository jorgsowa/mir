---
title: Docblock Annotations
description: Supported docblock annotations for type information and suppression.
---

mir understands the following docblock annotations.

## Type annotations

| Annotation | Effect |
|-----------|--------|
| `@param Type $name` | Parameter type override |
| `@return Type` | Return type override |
| `@var Type` | Variable type annotation |
| `@throws ClassName` | Declares thrown exception |

## Generics

| Annotation | Effect |
|-----------|--------|
| `@template T` | Declares a type parameter |
| `@template T of U` | Bounded type parameter (`T` must extend `U`) |

## Psalm-compatible

| Annotation | Effect |
|-----------|--------|
| `@psalm-suppress IssueName` | Suppress a specific issue at this site |
| `@psalm-pure` | Marks function as side-effect-free |
| `@psalm-immutable` | Marks class as immutable |

## Metadata

| Annotation | Effect |
|-----------|--------|
| `@deprecated` | Marks class/method as deprecated (emits `DeprecatedMethod`/`DeprecatedClass`) |
| `@internal` | Marks as internal (emits `InternalMethod` if called from outside the package) |

## Type syntax

mir supports standard Psalm/PHPStan type syntax:

```
int|string          union
?string             nullable (shorthand for string|null)
array<int, string>  typed array
list<string>        list (sequential integer keys from 0)
class-string<T>     string containing a class name
callable            callable type
never               bottom type (function never returns)
void                function returns with no value
mixed               any type
```
