===description===
A non-nullable string-typed variable can never be === to null.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(string $s): void {
    if ($s === null) {}
}
===expect===
ImpossibleIdenticalComparison@3:8-3:19: '===' between 'string' and 'null' is always false — these types can never be identical
RedundantCondition@3:8-3:19: Condition is always true/false for type 'bool'
