---
title: TaintedInput
code: MIR0800
description: Tainted user input flows to a sensitive sink (generic taint sink).
sidebar:
  hidden: true
  order: 800
---

Tainted user input flows to a sensitive sink. This is the generic taint kind
used by sinks that do not have a dedicated issue (XSS uses
[`TaintedHtml`](/reference/issues/security/tainted-html/), SQL injection uses
[`TaintedSql`](/reference/issues/security/tainted-sql/), command injection uses
[`TaintedShell`](/reference/issues/security/tainted-shell/)). The `sink` field
names the category that was reached, e.g. `file` or `unserialize`.

## Sinks

| `sink`        | Functions                                                                                  | Risk                                          |
| ------------- | ------------------------------------------------------------------------------------------ | --------------------------------------------- |
| `file`        | `fopen`, `file_get_contents`, `file_put_contents`, `readfile`, `file`, `unlink` (path arg) | Path traversal / local file inclusion / SSRF  |
| `unserialize` | `unserialize` (payload arg)                                                                 | PHP object injection                          |

Only the path/payload argument is considered tainted-sensitive — writing
tainted *data* to a constant path is not reported, only a tainted *path* is.

## Example

```php
<?php
function download(): string {
    $path = $_GET['path'];
    // TaintedInput: tainted path reaches a filesystem sink
    return file_get_contents($path);
}
```

## How to fix

Validate or canonicalize the value before it reaches the sink — for `file`
sinks, resolve against an allow-list of directories with `realpath()` and reject
paths that escape the base directory; for `unserialize`, prefer a safe format
(`json_decode`) or pass `['allowed_classes' => [...]]`. If the value is already
trusted, suppress with `@mir-suppress TaintedInput`.
