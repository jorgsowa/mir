---
title: InvalidNamedArguments
code: MIR0224
description: "Named arguments passed to a function or method tagged `@no-named-arguments`."
sidebar:
  hidden: true
  order: 224
---

Named arguments passed to a function or method tagged `@no-named-arguments`.

## Example

```php
<?php
/** @no-named-arguments */
function f(int $a){}
f(a: 1); // InvalidNamedArguments
```

## How to fix

Pass the arguments positionally.
