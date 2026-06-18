---
title: TaintedLlmPrompt
code: MIR0804
description: "Tainted input reaches a `@taint-sink llm_prompt` parameter without sanitization."
sidebar:
  hidden: true
  order: 804
---

Tainted input reaches a `@taint-sink llm_prompt` parameter without sanitization.

## Example

```php
<?php
$prompt = $_GET['q'];
$llm->complete($prompt); // TaintedLlmPrompt
```

## How to fix

Validate or sanitize untrusted input before building an LLM prompt.
