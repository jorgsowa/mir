---
title: InvalidScope
code: MIR0001
description: "`$this` used outside a class or inside a static method."
sidebar:
  hidden: true
  order: 1
---

`$this` is used in a context where it is not available: either outside any class, or inside a `static` method where no object instance exists.

## Example

```php
<?php
class Counter {
    private int $count = 0;

    public static function reset(): void {
        $this->count = 0; // $this is not available in a static method
    }
}
```

## How to fix

Remove the use of `$this`, change the method to non-static, or use a static property instead.

```php
<?php
class Counter {
    private static int $count = 0;

    public static function reset(): void {
        self::$count = 0;
    }
}
```
