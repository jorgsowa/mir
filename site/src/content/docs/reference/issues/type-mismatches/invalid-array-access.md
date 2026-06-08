---
title: InvalidArrayAccess
code: MIR0219
description: Array-style access on a non-array, non-string, non-`ArrayAccess` type.
sidebar:
  hidden: true
  order: 219
---

The `[key]` bracket access syntax is used on a value whose type does not support it. Only
arrays, strings, and objects implementing `ArrayAccess` (or `Countable`/`Traversable` in some
contexts) support bracket access.

## Example

```php
<?php
function first(int $value): mixed {
    return $value[0]; // int does not support array access
}
```

## How to fix

Ensure the value is an array, string, or `ArrayAccess` before using bracket notation.
