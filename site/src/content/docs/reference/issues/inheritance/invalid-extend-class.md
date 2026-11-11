---
title: InvalidExtendClass
code: MIR0704
description: A class extends a `final` class, a sealed type it is not listed in, or misses a class an interface requires.
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

A type that lists its subtypes with `@phpstan-sealed` (or `@psalm-inheritors`) rejects any other class, and an interface with `@phpstan-require-extends` rejects an implementing class that does not extend the named class:

```php
<?php
/** @phpstan-sealed Circle */
interface Shape {}

final class Square implements Shape {} // not in the sealed list

class Model {}

/** @phpstan-require-extends Model */
interface HasTable {}

final class Report implements HasTable {} // must extend Model
```

## How to fix

Remove the `final` keyword (or `@final` annotation) from the parent class, or do not extend it. For a sealed type, add the class to its list; for `@phpstan-require-extends`, extend the required class.
