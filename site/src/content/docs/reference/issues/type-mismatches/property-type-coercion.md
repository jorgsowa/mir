---
title: PropertyTypeCoercion
code: MIR0226
description: "A value assigned to a property had to be coerced to the property's declared type."
sidebar:
  hidden: true
  order: 226
---

A value assigned to a property had to be coerced to the property's declared type.

## Example

```php
<?php
class C { public Child $c; }
/** @param Parent $p */ function f(C $o,$p){ $o->c = $p; }
```

## How to fix

Assign a value of the exact declared type.
