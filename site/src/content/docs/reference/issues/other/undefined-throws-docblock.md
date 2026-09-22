---
title: UndefinedThrowsDocblock
code: MIR1106
description: A `@throws` docblock references an exception class that does not exist.
sidebar:
  hidden: true
  order: 1
---

A `@throws` docblock references an exception class that does not exist or is not imported.

## Example

```php
<?php
/**
 * @throws UndefinedException
 */
function process(): void {
    // No such class exists
}
```

## How to fix

Ensure the exception class exists and is properly imported:

```php
<?php
use InvalidArgumentException;

/**
 * @throws InvalidArgumentException
 */
function process(string $value): void {
    if ($value === '') {
        throw new InvalidArgumentException('Value cannot be empty');
    }
}
```
