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
//      ^^^^^^^^ ImpossibleIdenticalComparison: '!==' between '1|2|3' and '5' is always true — these types can never be identical
//      ^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
        $_ = $n; // always here
    }
}
===expect===
