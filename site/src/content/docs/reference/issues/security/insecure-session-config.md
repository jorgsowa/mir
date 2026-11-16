---
title: InsecureSessionConfig
code: MIR0817
description: "Code turns off a session protection that PHP's secure defaults rely on, or takes the session ID from the request."
sidebar:
  hidden: true
  order: 817
---

Reported when `ini_set()` or `session_start()` options turn off `session.use_strict_mode`, `session.use_only_cookies` or `session.cookie_httponly`, or turn on `session.use_trans_sid`. Also reported for `session_id()` of a `$_GET`, `$_POST` or `$_REQUEST` value, also through a variable, `trim()` or `filter_input()`,, which lets an attacker fix the victim's session. These open the door to session fixation, session IDs in URLs, and session cookies readable from JavaScript.

## Example

```php
<?php
ini_set('session.use_strict_mode', '0'); // InsecureSessionConfig
session_start(['use_trans_sid' => 1]); // InsecureSessionConfig
session_id($_GET['sid']); // InsecureSessionConfig
```

## How to fix

Keep strict mode, cookie-only sessions and HttpOnly session cookies on, leave `use_trans_sid` off, and let PHP read the session ID from the cookie.
