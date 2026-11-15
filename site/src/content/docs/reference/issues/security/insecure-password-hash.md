---
title: InsecurePasswordHash
code: MIR0815
description: "`password_hash()` is called with options or input that weaken it."
sidebar:
  hidden: true
  order: 815
---

Reported for a `salt` option (ignored since PHP 8.0), for a bcrypt `cost` below 10, and for `password_hash()` of a raw binary digest, which bcrypt truncates at the first NUL byte.

## Example

```php
<?php
password_hash(hash('sha256', $password, true), PASSWORD_BCRYPT); // InsecurePasswordHash
password_hash($password, PASSWORD_BCRYPT, ['cost' => 4]); // InsecurePasswordHash
```

## How to fix

Use `password_hash()` with its default options, and check passwords with `password_verify()`. To pre-hash long passwords, encode the digest (`base64_encode(hash('sha256', $p, true))`).
