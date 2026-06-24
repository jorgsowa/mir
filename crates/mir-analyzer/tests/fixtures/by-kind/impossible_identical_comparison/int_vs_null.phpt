===description===
An integer-typed variable can never be === to null.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(int $x): void {
    if ($x === null) {}
}
===expect===
ImpossibleIdenticalComparison@3:8-3:19: '===' between 'int' and 'null' is always false — these types can never be identical
RedundantCondition@3:8-3:19: Condition is always true/false for type 'bool'
