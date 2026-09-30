===description===
An integer-typed variable can never be === to null.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(int $x): void {
    if ($x === null) {}
//      ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'int' and 'null' is always false — these types can never be identical
//      ^^^^^^^^^^^ RedundantCondition: Condition is always true/false for type 'bool'
}
===expect===
