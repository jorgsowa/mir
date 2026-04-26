---
title: UnreachableCode
description: Code appears after an unconditional return, throw, or exit.
sidebar:
  order: 6
---

Code appears after an unconditional `return`, `throw`, or `exit`.

## Example

```php
<?php
function early(): int {
    return 1;
    $x = 2; // never executed
}
```

## How to fix

Remove the unreachable statements or restructure the control flow.
