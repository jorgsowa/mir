===description===
Objects can never be loosely equal to null in PHP.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(\stdClass $obj): void {
    if ($obj == null) {}
//      ^^^^^^^^^^^^ ImpossibleLooseComparison: '==' between 'stdClass' and 'null' is always false — these types can never be loosely equal
//      ^^^^^^^^^^^^ RedundantCondition: Condition is always true/false for type 'bool'
}
===expect===
