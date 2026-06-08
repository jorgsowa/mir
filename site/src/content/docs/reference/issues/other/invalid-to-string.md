---
title: InvalidToString
code: MIR1207
description: Using an object in a string context without a `__toString` method.
sidebar:
  hidden: true
  order: 1207
---

An object is used where a string is expected (e.g., string concatenation, `echo`, or string
interpolation), but the object's class does not implement the `__toString` method. This produces
a fatal error in PHP.

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
echo "Point: " . $p; // Point has no __toString method
```

## How to fix

Add a `__toString` method to the class, or explicitly convert the object to a string before
use:

```php
<?php
class Point {
    public function __construct(
        public float $x,
        public float $y,
    ) {}

    public function __toString(): string {
        return "({$this->x}, {$this->y})";
    }
}
```
