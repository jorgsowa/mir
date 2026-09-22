---
title: NotAnInterface
code: MIR0228
description: A class that is expected to be an interface is not an interface.
sidebar:
  hidden: true
  order: 28
---

A type that is expected to be an interface is actually a class or another type. This commonly occurs when using interfaces for dependency injection or type hints where only interfaces should be used.

## Example

```php
<?php
class Service {}

// Error: Service is a class, not an interface
interface Repository extends Service {
    function find(int $id);
}
```

## How to fix

Ensure the type is actually an interface:

```php
<?php
interface ServiceInterface {
    function execute();
}

interface Repository extends ServiceInterface {
    function find(int $id);
}
```
