---
title: DivisionByZero
code: MIR0229
description: A division operation that will always result in division by zero.
sidebar:
  hidden: true
  order: 29
---

A division operation uses a divisor that is statically known to be zero, which will cause a runtime error.

## Example

```php
<?php
function calculate(int $value): float {
    return $value / 0; // Division by zero
}
```

## How to fix

Ensure the divisor is not zero, or add a check before division:

```php
<?php
function calculate(int $value, int $divisor): float {
    if ($divisor === 0) {
        throw new InvalidArgumentException('Divisor cannot be zero');
    }
    return $value / $divisor;
}
```
