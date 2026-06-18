---
title: MixedArrayOffset
code: MIR1210
description: "An array is indexed with an offset of type `mixed`."
sidebar:
  hidden: true
  order: 1210
---

An array is indexed with an offset of type `mixed`.

## Example

```php
<?php
/** @param mixed $k */ function f(array $a,$k){ return $a[$k]; }
```

## How to fix

Narrow the offset to `int|string`.
