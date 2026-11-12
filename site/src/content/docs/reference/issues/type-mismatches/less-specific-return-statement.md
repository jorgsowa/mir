---
title: LessSpecificReturnStatement
code: MIR0110
description: "A returned value's type is wider than the declared return type that it may still satisfy."
sidebar:
  hidden: true
  order: 110
---

The returned value's type is a supertype of the declared return type, or contains elements that
are: the value may satisfy the declaration at runtime, but nothing proves it.

## Example

```php
<?php
/** @return array<string, int> */
function counts(array $rows): array {
    return array_count_values($rows); // array<array-key, int>
}
```

## How to fix

Narrow the value before returning it, or widen the declared return type.
