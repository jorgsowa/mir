===description===
`$x instanceof A && $x instanceof B` for two unrelated CONCRETE classes
(no common interface) is provably impossible — PHP's single inheritance
makes "also an A" impossible once already known to be a B — so the branch
is flagged as unreachable rather than silently narrowing to just B.
===config===
suppress=UnusedParam
===file===
<?php
class A {}
class B {}

/** @param A|B $x */
function f($x): void {
    if ($x instanceof A && $x instanceof B) {
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
        echo get_class($x);
    }
}
===expect===
