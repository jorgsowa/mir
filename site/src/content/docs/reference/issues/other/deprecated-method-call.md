---
title: DeprecatedMethodCall
code: MIR1001
description: "A method marked `@deprecated` is called on an instance."
sidebar:
  hidden: true
  order: 1001
---

A method marked `@deprecated` is called on an instance or via a static call.

## Example

```php
<?php
class Api {
    /** @deprecated Use newMethod() instead */
    public function oldMethod(): void {}

    public function newMethod(): void {}
}

$api = new Api();
$api->oldMethod(); // deprecated call
```

## How to fix

Switch to the recommended replacement method.

```php
<?php
$api = new Api();
$api->newMethod();
```
