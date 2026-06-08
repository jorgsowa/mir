---
title: WrongCaseClass
code: MIR1011
description: Class, interface, or enum name casing does not match its declaration.
sidebar:
  hidden: true
  order: 1011
---

A class, interface, or enum is referenced with casing that differs from its declaration. PHP
class name resolution is case-insensitive at runtime, but consistent casing is expected by
autoloaders and static analysis tools.

## Example

```php
<?php
class HttpClient {}

$client = new httpclient(); // wrong casing: should be HttpClient
```

## How to fix

Update the reference to use the exact casing from the class/interface/enum declaration.
