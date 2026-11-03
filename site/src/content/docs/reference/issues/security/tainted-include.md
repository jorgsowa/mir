---
title: TaintedInclude
code: MIR0812
description: User-controlled input picks the file that `include` or `require` loads.
sidebar:
  hidden: true
  order: 812
---

User-controlled input reaches the path of `include`, `include_once`, `require` or `require_once`, letting an attacker run any local (or remote) PHP file.

## Example

```php
<?php
include $_GET['page'] . '.php'; // TaintedInclude
```

## How to fix

Map the input to a fixed list of files instead of building the path from it.
