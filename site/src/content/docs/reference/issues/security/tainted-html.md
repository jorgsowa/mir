---
title: TaintedHtml
description: User-controlled input reaches an HTML output sink without sanitization.
sidebar:
  order: 1
---

User-controlled input reaches an HTML output sink without sanitization, creating a cross-site scripting (XSS) risk.

## Example

```php
<?php
echo $_GET['name']; // raw user input written to HTML
```

## How to fix

Escape output with `htmlspecialchars()` before writing to HTML.

```php
<?php
echo htmlspecialchars($_GET['name'], ENT_QUOTES, 'UTF-8');
```
