---
title: DuplicateTrait
code: MIR1604
description: "A trait with the same name is declared more than once."
sidebar:
  hidden: true
  order: 1604
---

A trait with the same name is declared more than once.

## Example

```php
<?php
trait T {}
trait T {} // DuplicateTrait
```

## How to fix

Remove the duplicate declaration.
