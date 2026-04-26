---
title: UndefinedConstant
description: A reference is made to a constant that has not been defined.
sidebar:
  order: 7
---

A reference is made to a constant that has not been defined.

## Example

```php
<?php
echo MAX_RETRIES; // constant not defined
```

## How to fix

Define the constant with `define()` or `const`, or fix the spelling.
