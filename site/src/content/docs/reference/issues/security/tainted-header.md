---
title: TaintedHeader
code: MIR0805
description: User-controlled input reaches header() without validation.
sidebar:
  hidden: true
  order: 805
---

User-controlled input reaches `header()`, risking response splitting or an open redirect.

## Example

```php
<?php
header('Location: ' . $_GET['next']); // TaintedHeader
```

## How to fix

Validate the value against an allowlist, or redirect only to known paths.

```php
<?php
$allowed = ['/home', '/account'];
header('Location: ' . (in_array($_GET['next'], $allowed, true) ? $_GET['next'] : '/home'));
```
