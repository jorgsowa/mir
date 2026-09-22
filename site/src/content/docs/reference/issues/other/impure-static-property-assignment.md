---
title: ImpureStaticPropertyAssignment
code: MIR1706
description: An impure function assigns to a static property, causing side effects.
sidebar:
  hidden: true
  order: 6
---

An impure function assigns to a static property, which introduces side effects that can make code harder to reason about and test.

## Example

```php
<?php
class Counter {
    public static $count = 0;
    
    public static function increment(): void {
        self::$count++; // Static property modification in function
    }
}
```

## How to fix

Consider using instance properties or dependency injection instead of static properties:

```php
<?php
class Counter {
    private int $count = 0;
    
    public function increment(): void {
        $this->count++;
    }
    
    public function getCount(): int {
        return $this->count;
    }
}
```
