---
title: InsecureFilePermissions
code: MIR0836
description: "`chmod()` or `umask()` with a literal value leaves files writable by every user."
sidebar:
  hidden: true
  order: 836
---

A world-writable file lets any local user, or any other site on a shared host, change it: a cache or config file becomes a way to run code (CWE-732). Reported for `chmod()` with a literal mode that includes the others-write bit (`0777`, `0666`, `0757`) and `umask()` with a literal mask that doesn't remove it (`umask(0)`). `mkdir()` isn't checked, because the umask applies to its mode and `0777` is its default.

## Example

```php
<?php
chmod($cacheFile, 0777); // InsecureFilePermissions
```

## How to fix

Grant write access to the owner only, or to the group if the web server needs it: `chmod($cacheFile, 0644)`.
