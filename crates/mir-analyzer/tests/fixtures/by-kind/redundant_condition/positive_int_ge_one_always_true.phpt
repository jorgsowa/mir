===description===
positive-int >= 1 is always true - should fire RedundantCondition
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param positive-int $n */
function test(int $n): void {
    if ($n >= 1) {
//      ^^^^^^^ RedundantCondition: Condition is always true/false for type 'bool'
        echo "always";
    }
}
===expect===
