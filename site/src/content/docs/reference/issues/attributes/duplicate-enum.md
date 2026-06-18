---
title: DuplicateEnum
code: MIR1605
description: "An enum with the same name is declared more than once."
sidebar:
  hidden: true
  order: 1605
---

An enum with the same name is declared more than once.

## Example

```php
<?php
enum E {}
enum E {} // DuplicateEnum
```

## How to fix

Remove the duplicate declaration.
