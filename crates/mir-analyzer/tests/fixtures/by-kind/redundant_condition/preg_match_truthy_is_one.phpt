===description===
After `if (preg_match(...))`, the result is known to be 1 (truthy int<0,1> = int<1,1>).
Checking `if ($r === 1)` in the true branch is therefore redundant.
===config===
suppress=UnusedVariable
===file===
<?php
function test(string $s): void {
    $r = preg_match('/foo/', $s);
    if ($r) {
        /** @mir-check $r is int<1, 1> */
        if ($r === 1) {
            $_ = 'always here';
        }
    }
}
===expect===
RedundantCondition@6:12-6:20: Condition is always true/false for type 'bool'
