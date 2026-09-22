---
title: InvalidReadonlyPropertyDeclaration
code: MIR0717
description: A readonly property is declared in a way that violates PHP's readonly rules.
sidebar:
  hidden: true
  order: 17
---

A readonly property is declared in a way that violates PHP's readonly rules, such as declaring it as static or abstract.

## Example

```php
<?php
class User {
    public static readonly string $name; // Error: readonly cannot be static
}
```

## How to fix

Remove the invalid modifier combination:

```php
<?php
class User {
    public readonly string $name; // Correct: instance property with readonly
}
```
