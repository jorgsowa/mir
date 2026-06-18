---
title: Trace
code: MIR0221
description: "Internal/debug output of the `@trace` annotation, showing the inferred type of an expression. Not a project diagnostic."
sidebar:
  hidden: true
  order: 221
---

This is an **internal diagnostic**. Internal/debug output of the `@trace` annotation, showing the inferred type of an expression. Not a project diagnostic.

## Example

```php
<?php
/** @trace $x */
$x = 42; // Trace: int
```

## How to fix

This is a debugging aid for fixtures; remove the `@trace` annotation to silence it.
