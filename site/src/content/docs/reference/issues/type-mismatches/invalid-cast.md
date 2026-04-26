---
title: InvalidCast
description: An explicit cast can never succeed for the given type.
sidebar:
  order: 8
---

An explicit cast can never succeed for the given type.

## Example

```php
<?php
/** @param array $data */
function process(array $data): int {
    return (int) $data; // casting array to int is always 1 or 0, never meaningful
}
```

## How to fix

Use an appropriate conversion function or remove the unnecessary cast.
