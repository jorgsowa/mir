---
title: InaccessibleProperty
code: MIR0014
description: Access to a private or protected property from an incompatible scope.
sidebar:
  hidden: true
  order: 14
---

A `private` or `protected` property is read from a scope that does not have permission to read it.
Private properties are only accessible within the declaring class; protected properties are
accessible within the declaring class and its subclasses. This applies to both instance
(`$obj->prop`) and static (`Class::$prop`) property access.

## Example

```php
<?php
class Vault {
    private string $secret = 'classified';
}

echo (new Vault())->secret; // cannot access private property from outside the class
```

## How to fix

Change the property's visibility to `public`, add an accessor method, or access it only from
within the declaring class (or a subclass, for `protected`).
