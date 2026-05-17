---
title: InvalidCast
code: MIR0207
description: A cast from an array or object to a scalar type always produces a meaningless result.
sidebar:
  hidden: true
  order: 8
---

A cast from an array or object to a scalar type (`int`, `float`, or `string`) always produces a meaningless result. For example, casting an array to `int` always gives `0` or `1`; casting an array to `string` always gives `"Array"` and triggers a notice in PHP 8.

## Example

```php
<?php
/** @param array $data */
function process(array $data): int {
    return (int) $data; // always 0 (empty) or 1 (non-empty), never the intended value
}
```

## How to fix

Use an appropriate function instead of a cast (e.g. `count()` for length, `implode()` for string, `array_sum()` for numeric aggregation).

```php
<?php
/** @param array $data */
function process(array $data): int {
    return count($data);
}
```
