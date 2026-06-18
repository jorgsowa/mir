---
title: AbstractMethodCall
code: MIR0711
description: "An abstract method is invoked where no concrete implementation exists."
sidebar:
  hidden: true
  order: 711
---

An abstract method is invoked where no concrete implementation exists.

## Example

```php
<?php
abstract class A { abstract public static function f(): void; }
A::f(); // AbstractMethodCall
```

## How to fix

Call it on a concrete subclass that implements the method.
