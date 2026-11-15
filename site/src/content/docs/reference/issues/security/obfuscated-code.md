---
title: ObfuscatedCode
code: MIR0840
description: "Code passed to `eval()` or a function called by name comes out of a decoder such as `base64_decode()`."
sidebar:
  hidden: true
  order: 840
---

Legitimate code almost never decodes the code it runs. `eval(gzinflate(base64_decode('...')))` or `$f = base64_decode('c3lzdGVt'); $f($_GET['c']);` hides what runs from reviewers and scanners, and is the signature of web shells and backdoors in compromised plugins and vendor packages. Reported for `eval()`, `assert()`, `create_function()`, `call_user_func()` and `call_user_func_array()`, and calls through a variable or expression, when the code or function name comes from `base64_decode()`, `gzinflate()`, `gzuncompress()`, `gzdecode()`, `str_rot13()`, `strrev()`, `hex2bin()` or `convert_uudecode()`, directly, concatenated or through a variable, or is spelled with three or more `chr()` calls.

## Example

```php
<?php
eval(gzinflate(base64_decode('S03OyFfIzy9KUQQA'))); // ObfuscatedCode
```

## How to fix

Remove the code and audit how it got there; if it is your own, write the code out in plain PHP.
