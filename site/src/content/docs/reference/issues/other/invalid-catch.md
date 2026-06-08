---
title: InvalidCatch
code: MIR1503
description: Catching a type that is not `Throwable`.
sidebar:
  hidden: true
  order: 1503
---

A `catch` block specifies a type that does not implement the `Throwable` interface. Only
`Throwable` and its descendants (`Exception`, `Error`, and their subclasses) can be caught.

## Example

```php
<?php
class NotAnException {}

try {
    doSomething();
} catch (NotAnException $e) { // NotAnException is not Throwable
    // unreachable
}
```

## How to fix

Only catch classes that implement `Throwable`. If `NotAnException` is meant to represent an
error condition, make it extend `\Exception` or `\RuntimeException`.
