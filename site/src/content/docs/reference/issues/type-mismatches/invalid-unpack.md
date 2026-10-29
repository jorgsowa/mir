---
title: InvalidUnpack
code: MIR0231
description: A scalar or `null` unpacked into a call with `...`.
sidebar:
  hidden: true
  order: 231
---

An argument unpacked with `...` is a scalar or `null`. Only arrays and `Traversable` objects
can be unpacked; PHP throws `TypeError: Only arrays and Traversables can be unpacked, string given`.
Unpacking a literal into an array (`[...1]`) is a compile error reported as `CompileError`.

## Example

```php
<?php
function sum(int ...$numbers): int {
    return array_sum($numbers);
}

function total(string $csv): int {
    return sum(...$csv); // a string is not iterable
}
```

## How to fix

Unpack an array or iterable:

```php
<?php
function total(string $csv): int {
    return sum(...array_map('intval', explode(',', $csv)));
}
```
