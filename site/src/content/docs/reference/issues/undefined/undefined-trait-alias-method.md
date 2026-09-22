---
title: UndefinedTraitAliasMethod
code: MIR0013
description: A method from a trait alias is used but the alias was not properly defined.
sidebar:
  hidden: true
  order: 13
---

A method from a trait alias is referenced, but the trait alias was not properly defined or the method does not exist in the trait.

## Example

```php
<?php
trait MyTrait {
    public function originalMethod(): void {}
}

class MyClass {
    use MyTrait {
        originalMethod as aliasedMethod;
    }
}

$obj = new MyClass();
// Error: aliasedMethod may not be properly resolved
$obj->aliasedMethod();
```

## How to fix

Ensure the trait alias is properly defined and the method exists in the trait:

```php
<?php
trait MyTrait {
    public function originalMethod(): void {}
}

class MyClass {
    use MyTrait {
        originalMethod as aliasedMethod;
    }
}

$obj = new MyClass();
// Correct: alias is properly defined
$obj->aliasedMethod();
```
