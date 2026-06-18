---
title: ImpureFunctionCall
code: MIR1704
description: "A `@pure` function calls a non-pure function."
sidebar:
  hidden: true
  order: 1704
---

A `@pure` function calls a non-pure function.

## Example

```php
<?php
/** @pure */ function f(){ return time(); }
```

## How to fix

Call only pure functions, or drop `@pure`.
