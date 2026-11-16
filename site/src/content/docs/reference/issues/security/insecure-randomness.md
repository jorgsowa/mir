---
title: InsecureRandomness
code: MIR0818
description: "A session ID or a security token comes from a guessable random generator."
sidebar:
  hidden: true
  order: 818
---

Reported when a value from `rand()`, `mt_rand()`, `uniqid()`, `lcg_value()`, `str_shuffle()` or `array_rand()`, or from the clock (`time()`, `microtime()`, `hrtime()`), even hashed, encoded or concatenated, is passed to `session_id()` or returned from a function or method whose name ends in `token`, `secret`, `password`, `nonce`, `salt`, `otp`, `apiKey` or a reset, verification or confirmation `code`; an acronym counts as one word (`getOTP`). Pagination and cursor tokens such as `nextPageToken()` aren't secrets. These generators can be predicted from a few outputs (CWE-338). A value that also mixes in `random_bytes()` or `random_int()` isn't reported.

## Example

```php
<?php
function generateResetToken(): string {
    return md5(uniqid()); // InsecureRandomness
}
```

## How to fix

Use `bin2hex(random_bytes(32))`, `random_int()`, or `Random\Randomizer` with its default secure engine.
