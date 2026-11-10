---
title: NonInvariantDocblockPropertyType
code: MIR0721
description: A redeclared property's docblock type differs from the parent's.
sidebar:
  hidden: true
  order: 721
---

A child class redeclares a property with a `@var` type that is not the same as the parent's.
PHP enforces invariance only for native property types, but code written against the parent
still reads and writes the property: a wider type lets a read return a value the parent never
promised, and a narrower one lets a write through the parent store a value the child rejects.

## Example

```php
<?php
interface Animal {}
interface Dog extends Animal {}

class Kennel {
    /** @var Animal|null */
    public $resident = null;
}

class DogKennel extends Kennel {
    /** @var Dog|null */
    public $resident = null; // narrower than the parent's Animal|null
}
```

## How to fix

Keep the parent's type, or make the parent generic and bind the type with `@extends`:

```php
<?php
/** @template T of Animal */
class Kennel {
    /** @var T|null */
    public $resident = null;
}

/** @extends Kennel<Dog> */
class DogKennel extends Kennel {}
```
