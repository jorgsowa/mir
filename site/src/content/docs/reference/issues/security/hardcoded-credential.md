---
title: HardcodedCredential
code: MIR0837
description: "A password, key or token is written in the source code."
sidebar:
  hidden: true
  order: 837
---

A credential in the source reaches everyone who can read the repository, its history, backups and deployed copies, and can't be rotated without a release (CWE-798). Reported for:

- a non-empty string literal as the password of `new PDO()`, `new mysqli()`, `mysqli_connect()`, `mysqli_real_connect()`, `ldap_bind()` and `oci_connect()`, or as the `password=` of a literal `pg_connect()` connection string;
- string literals shaped like a live secret: an AWS access key ID (`AKIA…`, `ASIA…`), a PEM private key with its key material, or a Stripe live secret key (`sk_live_…`, `rk_live_…`).

Matching known parameters and exact secret formats keeps this precise; it doesn't guess from entropy. AWS's documented example keys (containing `EXAMPLE`) aren't reported. Literal `openssl_encrypt()` keys are reported as `WeakCryptography`.

## Example

```php
<?php
$db = new PDO('mysql:host=db;dbname=app', 'app', 's3cr3t-pa55'); // HardcodedCredential
```

## How to fix

Read credentials from the environment or a secrets store at runtime: `getenv('DB_PASSWORD')`.
