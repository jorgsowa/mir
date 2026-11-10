---
title: NoDiscardReturnIgnored
code: MIR1514
description: The result of a `#[\NoDiscard]` function or method is discarded.
sidebar:
  hidden: true
  order: 1514
---

A function or method marked `#[\NoDiscard]` is called as a statement on its own, so its return
value is thrown away. From PHP 8.5 this emits a warning. Builtins such as
`DateTimeImmutable::add()` carry the attribute because calling them only for their side effects
is a bug.

## Example

```php
<?php
function shift(DateTimeImmutable $date): DateTimeImmutable {
    $date->add(new DateInterval('P1D')); // returns a new object; $date is unchanged
    return $date;
}
```

## How to fix

Use the return value, or cast the call to `(void)` when discarding it is intended:

```php
<?php
function shift(DateTimeImmutable $date): DateTimeImmutable {
    return $date->add(new DateInterval('P1D'));
}
```
