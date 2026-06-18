---
title: PossiblyRawObjectIteration
code: MIR0223
description: "A value that *might* be a non-iterable object is iterated."
sidebar:
  hidden: true
  order: 223
---

A value that *might* be a non-iterable object is iterated.

## Example

```php
<?php
/** @param object|array $x */
function f($x){ foreach ($x as $v) {} }
```

## How to fix

Narrow the type so only `Traversable`/arrays remain before iterating.
