---
title: TypeDoesNotContainType
description: A type check can never be true because the type does not include the tested type.
sidebar:
  order: 4
---

A type check can never be true because the type does not include the tested type.

## Example

```php
<?php
function handle(int $n): void {
    if ($n instanceof stdClass) { // int can never be stdClass
        // unreachable
    }
}
```

## How to fix

Remove the impossible check or correct the type of the variable.
