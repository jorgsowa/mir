---
title: StaticPropertyRedeclarationMismatch
code: MIR0715
description: A static property in a child class conflicts with the parent's declaration.
sidebar:
  hidden: true
  order: 15
---

A static property in a child class conflicts with the parent's declaration, such as incompatible types or modifiers.

## Example

```php
<?php
class Parent {
    public static string $name;
}

class Child extends Parent {
    public static int $name; // Error: incompatible type
}
```

## How to fix

Ensure the child class static property type is compatible with the parent:

```php
<?php
class Parent {
    public static string $name;
}

class Child extends Parent {
    // Keep the same type
    public static string $name;
}
```
