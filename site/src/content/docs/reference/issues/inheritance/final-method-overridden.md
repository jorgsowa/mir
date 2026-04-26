---
title: FinalMethodOverridden
description: A subclass overrides a method declared as final in the parent.
sidebar:
  order: 6
---

A subclass overrides a method declared as `final` in the parent.

## Example

```php
<?php
class Base {
    final public function id(): int { return 1; }
}

class Child extends Base {
    public function id(): int { return 2; } // cannot override final method
}
```

## How to fix

Remove the `final` modifier from the parent method or do not override it.
