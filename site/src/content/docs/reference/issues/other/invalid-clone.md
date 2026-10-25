---
title: InvalidClone
code: MIR1205
description: Cloning a non-object, an enum, or an object whose private __clone() is out of scope.
sidebar:
  hidden: true
  order: 1205
---

The `clone` keyword is used on a value that PHP cannot clone: a value that is not an object
(a scalar or an array), an enum case (`Trying to clone an uncloneable object`), or an object whose
`__clone()` is private and called from outside its declaring class.

## Example

```php
<?php
function duplicate(string $value): string {
    return clone $value; // cannot clone a string
}
```

## How to fix

Only use `clone` on objects. If you need to copy a scalar or array, just assign it to a new
variable (PHP copies these by value). Enum cases are singletons: use the case itself.
