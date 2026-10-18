---
title: TaintedXpath
code: MIR0809
description: User-controlled input reaches an XPath expression.
sidebar:
  hidden: true
  order: 809
---

User-controlled input reaches `DOMXPath::query()`, `DOMXPath::evaluate()` or `SimpleXMLElement::xpath()`, risking XPath injection.

## Example

```php
<?php
$xpath->query("//user[name='" . $_GET['name'] . "']"); // TaintedXpath
```

## How to fix

Validate the value against a strict format, or compare it in PHP after selecting candidate nodes with a fixed expression.
