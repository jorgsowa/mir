---
title: EnumInstantiation
code: MIR0719
description: Instantiating an enum with `new`.
sidebar:
  hidden: true
  order: 719
---

An enum is used with `new`. Enum instances are its cases; PHP throws `Cannot instantiate enum` for `new Suit()`, also for `new self()` or `new static()` inside the enum.

## Example

```php
<?php
enum Suit {
    case Hearts;
    case Spades;
}

$suit = new Suit(); // cannot instantiate enum
```

## How to fix

Use a case, or look one up by value on a backed enum:

```php
<?php
$suit = Suit::Hearts;
```
