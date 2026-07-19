===description===
A 3-way-or-more `||` chain mixing instanceof and is_TYPE() disjuncts on the
same variable (e.g. `$x instanceof A || is_string($x) || $x instanceof B`)
narrows to the full union — the 2-way case already worked, but a nested
`||` on the left-associative parse tree wasn't recursed into. A chain that
mixes in a different variable still bails out (no narrowing, no crash).
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class A {}
class B {}

/**
 * @param A|B|string|int $x
 */
function threeWay($x): void {
    if ($x instanceof A || is_string($x) || $x instanceof B) {
        /** @mir-check $x is A|string|B */
        $_ = 1;
    }
}

/**
 * @param A|B|string|int $x
 */
function threeWayReordered($x): void {
    if (is_string($x) || $x instanceof A || $x instanceof B) {
        /** @mir-check $x is string|A|B */
        $_ = 1;
    }
}

/**
 * @param A|string|int $x
 * @param B|string|int $y
 */
function differentVariablesStillBail($x, $y): void {
    if ($x instanceof A || is_string($y) || $x instanceof B) {
        /** @mir-check $x is A|string|int */
        $_ = 1;
    }
}
===expect===
