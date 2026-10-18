---
title: TaintedSsrf
code: MIR0807
description: User-controlled input picks the URL or host the server connects to.
sidebar:
  hidden: true
  order: 807
---

User-controlled input reaches a URL or host the server connects to (`curl_init()`, `CURLOPT_URL`, `fsockopen()`, `get_headers()`, `new SoapClient()`), letting an attacker make the server send requests to internal services.

## Example

```php
<?php
$ch = curl_init();
curl_setopt($ch, CURLOPT_URL, $_GET['url']); // TaintedSsrf
```

## How to fix

Build the URL from a fixed base, or check the host against an allowlist before connecting.
