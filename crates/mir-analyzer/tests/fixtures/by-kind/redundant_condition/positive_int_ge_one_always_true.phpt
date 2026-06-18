===description===
positive-int >= 1 is always true - should fire RedundantCondition
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param positive-int $n */
function test(int $n): void {
    if ($n >= 1) {
        echo "always";
    }
}
===expect===
RedundantCondition@4:8-4:15: Condition is always true/false for type 'bool'
