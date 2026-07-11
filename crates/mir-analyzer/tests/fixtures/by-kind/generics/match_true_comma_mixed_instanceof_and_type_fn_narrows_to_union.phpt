===description===
Comma-separated `match(true)` arm conditions mixing an `instanceof` check
with a scalar type-check function (`$x instanceof A, is_string($x)`) are OR
semantics — the arm must narrow $x to A|string, the same as an all-instanceof
or all-type-fn comma list already does.
===config===
suppress=UnusedVariable,MissingClosureReturnType
===file===
<?php
class A {}
/**
 * @param A|string|int $x
 */
function bar($x): void {
    match (true) {
        $x instanceof A, is_string($x) => (function () use ($x) {
            /** @mir-check $x is A|string */
            $_ = 1;
        })(),
        default => null,
    };
}
===expect===
