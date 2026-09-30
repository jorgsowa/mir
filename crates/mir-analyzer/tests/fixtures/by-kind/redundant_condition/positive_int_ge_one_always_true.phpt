===description===
positive-int >= 1 is always true - should fire RedundantCondition
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param positive-int $n */
function test(int $n): void {
    if ($n >= 1) {
//      ^^^^^^^ RedundantCondition: Condition is always true, so the else branch is never reached
        echo "always";
    }
}
===expect===
