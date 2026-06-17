===description===
int<0,0> (exactly zero) is never truthy; truthy-check on it is a RedundantCondition.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param int<0, 0> $n */
function test(int $n): void {
    if ($n) {
        $_ = $n;
    }
}
===expect===
RedundantCondition@4:8-4:10: Condition is always true/false for type 'int<0, 0>'
