---
title: TooFewArguments
description: A call provides fewer arguments than the function requires.
sidebar:
  order: 3
---

A call provides fewer arguments than the function requires.

## Example

```php
<?php
function add(int $a, int $b): int { return $a + $b; }

add(1); // missing second argument
```

## How to fix

Provide all required arguments, or make the missing parameter optional.
