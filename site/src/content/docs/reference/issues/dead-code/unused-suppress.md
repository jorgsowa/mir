---
title: UnusedSuppress
code: MIR0508
description: "A suppression annotation (`@mir-ignore`, `@psalm-suppress`, `@suppress`) did not match any issue."
sidebar:
  hidden: true
  order: 508
---

A suppression annotation that names an issue kind (`@mir-ignore UndefinedClass`, or the `@psalm-suppress` and `@suppress` aliases) did not match any issue on its target line.

## Example

```php
<?php
/** @mir-ignore UndefinedClass */
$x = new KnownClass(); // nothing to suppress
```

## How to fix

Remove the stale suppression annotation.
