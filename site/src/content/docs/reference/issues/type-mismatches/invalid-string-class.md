---
title: InvalidStringClass
code: MIR0211
description: A string used as a class name does not resolve to a known class.
sidebar:
  hidden: true
  order: 211
---

A string variable is used as a class name (e.g., via `new $className` or `$className::method()`)
but mir cannot determine that it resolves to a known class. This usually occurs when a plain
`string` type (not a `class-string`) is used in a dynamic class instantiation.

## Example

```php
<?php
function make(string $className): object {
    return new $className(); // $className is a plain string, not a known class
}
```

## How to fix

Narrow the type to a `class-string` or `class-string<T>` annotation so mir knows the string
represents a valid class name:

```php
<?php
/**
 * @param class-string $className
 */
function make(string $className): object {
    return new $className();
}
```
