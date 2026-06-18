---
title: ImpureStaticVariable
code: MIR1703
description: "A `@pure` function uses a static variable."
sidebar:
  hidden: true
  order: 1703
---

A `@pure` function uses a static variable.

## Example

```php
<?php
/** @pure */ function f(){ static $n = 0; return ++$n; }
```

## How to fix

Remove the static state, or drop `@pure`.
