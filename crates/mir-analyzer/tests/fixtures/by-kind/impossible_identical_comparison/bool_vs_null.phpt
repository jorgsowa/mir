===description===
A bool-typed variable can never be === to null.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(bool $b): void {
    if ($b === null) {}
}
===expect===
ImpossibleIdenticalComparison@3:8-3:19: '===' between 'bool' and 'null' is always false — these types can never be identical
RedundantCondition@3:8-3:19: Condition is always true/false for type 'bool'
