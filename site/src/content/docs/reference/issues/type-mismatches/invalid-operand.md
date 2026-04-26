---
title: InvalidOperand
description: An operator is applied to incompatible types.
sidebar:
  order: 9
---

An operator is applied to incompatible types.

## Example

```php
<?php
$result = [] + 5; // array + int is not valid
```

## How to fix

Ensure both operands are of compatible types for the operator used.
