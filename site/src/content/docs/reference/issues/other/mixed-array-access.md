---
title: MixedArrayAccess
code: MIR1209
description: "Array access produced a value of type `mixed`."
sidebar:
  hidden: true
  order: 1209
---

Array access produced a value of type `mixed`.

## Example

```php
<?php
/** @param mixed $a */ function f($a){ return $a['k']; }
```

## How to fix

Type the array, e.g. `array<string, int>`.
