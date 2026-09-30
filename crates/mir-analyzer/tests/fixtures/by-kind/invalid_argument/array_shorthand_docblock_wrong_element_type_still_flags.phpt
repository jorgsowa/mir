===description===
Negative control for the J8 array-shorthand-key fix: `T[]` widens the KEY
type to array-key, but the element (value) type is still enforced — a
wrong element type must still flag.
===config===
suppress=UnusedParam
===file===
<?php

/** @param string[] $context */
function log(string $message, array $context = []): void {}

function withWrongElementType(): void {
    log('oops', ['requestId' => 42]);
//              ^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $context of log() expects 'array<int|string, string>', got 'array{'requestId': 42}'
}
===expect===
