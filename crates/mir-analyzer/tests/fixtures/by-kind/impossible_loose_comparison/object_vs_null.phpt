===description===
Objects can never be loosely equal to null in PHP.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(\stdClass $obj): void {
    if ($obj == null) {}
}
===expect===
ImpossibleLooseComparison@3:8-3:20: '==' between 'stdClass' and 'null' is always false — these types can never be loosely equal
RedundantCondition@3:8-3:20: Condition is always true/false for type 'bool'
