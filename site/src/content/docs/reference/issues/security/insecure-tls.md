---
title: InsecureTls
code: MIR0841
description: "A curl or stream context option turns off checking the server's TLS certificate or host name."
sidebar:
  hidden: true
  order: 841
---

With certificate checks off, TLS still encrypts the connection, but to whoever answers: anyone on the network path can present their own certificate, read the traffic and change it. Reported for a literal `false` or `0` given to `CURLOPT_SSL_VERIFYPEER`, `CURLOPT_SSL_VERIFYHOST`, `CURLOPT_PROXY_SSL_VERIFYPEER` or `CURLOPT_PROXY_SSL_VERIFYHOST` through `curl_setopt()` or `curl_setopt_array()`, and to the `ssl` options `verify_peer` or `verify_peer_name` in `stream_context_create()`, `stream_context_set_option()` or `stream_context_set_options()`. A value held in a variable isn't checked, nor is a call inside a condition on a setting whose name mentions verify, SSL, TLS, certificates or insecure, which honours a caller's opt-out.

## Example

```php
<?php
$ch = curl_init('https://api.example.com/');
curl_setopt($ch, CURLOPT_SSL_VERIFYPEER, false); // InsecureTls
```

## How to fix

Leave verification on. For a private or self-signed certificate, trust it explicitly with `CURLOPT_CAINFO` or the `cafile` stream context option instead.
