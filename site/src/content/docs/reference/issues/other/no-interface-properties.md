---
title: NoInterfaceProperties
code: MIR1504
description: "A property is accessed on an interface that seals properties and does not declare it via `@property`."
sidebar:
  hidden: true
  order: 1504
---

A property is accessed on an interface that seals properties and does not declare it via `@property`.

## Example

```php
<?php
interface I {}
function f(I $i){ return $i->x; } // NoInterfaceProperties
```

## How to fix

Declare the property with `@property`, or access it on the implementing class.
