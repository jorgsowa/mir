---
title: DuplicateEnumCaseValue
code: MIR0718
description: Two cases of a backed enum have the same value.
sidebar:
  hidden: true
  order: 18
---

Every case of a backed enum needs its own value. PHP stops with a fatal error when two cases share one; the later case is reported.

## Example

```php
<?php
enum Status: int {
    case Active = 1;
    case Enabled = 1; // Error: duplicate of Active
}
```

## How to fix

Give the case its own value, or drop it and use the existing case:

```php
<?php
enum Status: int {
    case Active = 1;
    case Enabled = 2;
}
```
