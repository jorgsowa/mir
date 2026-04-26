---
title: RedundantCondition
description: A condition is always true or always false based on the known types.
sidebar:
  order: 1
---

A condition is always true or always false based on the known types.

## Example

```php
<?php
function process(int $n): void {
    if (is_int($n)) { // always true — $n is already typed as int
        // ...
    }
}
```

## How to fix

Remove the redundant check or widen the type to make the check meaningful.
