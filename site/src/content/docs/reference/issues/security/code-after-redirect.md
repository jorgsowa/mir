---
title: CodeAfterRedirect
code: MIR0833
description: "A `header('Location: ...')` redirect in one branch is followed by code that runs after the branch, with no `exit`."
sidebar:
  hidden: true
  order: 833
---

`header('Location: ...')` only adds a header: the script keeps running. When the redirect guards a protected page (`if (!$loggedIn) { header('Location: /login'); }`), the code after the guard still runs for the rejected request: a client that ignores the redirect reads the page, and any action the page performs still happens. Reported at the `header()` call when a branch sends the redirect, the other branch doesn't, and code after the condition runs with no `exit`, `die`, `return`, `throw` or call to a function that never returns in between. A redirect outside any condition followed by its own page body isn't reported: every request gets that page.

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
