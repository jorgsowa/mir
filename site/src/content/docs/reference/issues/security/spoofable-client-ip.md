---
title: SpoofableClientIp
code: MIR0832
description: "A condition decides on a client IP taken from `X-Forwarded-For`, `Client-IP` or `X-Real-IP`."
sidebar:
  hidden: true
  order: 832
---

`$_SERVER['HTTP_X_FORWARDED_FOR']`, `HTTP_CLIENT_IP` and `HTTP_X_REAL_IP` come from request headers, so any client can send `X-Forwarded-For: 127.0.0.1` and pass an IP-based admin gate or rate limit. Reported when an `if`, `elseif` or ternary condition compares such a value, directly or through a variable, with something other than an empty value, or looks it up with `in_array()`, `array_key_exists()`, `array_search()`, `str_starts_with()`, `strpos()`, `strncmp()` or `preg_match()`. A check of whether the header is present (`isset()`, `!== ''`) isn't reported.

## Example

```php
<?php
$ip = $_SERVER['HTTP_X_FORWARDED_FOR'] ?? $_SERVER['REMOTE_ADDR'];
if (in_array($ip, ['10.0.0.1', '10.0.0.2'], true)) { // SpoofableClientIp
    grant_admin();
}
```

## How to fix

Use `$_SERVER['REMOTE_ADDR']`, or read forwarded headers only when `REMOTE_ADDR` is a proxy you trust.
