===description===
Comma-separated `match(true)` arm conditions over scalar type-check
functions (`is_int($x), is_string($x)`) are OR semantics — the arm must
narrow $x to int|string, not collapse to just the last disjunct via
sequential (AND) narrowing.
===config===
suppress=UnusedVariable,MissingClosureReturnType
===file===
<?php
/**
 * @param int|string $x
 */
function bar($x): void {
    match (true) {
        is_int($x), is_string($x) => (function () use ($x) {
            /** @mir-check $x is int|string */
            $_ = 1;
        })(),
        default => null,
    };
}
===expect===
