---
title: WeakCryptography
code: MIR0816
description: "`openssl_encrypt()` or `openssl_decrypt()` is called with a broken cipher, a hardcoded key or IV, or an AEAD cipher without its tag."
sidebar:
  hidden: true
  order: 816
---

Reported for a literal cipher in ECB mode or from the DES, RC2, RC4 or Blowfish families, a key or IV written as a string literal, and `openssl_decrypt()` of a GCM, CCM, OCB or ChaCha20-Poly1305 cipher without the `tag` argument, which always fails. A value counts as written in the source when it is a literal, or a variable, global constant or class constant that holds one literal value at that point (`$mode = 0777; chmod($f, $mode)`, `self::CIPHER`).

## Example

```php
<?php
openssl_encrypt($data, 'aes-128-ecb', $key); // WeakCryptography
openssl_encrypt($data, 'aes-256-cbc', 'secret-key', 0, $iv); // WeakCryptography
```

## How to fix

Use an AEAD cipher such as `aes-256-gcm` with a key from configuration, a fresh `random_bytes()` IV for every message, and pass the tag to `openssl_decrypt()`. Consider the sodium extension.
