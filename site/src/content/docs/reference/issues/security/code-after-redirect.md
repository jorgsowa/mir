---
title: CodeAfterRedirect
code: MIR0833
description: "A `header('Location: ...')` redirect is followed by output on a path with no `exit`."
sidebar:
  hidden: true
  order: 833
---

`header('Location: ...')` only adds a header: the script keeps running and sends the rest of the page in the response body. When the redirect guards a protected page (`if (!$loggedIn) { header('Location: /login'); }`), a client that ignores the redirect still reads the page. Reported at the `header()` call when `echo`, `print` or inline HTML runs after it on some path with no `exit`, `die`, `return`, `throw` or call to a function that never returns in between.

## Example

```php
<?php
if (!isset($_SESSION['user'])) {
    header('Location: /login.php'); // CodeAfterRedirect
}
echo 'Secret admin page';
```

## How to fix

Add `exit;` right after the redirect.
