---
title: RedundantCast
description: A value is cast to a type it already has.
sidebar:
  order: 2
---

A value is cast to a type it already has.

## Example

```php
<?php
function wrap(int $n): int {
    return (int) $n; // $n is already an int
}
```

## How to fix

Remove the unnecessary cast.
