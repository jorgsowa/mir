---
title: InvalidArrayOffset
description: An array is accessed with a key of the wrong type.
sidebar:
  order: 1
---

An array is accessed with a key of the wrong type.

## Example

```php
<?php
/** @param array<string, int> $map */
function get(array $map): int {
    return $map[0]; // int key used on string-keyed array
}
```

## How to fix

Use the correct key type as declared in the array's type annotation.
