===description===
in_array($needle, [...], true) — with the strict 3rd argument — may safely
narrow a needle whose current type spans both strings and ints down to the
haystack's literal-int union, since strict (===) comparison rules out the
cross-type loose-equality matches (e.g. string "1" vs int 1) that make this
unsafe without the strict flag.
===config===
suppress=UnusedVariable
===file===
<?php
function test(int|string $x): void {
    if (in_array($x, [1, 2], true)) {
        /** @mir-check $x is 1|2 */
        $_ = $x;
    }
}
===expect===
