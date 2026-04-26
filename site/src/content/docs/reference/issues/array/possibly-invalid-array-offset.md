---
title: PossiblyInvalidArrayOffset
description: An array is accessed with a key that might not exist.
sidebar:
  order: 3
---

An array is accessed with a key that might not exist.

## Example

```php
<?php
function get(array $data, string $key): mixed {
    return $data[$key]; // $key may not be present in $data
}
```

## How to fix

Check for the key with `array_key_exists()` or `isset()` before accessing it.
