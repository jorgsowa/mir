---
title: TaintedShell
description: User-controlled input reaches a shell execution sink without escaping.
sidebar:
  order: 3
---

User-controlled input reaches a shell execution sink without escaping, creating a command injection risk.

## Example

```php
<?php
system('convert ' . $_POST['file']); // injection risk
```

## How to fix

Use `escapeshellarg()` on any user-supplied values before passing them to shell functions.

```php
<?php
system('convert ' . escapeshellarg($_POST['file']));
```
