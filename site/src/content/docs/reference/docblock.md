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

## Taint analysis

Each tag also accepts the `@psalm-` prefix.

| Annotation | Effect |
|-----------|--------|
| `@taint-source` | The return value is user-controlled input |
| `@taint-sink <kind> $param` | Tainted data passed to `$param` is reported (`html`, `sql`, `shell`, `header`, `cookie`, `file`, `unserialize`, `eval`, `llm_prompt`, ...) |
| `@taint-escape <kind>` | The return value is safe for sinks of `<kind>` |
| `@taint-unescape <kind>` | The return value is dangerous again for `<kind>` sinks when its input was tainted |
| `@psalm-flow ($a, $b) -> return` | The listed parameters' taint flows into the return value of a method without a body |

Without annotations, request data is tainted wherever it is read: `$_GET`, `$_POST`, `$_COOKIE`, `$_FILES`, `$_REQUEST`, client-controlled `$_SERVER` entries, `php://input`, and the request objects of Symfony, Laravel and PSR-7.

Function and method bodies are summarized automatically: a call's result carries the taint of the arguments its body returns, and an argument that reaches a sink inside the callee is reported at the call.

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
