---
title: ArgumentTypeCoercion
code: MIR0225
description: "An argument's type had to be widened/coerced to match the parameter's declared type."
sidebar:
  hidden: true
  order: 225
---

An argument's type had to be widened/coerced to match the parameter's declared type.

## Example

```php
<?php
function f(Child $c){}
/** @param Parent $p */ function g($p){ f($p); }
```

## How to fix

Pass a value of the exact expected type, or narrow first.
