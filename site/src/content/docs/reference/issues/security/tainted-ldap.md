---
title: TaintedLdap
code: MIR0808
description: User-controlled input reaches an LDAP search filter or base DN.
sidebar:
  hidden: true
  order: 808
---

User-controlled input reaches the base DN or filter of `ldap_search()`, `ldap_list()` or `ldap_read()`, risking LDAP injection.

## Example

```php
<?php
ldap_search($ldap, 'dc=example,dc=com', '(uid=' . $_GET['user'] . ')'); // TaintedLdap
```

## How to fix

Escape the value with `ldap_escape()` using the `LDAP_ESCAPE_FILTER` or `LDAP_ESCAPE_DN` flag.
