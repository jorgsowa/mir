---
title: InaccessibleClassConstant
code: MIR0011
description: Access to a private or protected class constant from an incompatible scope.
sidebar:
  hidden: true
  order: 11
---

A `private` or `protected` class constant is accessed from a scope that does not have permission
to read it. Private constants are only accessible within the declaring class; protected constants
are accessible within the declaring class and its subclasses.

## Example

```php
<?php
class Config {
    private const SECRET = 'abc123';
}

echo Config::SECRET; // cannot access private constant from outside the class
```

## How to fix

Change the constant's visibility to `public`, or access it only from within the declaring class
(or a subclass, for `protected`).
