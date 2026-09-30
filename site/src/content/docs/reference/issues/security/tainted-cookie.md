---
title: TaintedCookie
code: MIR0806
description: User-controlled input reaches setcookie() without validation.
sidebar:
  hidden: true
  order: 806
---

User-controlled input reaches `setcookie()` or `setrawcookie()`, risking cookie injection or session fixation.

## Example

```php
<?php
setcookie('session', $_GET['sid']); // TaintedCookie
```

## How to fix

Generate cookie values server-side, or validate them against a strict format first.
