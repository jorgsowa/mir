---
title: InsecureCheckResult
code: MIR0834
description: "A security check treats a function's error result as success."
sidebar:
  hidden: true
  order: 834
---

Reported for a condition that lets a function's error value through. `openssl_verify()`, `openssl_x509_verify()`, `openssl_pkcs7_verify()` and `ldap_compare()` return `-1` on error, and `-1` is truthy, so `if (openssl_verify($data, $sig, $key))`, `== true`, `!= 0` or `!== false` accepts a signature that failed to verify. This has caused real signature-bypass vulnerabilities.

## Example

```php
<?php
if (openssl_verify($data, $signature, $publicKey)) { // InsecureCheckResult
    accept($data);
}
```

## How to fix

Compare with the success value: `openssl_verify($data, $signature, $publicKey) === 1`.
