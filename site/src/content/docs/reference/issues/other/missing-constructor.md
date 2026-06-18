---
title: MissingConstructor
code: MIR1507
description: "A class has non-nullable, uninitialized typed properties but no constructor to initialize them."
sidebar:
  hidden: true
  order: 1507
---

A class has non-nullable, uninitialized typed properties but no constructor to initialize them.

## Example

```php
<?php
class C { public int $n; } // MissingConstructor
```

## How to fix

Add a constructor that initializes the properties, or give them defaults.
