---
title: InvalidDocblockType
code: MIR1107
description: A type in a docblock is invalid or cannot be resolved.
sidebar:
  hidden: true
  order: 2
---

A type in a docblock is invalid, malformed, or cannot be resolved to a known class or built-in type.

## Example

```php
<?php
/**
 * @param InvalidType $value
 */
function process($value): void {
    // InvalidType does not exist
}
```

## How to fix

Use valid types that exist or are properly imported:

```php
<?php
use DateTime;

/**
 * @param DateTime|string $value
 */
function process($value): void {
    // Valid types
}
```
