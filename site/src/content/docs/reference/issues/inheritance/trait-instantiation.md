---
title: TraitInstantiation
code: MIR0720
description: Instantiating a trait with `new`.
sidebar:
  hidden: true
  order: 720
---

A trait is used with `new`. A trait only supplies members to the classes that `use` it; PHP throws `Cannot instantiate trait` for `new Loggable()`. Inside a trait, `new self()` and `new static()` name the using class and are fine.

## Example

```php
<?php
trait Loggable {
    public function log(string $message): void {}
}

$logger = new Loggable(); // cannot instantiate trait
```

## How to fix

Instantiate a class that uses the trait:

```php
<?php
final class Logger {
    use Loggable;
}

$logger = new Logger();
```
