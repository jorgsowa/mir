---
title: DynamicPropertyCreation
code: MIR1014
description: A write creates an undeclared property, which PHP deprecates from 8.2.
sidebar:
  hidden: true
  order: 10
---

Assigning a property the class does not declare creates it at runtime. PHP 8.2 deprecates this (`Creation of dynamic property A::$x is deprecated`) unless the class or a parent has `__set` or `#[\AllowDynamicProperties]`, or extends `stdClass`. A later write to the same property in the same scope is not reported again. On enums and readonly classes the write throws instead: [InvalidDynamicProperty](../../undefined/invalid-dynamic-property/).

## Example

```php
<?php
class User {}

$u = new User();
$u->name = 'Ada';
```

## How to fix

Declare the property, or mark the class `#[\AllowDynamicProperties]` when dynamic properties are intended.
