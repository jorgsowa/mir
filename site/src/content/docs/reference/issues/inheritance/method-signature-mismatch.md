---
title: MethodSignatureMismatch
description: An overriding method has a signature incompatible with the parent's.
sidebar:
  order: 3
---

An overriding method has a signature incompatible with the parent's.

## Example

```php
<?php
class Base {
    public function process(string $input): string { return $input; }
}

class Child extends Base {
    public function process(int $input): string { return (string) $input; } // parameter type changed
}
```

## How to fix

Make the overriding method's signature compatible with the parent's (contravariant parameters, covariant return types).
