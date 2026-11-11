---
title: InvalidGeneratorIteration
code: MIR0237
description: "`foreach` iterates a generator whose send type does not accept `null`."
sidebar:
  hidden: true
  order: 237
---

`foreach` resumes a generator without sending a value, so every `yield` expression inside it
evaluates to `null`. A generator declared as `Generator<TKey, TValue, TSend, TReturn>` with a
`TSend` that does not accept `null` expects to be driven with `send()` instead.

## Example

```php
<?php
/** @return Generator<int, string, int, void> */
function prompts(): Generator {
    $answer = yield 'How many?';
    echo $answer + 1;
}

foreach (prompts() as $prompt) {} // InvalidGeneratorIteration
```

## How to fix

Drive the generator with `current()` and `send()`, or declare the send type as nullable (or
`void`/`mixed`) when the generator does not rely on a sent value.
