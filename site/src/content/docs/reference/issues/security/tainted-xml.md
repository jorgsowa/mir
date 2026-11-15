---
title: TaintedXml
code: MIR0819
description: "User-controlled XML is parsed with flags that load external entities or lift libxml's size limits."
sidebar:
  hidden: true
  order: 819
---

User-controlled input reaches `simplexml_load_string()`, `new SimpleXMLElement()`, `DOMDocument::loadXML()` or `XMLReader::XML()` with `LIBXML_NOENT`, `LIBXML_DTDLOAD` or `LIBXML_PARSEHUGE`. libxml 2.9+ is safe by default; these flags turn on external entity loading (XXE: local file reads, SSRF) or remove the limits that stop billion-laughs denial of service.

## Example

```php
<?php
simplexml_load_string($_POST['xml'], 'SimpleXMLElement', LIBXML_NOENT); // TaintedXml
```

## How to fix

Drop `LIBXML_NOENT`, `LIBXML_DTDLOAD` and `LIBXML_PARSEHUGE` when parsing untrusted XML.
