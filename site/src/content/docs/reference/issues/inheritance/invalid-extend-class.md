---
title: InvalidExtendClass
code: MIR0704
description: A class extends a `final` class or a class annotated `@final`.
sidebar:
  hidden: true
  order: 704
---

A class extends a class declared as `final` or annotated with `@final` in its docblock.

This issue was previously named `FinalClassExtended` (same code MIR0704).

## Example

```php
<?php
final class Singleton {}

class MyClass extends Singleton {} // cannot extend final class
```

`@final` docblock annotations are also respected:

```php
<?php
/** @final */
class Base {}

class Child extends Base {} // cannot extend @final class
```

## How to fix

Remove the `final` keyword (or `@final` annotation) from the parent class, or do not extend it.
