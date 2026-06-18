---
title: ImpureGlobalVariable
code: MIR1702
description: "A `@pure` function reads or writes a global variable."
sidebar:
  hidden: true
  order: 1702
---

A `@pure` function reads or writes a global variable.

## Example

```php
<?php
/** @pure */ function f(){ global $g; return $g; }
```

## How to fix

Pass the value as a parameter, or drop `@pure`.
