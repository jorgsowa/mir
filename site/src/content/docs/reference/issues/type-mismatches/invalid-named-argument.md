---
title: InvalidNamedArgument
description: A named argument does not correspond to any parameter of the callable.
sidebar:
  order: 5
---

A named argument does not correspond to any parameter of the callable.

## Example

```php
<?php
function connect(string $host, int $port): void {}

connect(host: 'localhost', timeout: 30); // no parameter named 'timeout'
```

## How to fix

Use the correct parameter name or switch to a positional argument.
