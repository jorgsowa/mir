---
title: InvalidReturnType
description: The returned value does not match the declared return type.
sidebar:
  order: 1
---

The returned value does not match the declared return type.

## Example

```php
<?php
function getCount(): int {
    return '5'; // string returned, int expected
}
```

## How to fix

Return a value of the correct type or update the return type declaration.
