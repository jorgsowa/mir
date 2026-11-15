---
title: TaintedCsv
code: MIR0829
description: "User input is written as a CSV field that a spreadsheet may run as a formula."
sidebar:
  hidden: true
  order: 829
---

Spreadsheet programs run a CSV field that starts with `=`, `+`, `-` or `@` as a formula, which can leak data or run commands on the machine of whoever opens an export (CWE-1236). Reported for tainted elements of the row `fputcsv()` or `SplFileObject::fputcsv()` writes, unless the element starts with literal text that isn't a formula character, and for a request array written as a whole row.

## Example

```php
<?php
fputcsv($out, [$_POST['name'], $_POST['email']]); // TaintedCsv
```

## How to fix

Prefix fields that start with `=`, `+`, `-`, `@`, tab or carriage return with a single quote, or cast numeric fields.
