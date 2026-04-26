---
title: TooManyArguments
description: A call provides more arguments than the function accepts.
sidebar:
  order: 4
---

A call provides more arguments than the function accepts.

## Example

```php
<?php
function greet(string $name): string { return "Hello, $name"; }

greet('Alice', 'extra'); // only one parameter accepted
```

## How to fix

Remove the extra arguments or add more parameters to the function.
