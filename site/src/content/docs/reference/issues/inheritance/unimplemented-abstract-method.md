---
title: UnimplementedAbstractMethod
description: A concrete class does not implement an abstract method from its parent.
sidebar:
  order: 1
---

A concrete class does not implement an abstract method from its parent.

## Example

```php
<?php
abstract class Shape {
    abstract public function area(): float;
}

class Circle extends Shape {} // area() not implemented
```

## How to fix

Implement the abstract method in the concrete class.
