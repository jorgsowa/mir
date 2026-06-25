===description===
Passing a plain `int` (not a specific literal) to an int-mask parameter is
not flagged — the widening heuristic treats an unverified int as potentially
valid. Only known-wrong literals are rejected.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
/**
 * @param int-mask<1, 2, 4> $flags
 */
function set_flags(int $flags): void {}

function caller(int $x): void {
    set_flags($x);  // int is not statically verified but not flagged
}
===expect===
