---
title: MixedPropertyAssignment
code: MIR1208
description: "A `mixed` value is assigned to a property, hiding its concrete type."
sidebar:
  hidden: true
  order: 1208
---

A `mixed` value is assigned to a property, hiding its concrete type.

## Example

```php
<?php
class C { public int $n; }
/** @param mixed $x */ function f(C $o,$x){ $o->n = $x; }
```

## How to fix

Narrow the value to the property's type before assigning.
