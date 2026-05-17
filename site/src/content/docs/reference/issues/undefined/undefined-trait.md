---
title: UndefinedTrait
code: MIR0009
description: A trait is used that does not exist in the codebase or stubs.
sidebar:
  hidden: true
  order: 9
---

A trait is used that does not exist in the codebase or stubs.

## Example

```php
<?php
class Foo {
    use Loggable; // trait not defined anywhere
}
```

## How to fix

Define the trait, add the missing `use` import, or fix the spelling.
