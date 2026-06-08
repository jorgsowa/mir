---
title: PossiblyInvalidOperand
code: MIR0213
description: Operator applied to a union type with some incompatible members.
sidebar:
  hidden: true
  order: 213
---

An operator is applied to a value whose type is a union, and at least one member of that union
is not compatible with the operator. At runtime the operation may succeed or fail depending on
which branch of the union is actually present.

## Example

```php
<?php
function double(int|string $value): int|float {
    return $value * 2; // string is not a valid operand for *
}
```

## How to fix

Narrow the type before applying the operator, or ensure all union members are compatible:

```php
<?php
function double(int|string $value): int|float {
    if (!is_int($value)) {
        throw new \InvalidArgumentException('Expected int');
    }
    return $value * 2;
}
```
