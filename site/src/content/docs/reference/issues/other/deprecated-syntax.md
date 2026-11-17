---
title: DeprecatedSyntax
code: MIR1015
description: A declaration or statement that PHP deprecates at compile time for the target version.
sidebar:
  hidden: true
  order: 10
---

The message is the deprecation PHP emits when it compiles the code. From PHP 8.6 these are:

- `return` inside a `finally` block.
- `return` with a value, or `yield`, in `__construct()` or `__destruct()`.
- A class constant named `namespace`.
- A function named `readonly`.
- `let` or `is` as the name of a class, interface, trait, enum, function or constant, or as a `use` alias.
- `_` as a constant name or as a `use const` alias.
- A `__debugInfo()` return type that allows `null`.

## Example

```php
<?php
function load(): string {
    try {
        return read();
    } finally {
        return 'default';
    }
}
```

## How to fix

Move the `return` out of the `finally` block, so the block only cleans up.
