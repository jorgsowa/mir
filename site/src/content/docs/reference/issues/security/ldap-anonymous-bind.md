---
title: LdapAnonymousBind
code: MIR0835
description: "`ldap_bind()` takes a password from the request that isn't checked to be non-empty."
sidebar:
  hidden: true
  order: 835
---

LDAP servers treat a simple bind with a DN and an empty password as an unauthenticated (anonymous) bind, and report it as a success. A login that checks only whether `ldap_bind($conn, $dn, $_POST['password'])` returned true lets anyone in with an empty password. Reported when the password argument is tainted and its type still allows an empty or falsy value: no `=== ''`, `empty()` or `strlen()` check narrowed it first.

## Example

```php
<?php
if (ldap_bind($conn, $dn, $_POST['password'])) { // LdapAnonymousBind
    log_in($user);
}
```

## How to fix

Reject an empty password before binding: `if ($password === '') { return false; }`.
