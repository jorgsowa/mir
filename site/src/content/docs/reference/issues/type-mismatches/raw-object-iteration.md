---
title: RawObjectIteration
code: MIR0222
description: "An object that does not implement `Traversable` is iterated (e.g. via `foreach` or `yield from`)."
sidebar:
  hidden: true
  order: 222
---

An object that does not implement `Traversable` is iterated (e.g. via `foreach` or `yield from`).

## Example

```php
<?php
foreach (new stdClass() as $v) {} // RawObjectIteration
```

## How to fix

Iterate an array, or implement `IteratorAggregate`/`Iterator`.
