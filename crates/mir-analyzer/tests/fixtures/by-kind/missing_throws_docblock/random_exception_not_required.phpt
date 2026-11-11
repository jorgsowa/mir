===description===
`random_int()` and `random_bytes()` throw `Random\RandomException` only when the system has no randomness source, so callers need no `@throws` for it. Other checked exceptions are still reported.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

function roll(): int {
    return random_int(1, 6);
}

function token(): string {
    return bin2hex(random_bytes(16));
}

/** @throws \Exception */
function mayFail(): void {
    throw new \Exception('failed');
}

function caller(): void {
    mayFail();
//  ^^^^^^^^^ MissingThrowsDocblock: Exception Exception is thrown but not declared in @throws
}
