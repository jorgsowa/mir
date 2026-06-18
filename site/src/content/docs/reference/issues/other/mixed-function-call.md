---
title: MixedFunctionCall
code: MIR1211
description: "A dynamic call target has type `mixed`."
sidebar:
  hidden: true
  order: 1211
---

A dynamic call target has type `mixed`.

## Example

```php
<?php
/** @param mixed $fn */ function f($fn){ $fn(); }
```

## How to fix

Type the callable, e.g. `callable` or `Closure`.
