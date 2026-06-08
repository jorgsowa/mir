---
title: InvalidOverride
code: MIR0708
description: Method declared `#[Override]` does not actually override a parent method.
sidebar:
  hidden: true
  order: 708
---

A method is annotated with the `#[Override]` attribute (introduced in PHP 8.3), signalling that
it is intended to override a method from a parent class or interface. However, no matching method
exists in any ancestor, so the annotation is incorrect.

## Example

```php
<?php
class Base {}

class Child extends Base {
    #[Override]
    public function process(): void {} // Base has no process() method
}
```

## How to fix

Either remove the `#[Override]` attribute if the method is not intended to override anything, or
fix the method name / add the method to the parent class.
