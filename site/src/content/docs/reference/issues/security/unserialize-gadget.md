---
title: UnserializeGadget
code: MIR0821
description: "A magic method that runs on unserialized objects passes one of its properties to a dangerous call."
sidebar:
  hidden: true
  order: 821
---

Opt-in: enable it with an `<issueHandlers>` entry such as `<UnserializeGadget errorLevel="warning"/>`. `__destruct()`, `__wakeup()`, `__unserialize()` and `__toString()` run on objects `unserialize()` creates, with properties the payload chooses. A property reaching a file, shell, callback, `unserialize()`, network or SQL call there is a gadget that turns object injection into file deletion or code execution.

## Example

```php
<?php
final class TempFile {
    private string $path;
    public function __destruct() {
        unlink($this->path); // UnserializeGadget
    }
}
```

## How to fix

Keep side effects out of these magic methods, validate the property first, or block unserialization with `__unserialize()` throwing or `allowed_classes` at every `unserialize()` call.
