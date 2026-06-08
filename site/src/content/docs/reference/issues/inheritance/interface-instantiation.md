---
title: InterfaceInstantiation
code: MIR0709
description: Instantiating an interface with `new`.
sidebar:
  hidden: true
  order: 709
---

An interface is used with `new` to create an instance. Interfaces cannot be instantiated; only
concrete classes that implement the interface can be.

## Example

```php
<?php
interface Serializable {
    public function serialize(): string;
}

$obj = new Serializable(); // cannot instantiate interface
```

## How to fix

Instantiate a concrete class that implements the interface instead:

```php
<?php
class JsonSerializable implements Serializable {
    public function serialize(): string {
        return '{}';
    }
}

$obj = new JsonSerializable();
```
