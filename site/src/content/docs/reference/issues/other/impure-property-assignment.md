---
title: ImpurePropertyAssignment
code: MIR1700
description: "A function marked `@pure` assigns to a property."
sidebar:
  hidden: true
  order: 1700
---

A function marked `@pure` assigns to a property.

## Example

```php
<?php
/** @pure */ function f(C $o){ $o->n = 1; }
```

## How to fix

Remove the side effect, or drop the `@pure` annotation.
