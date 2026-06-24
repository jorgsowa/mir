===description===
scalar (int|float|string|bool) is open — no false positive.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param scalar $x */
function test(mixed $x): void {
    if ($x === "foo") {}
    if ($x === 42) {}
}
===expect===
