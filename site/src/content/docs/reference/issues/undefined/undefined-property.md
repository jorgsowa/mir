---
title: UndefinedProperty
description: A property is accessed that is not declared on the class.
sidebar:
  order: 6
---

A property is accessed that is not declared on the class.

## Example

```php
<?php
class Order {
    public int $id;
}

$order = new Order();
echo $order->total; // property not declared
```

## How to fix

Declare the property on the class or fix the property name.
