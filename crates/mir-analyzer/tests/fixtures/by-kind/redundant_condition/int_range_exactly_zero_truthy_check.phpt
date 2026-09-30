===description===
int<0,0> (exactly zero) is never truthy; truthy-check on it is a RedundantCondition.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param int<0, 0> $n */
function test(int $n): void {
    if ($n) {
//      ^^ RedundantCondition: Condition is always false, so the then branch is never reached
        $_ = $n;
    }
}
===expect===
