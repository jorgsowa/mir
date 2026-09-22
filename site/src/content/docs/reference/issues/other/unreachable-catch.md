---
title: UnreachableCatch
code: MIR1508
description: A catch block is unreachable because a more general exception type is caught before a more specific one.
sidebar:
  hidden: true
  order: 10
---

A catch block is unreachable because a more general exception type is caught before a more specific one. Exception types should be ordered from most specific to most general.

## Example

```php
<?php
try {
    // Some code
} catch (Exception $e) {
    // General catch
} catch (InvalidArgumentException $e) {
    // This is unreachable - Exception already caught
}
```

## How to fix

Order catch blocks from most specific to most general:

```php
<?php
try {
    // Some code
} catch (InvalidArgumentException $e) {
    // Specific catch first
} catch (Exception $e) {
    // General catch last
}
```
