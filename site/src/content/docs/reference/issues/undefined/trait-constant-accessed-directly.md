---
title: TraitConstantAccessedDirectly
code: MIR0012
description: A trait constant is accessed directly, which is not allowed in PHP.
sidebar:
  hidden: true
  order: 12
---

A trait constant is accessed directly (e.g., `MyTrait::CONST_NAME`). In PHP, trait constants cannot be accessed directly; they must be accessed through a class that uses the trait.

## Example

```php
<?php
trait MyTrait {
    const NAME = 'trait';
}

class MyClass {
    use MyTrait;
}

// Error: accessing trait constant directly
echo MyTrait::NAME;
```

## How to fix

Access the constant through a class that uses the trait:

```php
<?php
trait MyTrait {
    const NAME = 'trait';
}

class MyClass {
    use MyTrait;
}

// Correct: access through the class
echo MyClass::NAME;
```
