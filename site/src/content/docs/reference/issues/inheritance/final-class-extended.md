---
title: FinalClassExtended
code: MIR0704
description: A class extends a class declared as final.
sidebar:
  hidden: true
  order: 5
---

A class extends a class declared as `final`.

## Example

```php
<?php
final class Singleton {}

class MyClass extends Singleton {} // cannot extend final class
```

## How to fix

Remove the `final` keyword from the parent class or do not extend it.
