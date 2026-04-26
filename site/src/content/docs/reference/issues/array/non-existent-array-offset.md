---
title: NonExistentArrayOffset
description: An array is accessed with a key that is known not to exist.
sidebar:
  order: 2
---

An array is accessed with a key that is known not to exist.

## Example

```php
<?php
$point = ['x' => 1, 'y' => 2];
echo $point['z']; // key 'z' does not exist in this literal array
```

## How to fix

Use an existing key or add the key to the array shape definition.
