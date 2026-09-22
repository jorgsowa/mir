---
title: ImpossibleLooseComparison
code: MIR0409
description: A loose comparison (`==`) that can never be true due to incompatible types.
sidebar:
  hidden: true
  order: 9
---

A loose comparison (`==`) between values of incompatible types that can never be true. This indicates dead code or a logic error.

## Example

```php
<?php
function checkValue($value): bool {
    if ($value == null && $value === false) {
        // This can never be true - null and false have different meanings
        return true;
    }
    return false;
}
```

## How to fix

Review the comparison logic and remove impossible conditions:

```php
<?php
function checkValue($value): bool {
    // Check for null OR false separately
    if ($value === null || $value === false) {
        return true;
    }
    return false;
}
```
