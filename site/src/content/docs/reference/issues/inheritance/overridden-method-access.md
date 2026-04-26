---
title: OverriddenMethodAccess
description: An overriding method reduces the visibility of the parent method.
sidebar:
  order: 4
---

An overriding method reduces the visibility of the parent method.

## Example

```php
<?php
class Base {
    public function render(): void {}
}

class Child extends Base {
    protected function render(): void {} // reducing public to protected
}
```

## How to fix

Keep the same or wider visibility in the overriding method.
