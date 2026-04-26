---
title: UnimplementedInterfaceMethod
description: A class does not implement a method required by an interface it declares.
sidebar:
  order: 2
---

A class does not implement a method required by an interface it declares.

## Example

```php
<?php
interface Logger {
    public function log(string $message): void;
}

class NullLogger implements Logger {} // log() not implemented
```

## How to fix

Implement all required interface methods.
