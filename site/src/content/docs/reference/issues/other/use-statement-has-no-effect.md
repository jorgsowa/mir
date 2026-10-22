---
title: UseStatementHasNoEffect
code: MIR1511
description: A use statement imports a single-segment name without an alias outside any namespace.
sidebar:
  hidden: true
  order: 10
---

A `use` of a name without a namespace separator and without an alias, outside any namespace, imports the name under itself. PHP warns that it has no effect.

## Example

```php
<?php
use Foo;
```

## How to fix

Remove the statement, or move it into the namespace that needs the import.
