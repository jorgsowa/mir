---
title: UnusedForeachValue
code: MIR0506
description: Foreach value variable assigned but never read.
sidebar:
  hidden: true
  order: 506
---

The value variable in a `foreach` loop is assigned but never used in the loop body. This is
typically a sign that the variable was accidentally omitted from the logic, or that only the key
was needed.

## Example

```php
<?php
$items = ['a', 'b', 'c'];
foreach ($items as $key => $value) {
    echo $key; // $value is never used
}
```

## How to fix

If you only need the key, use the single-variable form of `foreach`:

```php
<?php
foreach ($items as $key) {
    echo $key;
}
```

If you need the value, make sure to use `$value` in the loop body.
