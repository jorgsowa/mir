---
title: InvalidClone
code: MIR1205
description: Cloning a non-object type.
sidebar:
  hidden: true
  order: 1205
---

The `clone` keyword is used on a value that is not an object. PHP's `clone` operator is only
valid for objects; using it on scalars or arrays produces a fatal error.

## Example

```php
<?php
function duplicate(string $value): string {
    return clone $value; // cannot clone a string
}
```

## How to fix

Only use `clone` on objects. If you need to copy a scalar or array, just assign it to a new
variable (PHP copies these by value).
