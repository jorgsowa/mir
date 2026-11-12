---
title: InvalidScalarArgument
code: MIR0238
description: "A scalar argument of another scalar type is passed in a file without strict_types."
sidebar:
  hidden: true
  order: 238
---

Without `declare(strict_types=1)`, PHP converts a scalar argument to the parameter's scalar type
when it can, so the call works for some values (a numeric string for an `int`) and throws a
`TypeError` or loses data for others.

## Example

```php
<?php
function page(int $n): int { return $n; }
function f(string $input): int { return page($input); }
```

## How to fix

Convert the value explicitly (`(int) $input`) after validating it, or change the parameter type.
