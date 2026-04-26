---
title: NullableReturnStatement
description: A nullable value is returned from a function with a non-nullable return type.
sidebar:
  order: 5
---

A nullable value is returned from a function with a non-nullable return type.

## Example

```php
<?php
function getName(): string {
    return null; // null is not a string
}
```

## How to fix

Change the return type to `?string` or ensure a non-null value is always returned.
