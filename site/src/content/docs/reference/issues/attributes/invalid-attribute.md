---
title: InvalidAttribute
code: MIR1600
description: "`#[Attribute]` usage violates target restrictions or argument constraints."
sidebar:
  hidden: true
  order: 1600
---

An attribute is applied to a target (class, method, property, parameter, etc.) that the
attribute does not allow, or the arguments passed to the attribute do not match its constructor
signature. PHP validates attribute targets at runtime; mir reports these violations statically.

## Example

```php
<?php
#[\Attribute(\Attribute::TARGET_METHOD)]
class MethodOnly {}

#[MethodOnly] // applied to a class, but only METHOD is allowed
class MyClass {}
```

## How to fix

Apply the attribute only to the targets listed in its `\Attribute` declaration, and ensure the
arguments match the attribute class constructor.
