---
title: InvalidPropertyFetch
code: MIR0218
description: Property access on a non-object type.
sidebar:
  hidden: true
  order: 218
---

A property is accessed with `->prop` on a value that is not an object. At runtime this will
produce a fatal error.

## Example

```php
<?php
function getLabel(string $item): string {
    return $item->label; // string is not an object
}
```

## How to fix

Ensure the value is an object before accessing its properties, or fix the type annotation:

```php
<?php
function getLabel(Item $item): string {
    return $item->label;
}
```
