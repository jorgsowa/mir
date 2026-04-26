---
title: InvalidPropertyAssignment
description: A value assigned to a property is incompatible with the property's declared type.
sidebar:
  order: 7
---

A value assigned to a property is incompatible with the property's declared type.

## Example

```php
<?php
class User {
    public int $age;
}

$user = new User();
$user->age = 'twenty'; // string, int expected
```

## How to fix

Assign a value of the correct type or update the property type declaration.
