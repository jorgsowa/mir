---
title: ImpossibleIdenticalComparison
code: MIR0408
description: An identical comparison (`===`) that can never be true due to incompatible types.
sidebar:
  hidden: true
  order: 8
---

An identical comparison (`===`) between values of incompatible types that can never be true. This indicates dead code or a logic error.

## Example

```php
<?php
function checkType($value): bool {
    if ($value === 'string' && $value === 123) {
        // This can never be true - a value cannot be both string and int
        return true;
    }
    return false;
}
```

## How to fix

Review the comparison logic and remove impossible conditions:

```php
<?php
function checkType($value): bool {
    // Check for string OR int instead
    if ($value === 'string' || $value === 123) {
        return true;
    }
    return false;
}
```
