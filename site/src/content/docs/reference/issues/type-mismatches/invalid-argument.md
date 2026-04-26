---
title: InvalidArgument
description: An argument's type does not match the parameter's declared type.
sidebar:
  order: 2
---

An argument's type does not match the parameter's declared type.

## Example

```php
<?php
function double(int $n): int { return $n * 2; }

double('five'); // string passed, int expected
```

## How to fix

Pass a value of the correct type or widen the parameter type.
