===description===
`$x instanceof A || is_string($x)` in an if-condition's true branch must
narrow $x to A|string — a MIX of instanceof and is_TYPE() disjuncts that
neither the pure-instanceof-OR nor the pure-type-fn-OR narrowing handles
alone, since each requires every disjunct to be its own single kind.
===config===
suppress=UnusedVariable
===file===
<?php
class A {}
/**
 * @param A|string|int $x
 */
function bar($x): void {
    if ($x instanceof A || is_string($x)) {
        /** @mir-check $x is A|string */
        $_ = 1;
    }
}
===expect===
