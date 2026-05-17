---
title: ImplicitToStringCast
code: MIR1501
description: "An object without `__toString` or `Stringable` is implicitly coerced to a string (e.g. in string concatenation)."
sidebar:
  hidden: true
  order: 1501
---

An object that does not implement `__toString()` or `Stringable` is implicitly coerced to a string (e.g. in string concatenation or an interpolated string).

## Example

```php
<?php
class Point {
    public function __construct(public int $x, public int $y) {}
}

$p = new Point(1, 2);
echo "Position: " . $p; // Point has no __toString
```

## How to fix

Implement `__toString()` on the class, or cast explicitly with `(string)` only after implementing it.

```php
<?php
class Point {
    public function __construct(public int $x, public int $y) {}

    public function __toString(): string {
        return "({$this->x}, {$this->y})";
    }
}
```
