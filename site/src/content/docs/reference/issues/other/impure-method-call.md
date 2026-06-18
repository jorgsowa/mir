---
title: ImpureMethodCall
code: MIR1701
description: "A `@pure` function calls an impure method."
sidebar:
  hidden: true
  order: 1701
---

A `@pure` function calls an impure method.

## Example

```php
<?php
/** @pure */ function f(C $o){ $o->mutate(); }
```

## How to fix

Call only pure methods, or drop `@pure`.
