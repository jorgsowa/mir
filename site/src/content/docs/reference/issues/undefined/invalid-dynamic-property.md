---
title: InvalidDynamicProperty
code: MIR0016
description: A write creates an undeclared property on an enum or readonly class, which throws.
sidebar:
  hidden: true
  order: 16
---

Enums and readonly classes cannot hold dynamic properties: writing a property they do not declare throws `Error: Cannot create dynamic property E::$x`. On other classes the same write is [DynamicPropertyCreation](../../other/dynamic-property-creation/).

## Example

```php
<?php
readonly class Point {
    public function __construct(public int $x) {}
}

$p = new Point(1);
$p->y = 2;
```

## How to fix

Declare the property, or keep the extra data elsewhere.
