---
title: PossiblyNullOperand
code: MIR0214
description: Operand could be `null`, making the operation potentially unsafe.
sidebar:
  hidden: true
  order: 214
---

An arithmetic or string operation is performed on a value that could be `null`. Passing `null`
to most operators produces unexpected results or a warning at runtime.

## Example

```php
<?php
function divide(?int $a, int $b): float {
    return $a / $b; // $a could be null
}
```

## How to fix

Guard against `null` before the operation:

```php
<?php
function divide(?int $a, int $b): float {
    if ($a === null) {
        return 0.0;
    }
    return $a / $b;
}
```
