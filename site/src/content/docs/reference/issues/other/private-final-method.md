---
title: PrivateFinalMethod
code: MIR1512
description: A method other than the constructor is declared both private and final.
sidebar:
  hidden: true
  order: 10
---

A private method is never overridden by a child class, so `final` on it has no effect. PHP warns about it for every method except the constructor.

## Example

```php
<?php
class A {
    final private function helper(): void {}
}
```

## How to fix

Drop `final`, or make the method `protected` if child classes should see it but not override it.
