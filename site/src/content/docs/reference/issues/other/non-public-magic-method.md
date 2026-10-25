---
title: NonPublicMagicMethod
code: MIR1513
description: A magic method PHP calls from outside the class is declared protected or private.
sidebar:
  hidden: true
  order: 10
---

PHP calls `__get`, `__set`, `__isset`, `__unset`, `__call`, `__callStatic`, `__toString`, `__invoke`, `__debugInfo`, `__serialize`, `__unserialize`, `__sleep`, `__wakeup` and `__set_state` from outside the class, so it warns when one is not public. The constructor, destructor and `__clone` may be non-public.

## Example

```php
<?php
class Config {
    private function __get(string $name): mixed { return null; }
}
```

## How to fix

Make the method `public`.
