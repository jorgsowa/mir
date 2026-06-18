---
title: UnusedSuppress
code: MIR0508
description: "A suppression annotation (`@psalm-suppress` / `@mir-suppress` / `@suppress`) did not match any issue."
sidebar:
  hidden: true
  order: 508
---

A suppression annotation (`@psalm-suppress` / `@mir-suppress` / `@suppress`) did not match any issue.

## Example

```php
<?php
/** @psalm-suppress UndefinedClass */
$x = new KnownClass(); // nothing to suppress
```

## How to fix

Remove the stale suppression annotation.
