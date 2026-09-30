===description===
positive-int >= 1 is always true - should fire RedundantCondition
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param positive-int $n */
function test(int $n): void {
    if ($n >= 1) {
//      ^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
        echo "always";
    }
}
===expect===
