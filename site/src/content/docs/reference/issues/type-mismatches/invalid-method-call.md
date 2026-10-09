---
title: InvalidMethodCall
code: MIR0230
description: Method call on a non-object type.
sidebar:
  hidden: true
  order: 230
---

A method is called with `->method()` on a value that is a scalar or array. At runtime this
throws an `Error`.

## Example

```php
<?php
function label(string $item): string {
    return $item->label(); // string is not an object
}
```

## How to fix

Ensure the value is an object before calling methods on it, or fix the type annotation:

```php
<?php
function label(Item $item): string {
    return $item->label();
}
```
