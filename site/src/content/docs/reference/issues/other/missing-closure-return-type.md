---
title: MissingClosureReturnType
code: MIR1105
description: "A closure or arrow function has no declared return type."
sidebar:
  hidden: true
  order: 1105
---

A closure or arrow function has no declared return type.

## Example

```php
<?php
$f = function () { return 1; }; // MissingClosureReturnType
```

## How to fix

Add a return type: `function (): int { ... }`.
