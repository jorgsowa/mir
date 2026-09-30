===description===
int<0,0> (exactly zero) is never truthy; truthy-check on it is a RedundantCondition.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param int<0, 0> $n */
function test(int $n): void {
    if ($n) {
//      ^^ RedundantCondition: Condition of type 'int<0, 0>' always evaluates the same way, so one branch is unreachable
        $_ = $n;
    }
}
===expect===
