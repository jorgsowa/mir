---
title: DuplicateArrayKey
code: MIR0303
description: An array literal contains duplicate keys, with later values overwriting earlier ones.
sidebar:
  hidden: true
  order: 4
---

An array literal contains duplicate keys. In PHP, when duplicate keys exist, later values overwrite earlier ones, which may indicate a logic error.

## Example

```php
<?php
$config = [
    'host' => 'localhost',
    'port' => 3306,
    'host' => '127.0.0.1', // Duplicate key - overwrites 'localhost'
];
```

## How to fix

Remove duplicate keys or use different keys:

```php
<?php
$config = [
    'host' => '127.0.0.1',
    'port' => 3306,
];
```
