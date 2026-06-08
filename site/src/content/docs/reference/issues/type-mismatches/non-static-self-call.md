---
title: NonStaticSelfCall
code: MIR0216
description: "`self::method()` called when the method is not static."
sidebar:
  hidden: true
  order: 216
---

`self::method()` is used to call an instance method from within the same class, but the target
method is not declared `static`. This is only valid inside a static context.

## Example

```php
<?php
class MyClass {
    public function helper(): string {
        return 'help';
    }

    public static function run(): string {
        return self::helper(); // helper() is not static
    }
}
```

## How to fix

Declare `helper()` as `static`, or call it on an instance instead:

```php
<?php
public static function run(): string {
    $instance = new self();
    return $instance->helper();
}
```
