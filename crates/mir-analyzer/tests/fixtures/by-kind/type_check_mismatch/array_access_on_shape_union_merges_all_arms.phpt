===description===
FN: indexing a union of shapes by a literal key returned only the FIRST
matching union member's type for that key, discarding the other arms —
`array{a: int}|array{a: string}` accessed via `$x['a']` gave `int` only.
===config===
suppress=UnusedVariable
===file===
<?php
/** @param array{a: int}|array{a: string} $x */
function f(array $x): void {
    $v = $x['a'];
    /** @mir-check $v is int|string */
    $_ = $v;
}
===expect===
