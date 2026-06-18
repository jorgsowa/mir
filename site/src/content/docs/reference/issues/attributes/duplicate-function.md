---
title: DuplicateFunction
code: MIR1606
description: "A function with the same name is declared more than once."
sidebar:
  hidden: true
  order: 1606
---

A function with the same name is declared more than once.

## Example

```php
<?php
function f(){}
function f(){} // DuplicateFunction
```

## How to fix

Rename or remove the duplicate declaration.
