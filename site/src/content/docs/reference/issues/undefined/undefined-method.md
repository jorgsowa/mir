---
title: UndefinedMethod
description: A method is called on a type that does not declare that method.
sidebar:
  order: 4
---

A method is called on a type that does not declare that method.

## Example

```php
<?php
class User {}

$user = new User();
$user->getName(); // User has no getName method
```

## How to fix

Add the method to the class, fix the method name, or correct the type of the variable.
