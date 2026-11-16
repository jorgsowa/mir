---
title: TaintedNoSql
code: MIR0828
description: "User input reaches a MongoDB `$where` clause, or may be an array in a MongoDB filter."
sidebar:
  hidden: true
  order: 828
---

Reported for:

- a tainted value under a `'$where'` key of a query filter, also nested under `$or` or `$and` or in a filter built in a variable first, which MongoDB runs as JavaScript;
- a request value that may be an array in the filter passed to a `MongoDB\Collection` query or write method (`find()`, `findOne()`, `updateOne()`, `deleteMany()`, `countDocuments()` and so on) or to `new MongoDB\Driver\Query()`. PHP turns `?user[$ne]=x` into `['$ne' => 'x']`, so the filter `['user' => $_GET['user']]` becomes an operator query that matches every document. Taint on strings alone can't catch this; casting to `(string)` fixes it;
- `@taint-sink nosql` parameters.

## Example

```php
<?php
$users->findOne(['user' => $_POST['user'], 'pass' => $_POST['pass']]); // TaintedNoSql
```

## How to fix

Cast filter values to the expected scalar type, `(string) $_POST['user']`, and never build `$where` from input.
