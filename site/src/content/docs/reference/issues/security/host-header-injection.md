---
title: HostHeaderInjection
code: MIR0827
description: "An email body contains a value built from the `Host` header the client sent."
sidebar:
  hidden: true
  order: 827
---

The client chooses the `Host` header. A password-reset or confirmation link built from it, `"https://{$_SERVER['HTTP_HOST']}/reset?token=$token"`, points the victim's email at the attacker's server, and the token arrives there when the victim clicks it: account takeover. Reported for a `mail()` or `mb_send_mail()` message containing `$_SERVER['HTTP_HOST']`, `$_SERVER['HTTP_X_FORWARDED_HOST']`, `getHttpHost()` or `getSchemeAndHttpHost()`, directly or through variables and string building. A host compared with an expected value, checked against a list or with a `.domain` suffix first isn't reported; a check that it's set or short enough doesn't count.

## Example

```php
<?php
$link = 'https://' . $_SERVER['HTTP_HOST'] . '/reset?token=' . $token;
mail($email, 'Password reset', "Reset your password: $link"); // HostHeaderInjection
```

## How to fix

Build links from a configured base URL, or check the host against an allowlist.
