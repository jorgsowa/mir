---
title: InsecureCors
code: MIR0838
description: "`Access-Control-Allow-Origin` echoes the request's `Origin` header unchecked while credentials are allowed."
sidebar:
  hidden: true
  order: 838
---

`header('Access-Control-Allow-Origin: ' . $_SERVER['HTTP_ORIGIN'])` together with `Access-Control-Allow-Credentials: true` tells the browser that every site may read responses made with the user's cookies, so any page the user visits can read their data. Reported for such a header when the same function also sends `Access-Control-Allow-Credentials: true`, before or after it, and no condition before it pinned that origin: a comparison with a value, an allowlist lookup, an anchored `preg_match()`, a `.domain` suffix check or a helper of the program's own. A presence check (`isset()`, `!== ''`) or a suffix without the leading dot (`str_ends_with($origin, 'trusted.com')` accepts `eviltrusted.com`) doesn't count. `Access-Control-Allow-Origin: *` with credentials isn't reported, since browsers refuse that combination.

## Example

```php
<?php
header('Access-Control-Allow-Origin: ' . $_SERVER['HTTP_ORIGIN']); // InsecureCors
header('Access-Control-Allow-Credentials: true');
```

## How to fix

Reflect the origin only after checking it against an allowlist: `if (in_array($origin, $allowedOrigins, true)) { ... }`.
