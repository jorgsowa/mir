---
title: TooManyArguments
code: MIR0203
description: A call provides more arguments than the function accepts.
sidebar:
  hidden: true
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
