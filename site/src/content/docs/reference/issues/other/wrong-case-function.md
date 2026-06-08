---
title: WrongCaseFunction
code: MIR1009
description: Function name casing does not match its declaration.
sidebar:
  hidden: true
  order: 1009
---

A function is called with a casing that differs from its declaration. While PHP function calls
are case-insensitive at runtime, using consistent casing improves readability and avoids
confusion.

## Example

```php
<?php
function calculateTotal(float $price): float {
    return $price * 1.2;
}

echo CalculateTotal(10.0); // wrong casing: should be calculateTotal
```

## How to fix

Update the call site to use the exact casing from the function declaration.
