---
title: PossiblyInvalidArrayAccess
description: Array access is performed on a value that might not be an array.
sidebar:
  order: 4
---

Array access is performed on a value that might not be an array.

## Example

```php
<?php
function first(array|false $rows): mixed {
    return $rows[0]; // $rows might be false
}
```

## How to fix

Guard with a type check before the array access.

```php
<?php
function first(array|false $rows): mixed {
    if ($rows === false) {
        return null;
    }
    return $rows[0];
}
```
