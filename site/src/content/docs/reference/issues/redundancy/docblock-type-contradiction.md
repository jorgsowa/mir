---
title: DocblockTypeContradiction
code: MIR0406
description: "A docblock-declared type makes a later assertion or comparison impossible (e.g. `assert($a < 4)` on `@param int<5, max> $a`)."
sidebar:
  hidden: true
  order: 406
---

A docblock-declared type makes a later assertion or comparison impossible (e.g. `assert($a < 4)` on `@param int<5, max> $a`).

## Example

```php
<?php
/** @param int<5, max> $a */
function f(int $a){ assert($a < 4); }
```

## How to fix

Correct the docblock or remove the impossible assertion.
