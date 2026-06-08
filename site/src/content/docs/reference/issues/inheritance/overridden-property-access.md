---
title: OverriddenPropertyAccess
code: MIR0710
description: An overriding property reduces the visibility of the parent property.
sidebar:
  hidden: true
  order: 710
---

A child class declares a property with the same name as a parent property but with a more
restrictive visibility. PHP does not allow reducing the visibility of a property when overriding.

## Example

```php
<?php
class Base {
    public string $name = '';
}

class Child extends Base {
    protected string $name = ''; // reduces visibility from public to protected
}
```

## How to fix

Keep the visibility at least as permissive as the parent property, or rename the property in
the child class.
