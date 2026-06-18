===description===
A variable known to be one of a closed literal union can never === a value
outside the union; both branches become known (true is always-true for !==,
always-false for ===).
===config===
suppress=UnusedVariable,UnusedParam,MissingParamType,DocblockTypeContradiction
===file===
<?php
/**
 * @param 1|2|3 $n
 */
function test_ne_impossible($n): void {
    if ($n !== 5) {
        $_ = $n; // always here
    }
}
===expect===
RedundantCondition@6:8-6:16: Condition is always true/false for type 'bool'
