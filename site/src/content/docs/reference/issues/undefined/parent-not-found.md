---
title: ParentNotFound
code: MIR0010
description: Use of `parent::` in a class that has no parent.
sidebar:
  hidden: true
  order: 10
---

`parent::` is used inside a class that does not extend any other class. There is no parent to
resolve the reference to.

## Example

```php
<?php
class Foo {
    public function bar(): string {
        return parent::bar(); // Foo has no parent class
    }
}
```

## How to fix

Add an `extends` clause to the class, or remove the `parent::` call and implement the logic
directly.
