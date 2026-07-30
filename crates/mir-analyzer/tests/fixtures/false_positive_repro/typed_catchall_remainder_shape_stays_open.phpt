===description===
`array{key?: T, ...array<K, V>}`'s typed catch-all remainder wasn't recognized as an
open-shape marker at all — only a bare `...` item set `is_open`, so this shape parsed as
sealed to just its declared keys and rejected any extra key.
===config===
suppress=UnusedParam
===file===
<?php
/** @param array{exception?: Throwable, ...array<string, mixed>} $context */
function logError(string $msg, array $context = []): void {}

logError('oops', ['requestId' => 42]);

===expect===
