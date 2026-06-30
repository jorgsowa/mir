---
title: Suppressions
description: How to silence false positives inline using suppression directives.
---

Inline suppression lets you silence a false positive directly in source without
modifying `mir.xml` or a baseline file. A suppression applies only to the
specific line (or file) you target — it does not affect the rest of the
codebase.

## Native directives

| Directive | Scope |
|-----------|-------|
| `@mir-ignore [Kind …]` | trailing comment → its line; standalone comment → next code line |
| `@mir-ignore-line [Kind …]` | the comment's own line |
| `@mir-ignore-next-line [Kind …]` | the next physical line |
| `@mir-ignore-file [Kind …]` | the entire file |

`@mir-suppress*` is accepted as an alias for every `@mir-ignore*` form.

## Third-party aliases

mir accepts these for drop-in compatibility with Psalm and PHPStan:

| Directive | Scope / kinds |
|-----------|---------------|
| `@psalm-suppress Kind …` | like `@mir-ignore` (named kinds) |
| `@suppress Kind …` | like `@mir-ignore` (named kinds) |
| `@phpstan-ignore-line` | the comment's own line, all kinds |
| `@phpstan-ignore-next-line` | the next physical line, all kinds |
| `@phpstan-ignore …` | the next physical line, all kinds |

PHPStan identifiers do not map onto mir's issue names, so `@phpstan-ignore*`
forms always suppress every issue on their target line regardless of any kind
arguments.

## Specifying kinds

When no kind is given, all issues on the target line are suppressed. Kinds may
be given by **name** (`UndefinedClass`) or by **code** (`MIR0123`), and
multiple kinds are space- or comma-separated:

```php
// @mir-ignore UndefinedClass
$obj = new LegacyClass();

// @mir-ignore MIR0001, MIR0002
doSomething();

/** @psalm-suppress InvalidArgument */
$result = takesInt("string");

$x = mystery(); // @mir-ignore-line
```

## Unused suppressions

mir reports `UnusedSuppress` when a named suppression does not match any
issue on its target line, helping you remove stale annotations after a fix.
