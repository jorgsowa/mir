---
title: PropertyTypeRedeclarationMismatch
code: MIR0712
description: A property type declaration in a child class is incompatible with the parent.
sidebar:
  hidden: true
  order: 12
---

A property type declaration in a child class is incompatible with the parent class declaration. Property types must be compatible across the inheritance hierarchy.

## Example

```php
<?php
class Parent {
    public string $name;
}

class Child extends Parent {
    public int $name; // Error: incompatible type
}
```

## How to fix

Ensure the child class property type is compatible with the parent:

```php
<?php
class Parent {
    public string $name;
}

class Child extends Parent {
    // Keep the same type or use a more specific subtype
    public string $name;
}
```
