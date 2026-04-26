---
title: InvalidPassByReference
description: A by-reference parameter receives an expression that cannot be referenced.
sidebar:
  order: 6
---

A by-reference parameter receives an expression that cannot be referenced.

## Example

```php
<?php
function increment(int &$val): void { $val++; }

increment(42); // literal cannot be passed by reference
```

## How to fix

Pass a variable instead of a literal or expression.
