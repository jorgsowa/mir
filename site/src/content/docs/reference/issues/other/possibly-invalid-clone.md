---
title: PossiblyInvalidClone
code: MIR1206
description: Cloning a value whose type might not be an object.
sidebar:
  hidden: true
  order: 1206
---

The `clone` keyword is used on a value with a union type where at least one member is not an
object. At runtime, if the non-object branch is taken, a fatal error will occur.

## Example

```php
<?php
function duplicate(object|null $value): ?object {
    return clone $value; // $value could be null
}
```

## How to fix

Narrow the type before cloning, or handle the non-object case explicitly:

```php
<?php
function duplicate(object|null $value): ?object {
    return $value !== null ? clone $value : null;
}
```
