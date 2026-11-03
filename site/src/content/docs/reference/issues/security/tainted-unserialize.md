---
title: TaintedUnserialize
code: MIR0814
description: User-controlled input reaches `unserialize()`.
sidebar:
  hidden: true
  order: 814
---

User-controlled input reaches the payload of `unserialize()`, letting an attacker instantiate classes and trigger their magic methods (PHP object injection).

## Example

```php
<?php
$prefs = unserialize($_COOKIE['prefs']); // TaintedUnserialize
```

## How to fix

Use `json_decode()` for untrusted data, or pass `['allowed_classes' => false]`.
