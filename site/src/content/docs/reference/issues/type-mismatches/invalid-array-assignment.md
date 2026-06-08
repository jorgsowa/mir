---
title: InvalidArrayAssignment
code: MIR0220
description: Array-style assignment on a non-array type.
sidebar:
  hidden: true
  order: 220
---

An assignment of the form `$value[key] = ...` is made to a value whose type does not support
array-style assignment. Only arrays and objects implementing `ArrayAccess` support this syntax.

## Example

```php
<?php
function setFirst(string $items, mixed $val): void {
    $items[0] = $val; // string does not support array-style assignment
}
```

## How to fix

Change the type to an array or an `ArrayAccess` implementation, or avoid array-style assignment.
