---
title: MixedReturnStatement
code: MIR1212
description: "A `mixed` value is returned from a function with a typed return."
sidebar:
  hidden: true
  order: 1212
---

A `mixed` value is returned from a function with a typed return.

## Example

```php
<?php
function f(): int { /** @var mixed $x */ return $x; }
```

## How to fix

Narrow the value to the declared return type.
