---
title: UndefinedFunction
description: A call targets a function that does not exist in the codebase or stubs.
sidebar:
  order: 3
---

A call targets a function that does not exist in the codebase or stubs.

## Example

```php
<?php
$result = computeHash('data'); // function not defined anywhere
```

## How to fix

Define the function, add the missing `use` import, or fix the spelling.
