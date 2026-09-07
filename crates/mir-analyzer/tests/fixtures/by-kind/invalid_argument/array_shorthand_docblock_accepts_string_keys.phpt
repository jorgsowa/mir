===description===
FP-J8: `T[]` docblock shorthand parsed as `array<int, T>` instead of
`array<array-key, T>` (Psalm/PHPStan's own documented meaning) — any
string-keyed array (the common PSR-3 `$context` idiom) into a
`mixed[]`/`string[]`-docblocked param falsely flagged.
===config===
suppress=UnusedParam
===file===
<?php

/** @param string[] $context */
function log(string $message, array $context = []): void {}

function withStringKeys(): void {
    log('oops', ['requestId' => '42', 'userId' => '7']);
}
===expect===
