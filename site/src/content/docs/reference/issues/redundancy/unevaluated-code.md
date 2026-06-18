---
title: UnevaluatedCode
code: MIR0407
description: "A `switch`/`match` arm can never be reached given the subject's inferred type (e.g. `case \"int\"` for `gettype()`, which returns `\"integer\"`)."
sidebar:
  hidden: true
  order: 407
---

A `switch`/`match` arm can never be reached given the subject's inferred type (e.g. `case "int"` for `gettype()`, which returns `"integer"`).

## Example

```php
<?php
switch (gettype($x)) { case 'int': break; } // never matches
```

## How to fix

Use the value the subject can actually take (`'integer'`).
