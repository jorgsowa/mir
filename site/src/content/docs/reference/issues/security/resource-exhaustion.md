---
title: ResourceExhaustion
code: MIR0831
description: "A size, count or duration from the request reaches a builtin whose cost grows with it."
sidebar:
  hidden: true
  order: 831
---

A client that picks how much work a request does can tie up memory or workers with one request (CWE-400). Reported when a request value, directly or through a variable, reaches the count of `str_repeat()`, `str_pad()`, `range()` (its end), `array_fill()` or `array_pad()`, or the duration of `sleep()`, `usleep()`, `time_nanosleep()` or `set_time_limit()`. An `(int)` cast or `intval()` doesn't help, since the number itself is the problem. A value compared with a bound first, or clamped with `min()`, isn't reported.

## Example

```php
<?php
$padding = str_repeat(' ', (int) $_GET['width']); // ResourceExhaustion
```

## How to fix

Clamp the value to a sensible maximum: `min((int) $_GET['width'], 200)`.
