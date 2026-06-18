---
title: UndefinedDocblockClass
code: MIR1505
description: "A class referenced only in a docblock does not exist."
sidebar:
  hidden: true
  order: 1505
---

A class referenced only in a docblock does not exist.

## Example

```php
<?php
/** @return Nonexistent */
function f() {} // UndefinedDocblockClass
```

## How to fix

Import or define the class, or fix the name in the docblock.
