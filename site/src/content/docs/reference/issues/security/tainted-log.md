---
title: TaintedLog
code: MIR0830
description: "User input reaches a log message, where a line break forges log entries."
sidebar:
  hidden: true
  order: 830
---

Opt-in: enable it with an `<issueHandlers>` entry such as `<TaintedLog errorLevel="warning"/>`. A line break in the message forges log entries, which can hide an attack or mislead whoever reads the log (CWE-117). Reported for the message of `error_log()` and of the PSR-3 `Psr\Log\LoggerInterface` methods (`error()`, `info()`, `log()` and so on), and for `@taint-sink log` parameters. Values passed through the context array and referenced by `{placeholder}` aren't reported, so using PSR-3 as intended keeps the check quiet. It is low severity and common in application code, which is why it is opt-in.

## Example

```php
<?php
$logger->warning('Failed login for ' . $_POST['user']); // TaintedLog
```

## How to fix

Pass the value in the context array, `$logger->warning('Failed login for {user}', ['user' => $_POST['user']])`, or strip line breaks.
