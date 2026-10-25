---
title: ImplicitlyNullableParameter
code: MIR1013
description: A typed parameter is nullable only through its null default, which PHP deprecates from 8.4.
sidebar:
  hidden: true
  order: 10
---

`int $a = null` accepts `null` although its type does not say so. PHP 8.4 deprecates this; the type must spell out `null` (`?int`, `int|null`). A promoted constructor parameter written this way is a compile error on every version (`Cannot use null as default value for parameter $a of type int`).

## Example

```php
<?php
function find(string $id = null): void {}
```

## How to fix

Write the type as nullable: `?string $id = null`.
