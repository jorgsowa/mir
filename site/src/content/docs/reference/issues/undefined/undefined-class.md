---
title: UndefinedClass
description: A reference is made to a class, interface, or trait that does not exist.
sidebar:
  order: 5
---

A reference is made to a class, interface, or trait that does not exist.

## Example

```php
<?php
$obj = new PaymentGateway(); // class not found
```

## How to fix

Add the missing `use` statement, check the namespace, or define the class.
