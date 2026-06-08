---
title: DirectConstructorCall
code: MIR0217
description: Direct call to `__construct()` outside of a constructor chain.
sidebar:
  hidden: true
  order: 217
---

`__construct()` is called directly on an object (e.g., `$obj->__construct(...)`) outside of the
accepted constructor-chaining context (`parent::__construct(...)`). This bypasses normal object
initialisation semantics and is almost always a mistake.

## Example

```php
<?php
class Point {
    public function __construct(
        public float $x,
        public float $y,
    ) {}
}

$p = new Point(1.0, 2.0);
$p->__construct(3.0, 4.0); // direct constructor call
```

## How to fix

Do not call `__construct()` directly after object creation. If you need to re-initialise an
object, extract the initialisation logic into a separate method.
