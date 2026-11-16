---
title: MissingSensitiveParameter
code: MIR0822
description: "A parameter named like a password, secret, token or API key lacks `#[\SensitiveParameter]`."
sidebar:
  hidden: true
  order: 822
---

Opt-in: enable it with an `<issueHandlers>` entry such as `<MissingSensitiveParameter errorLevel="warning"/>`. Since PHP 8.2, `#[\SensitiveParameter]` replaces an argument's value in stack traces, and error messages that print arguments otherwise put the value into logs. Reported for an untyped, `string` or `mixed` parameter of a function or method with a body whose name is `password`, `passwd`, `passphrase`, `secret`, `token`, `otp` or `apiKey`, or ends in one of them (`$newPassword`, `$accessToken`, `$userpassword`), except pagination and cursor tokens such as `$nextPageToken`, when the target PHP version is 8.2 or later. It relies on names, which is why it is opt-in.

## Example

```php
<?php
function login(string $user, string $password): bool { /* ... */ } // MissingSensitiveParameter
```

## How to fix

Add `#[\SensitiveParameter]` before the parameter.
