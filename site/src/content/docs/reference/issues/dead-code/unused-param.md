---
title: UnusedParam
description: A function parameter is never referenced in the function body.
sidebar:
  order: 2
---

A function parameter is never referenced in the function body.

## Example

```php
<?php
function greet(string $name, string $title): string {
    return "Hello!"; // $name and $title are ignored
}
```

## How to fix

Remove the unused parameter or use it in the function body.
