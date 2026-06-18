---
title: UnusedClass
code: MIR0507
description: "A class is declared but never referenced anywhere in the analyzed code."
sidebar:
  hidden: true
  order: 507
---

A class is declared but never referenced anywhere in the analyzed code.

## Example

```php
<?php
class Helper {} // never used
```

## How to fix

Remove the class, or reference it.
