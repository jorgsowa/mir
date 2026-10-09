---
title: PossiblyUndefinedMethod
code: MIR0015
description: A method is called on a union where only some members declare it.
sidebar:
  hidden: true
  order: 15
---

A method is called on a union type where at least one member declares the method and another does not. Reported as info; a method missing from every member is [UndefinedMethod](../undefined-method/).

## Example

```php
<?php
class Reader { public function read(): void {} }
class Writer { public function write(): void {} }

function run(Reader|Writer $io): void {
    $io->read(); // Writer has no read method
}
```

## How to fix

Narrow the receiver (`instanceof`) before the call, or fix the declared type.
