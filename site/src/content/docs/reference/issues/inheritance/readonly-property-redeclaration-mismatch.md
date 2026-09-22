---
title: ReadonlyPropertyRedeclarationMismatch
code: MIR0714
description: A readonly property in a child class conflicts with the parent's declaration.
sidebar:
  hidden: true
  order: 14
---

A readonly property in a child class conflicts with the parent's declaration, such as redeclaring a non-readonly property as readonly or vice versa.

## Example

```php
<?php
class Parent {
    public string $name;
}

class Child extends Parent {
    public readonly string $name; // Error: cannot change readonly modifier
}
```

## How to fix

Maintain consistent readonly modifiers across the inheritance hierarchy:

```php
<?php
class Parent {
    public readonly string $name;
}

class Child extends Parent {
    // Keep the same readonly modifier
    public readonly string $name;
}
```
