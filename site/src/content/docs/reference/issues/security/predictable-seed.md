---
title: PredictableSeed
code: MIR0839
description: "`mt_srand()` or `srand()` gets a literal or clock-based seed."
sidebar:
  hidden: true
  order: 839
---

Opt-in: enable it with an `<issueHandlers>` entry such as `<PredictableSeed errorLevel="warning"/>`. A literal seed (`mt_srand(1234)`) or one built from the clock or process ID (`srand(time())`, `mt_srand((int) microtime(true) * 1000)`) lets anyone who knows or guesses it compute every later `mt_rand()`, `rand()`, `shuffle()` and `array_rand()` result. Seeding is often deliberate, for reproducible tests or simulations, which is why this is opt-in; `InsecureRandomness` already reports the generator's values reaching a token or session ID.

## Example

```php
<?php
mt_srand(time()); // PredictableSeed
```

## How to fix

Don't seed: PHP seeds the generator randomly. For secrets use `random_bytes()` or `random_int()`.
