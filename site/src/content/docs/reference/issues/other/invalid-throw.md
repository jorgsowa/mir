---
title: InvalidThrow
description: A value that does not implement Throwable is thrown.
sidebar:
  order: 4
---

A value that does not implement `Throwable` is thrown.

## Example

```php
<?php
throw 'error message'; // string does not implement Throwable
```

## How to fix

Throw an instance of `Exception` or another `Throwable` class.

```php
<?php
throw new \RuntimeException('error message');
```
