---
title: ReadonlyClassExtendsMismatch
code: MIR0716
description: A readonly class extends a non-readonly class, which is not allowed.
sidebar:
  hidden: true
  order: 16
---

A readonly class extends a non-readonly class. In PHP, a readonly class can only extend another readonly class.

## Example

```php
<?php
class Base {
    public string $name;
}

readonly class Child extends Base { // Error: cannot extend non-readonly class
    public string $value;
}
```

## How to fix

Make the parent class readonly as well:

```php
<?php
readonly class Base {
    public string $name;
}

readonly class Child extends Base {
    public string $value;
}
```
