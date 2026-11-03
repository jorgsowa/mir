---
title: UnusedConstant
code: MIR0509
description: A private class or enum constant is never used.
sidebar:
  hidden: true
  order: 509
---

A private class or enum constant is never used.

## Example

```php
<?php
class Config {
    private const TIMEOUT = 30; // never used
}
```

## How to fix

Remove the constant or use it.
