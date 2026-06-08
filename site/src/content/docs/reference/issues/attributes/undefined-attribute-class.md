---
title: UndefinedAttributeClass
code: MIR1601
description: Attribute class referenced with `#[...]` does not exist.
sidebar:
  hidden: true
  order: 1601
---

An attribute syntax `#[ClassName]` references a class that cannot be found in the project or
its dependencies. This will produce a fatal error when PHP attempts to resolve the attribute at
runtime.

## Example

```php
<?php
#[NonExistentAttribute] // class NonExistentAttribute not found
class MyClass {}
```

## How to fix

Ensure the attribute class is defined and autoloaded, or add the correct `use` import for a
namespaced attribute class.
