---
title: TimingUnsafeComparison
code: MIR0842
description: "A `hash_hmac()` digest or a stored secret is compared with `==` or `===` instead of `hash_equals()`."
sidebar:
  hidden: true
  order: 842
---

`==` and `===` stop at the first character that differs, so a comparison takes slightly longer the more leading characters of a guess are right. An attacker who can time many requests can recover a valid signature or token one character at a time. `hash_equals()` takes the same time whatever matches. Reported for `==`, `===`, `!=` and `!==` between a `hash_hmac()` digest (also hex- or base64-encoded, or stored in a variable first) and any value that isn't a literal, and between request data and a variable (other than a parameter, which usually holds the submitted value), property or array key whose name ends in `token`, `secret`, `password`, `nonce`, `salt`, `otp` or `apiKey`. Request data includes `$_GET`, `$_POST`, `$_REQUEST` and `$_COOKIE`, and values read through Symfony's `Request` and its parameter bags (`$request->query->getInt('n')`), a Laravel request or `Request` facade (`$request->input('n')`), or a PSR-7 server request (`$request->getQueryParams()['n']`).

## Example

```php
<?php
$expected = hash_hmac('sha256', $payload, $key);
if ($expected !== $_SERVER['HTTP_X_SIGNATURE']) { // TimingUnsafeComparison
    exit;
}
```

## How to fix

Compare with `hash_equals($expected, $given)`, passing the known value first. Check passwords with `password_verify()` instead.
