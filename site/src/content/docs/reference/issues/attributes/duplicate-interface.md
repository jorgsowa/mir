---
title: DuplicateInterface
code: MIR1603
description: "An interface with the same name is declared more than once."
sidebar:
  hidden: true
  order: 1603
---

An interface with the same name is declared more than once.

## Example

```php
<?php
interface I {}
interface I {} // DuplicateInterface
```

## How to fix

Remove the duplicate declaration.
