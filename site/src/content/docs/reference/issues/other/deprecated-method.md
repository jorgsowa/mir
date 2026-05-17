---
title: DeprecatedMethod
code: MIR1002
description: A method declaration is itself marked @deprecated (reported on the method definition, not its call sites).
sidebar:
  hidden: true
  order: 1
---

A method declaration is marked `@deprecated`. This is reported on the method itself to signal that it is a deprecated API entry point. Call-site violations are reported as [`DeprecatedMethodCall`](../deprecated-method-call/).

:::note
This issue kind is defined but not yet emitted by the analyzer. Call sites of deprecated methods currently produce `DeprecatedMethodCall` (MIR1001).
:::

## Example

```php
<?php
class Api {
    /** @deprecated Use newMethod() instead */
    public function oldMethod(): void {}

    public function newMethod(): void {}
}
```

## How to fix

Remove the `@deprecated` annotation once all callers have migrated, or keep it to signal to users that the method should not be used.
