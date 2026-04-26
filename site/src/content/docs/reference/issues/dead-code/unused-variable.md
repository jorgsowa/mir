---
title: UnusedVariable
description: A variable is assigned but never read.
sidebar:
  order: 1
---

A variable is assigned but never read.

## Example

```php
<?php
function compute(): int {
    $temp = expensiveCall(); // $temp is never used
    return 42;
}
```

## How to fix

Remove the assignment or use the variable.
