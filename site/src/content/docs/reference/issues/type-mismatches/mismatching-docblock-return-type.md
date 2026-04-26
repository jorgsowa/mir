---
title: MismatchingDocblockReturnType
description: The @return docblock type conflicts with the native return type hint.
sidebar:
  order: 10
---

The `@return` docblock type conflicts with the native return type hint.

## Example

```php
<?php
/** @return string */
function getId(): int { // docblock says string, native says int
    return 1;
}
```

## How to fix

Align the `@return` annotation with the native type hint, or remove one of them.
