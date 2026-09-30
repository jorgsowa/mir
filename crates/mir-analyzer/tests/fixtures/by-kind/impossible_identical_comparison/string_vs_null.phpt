===description===
A non-nullable string-typed variable can never be === to null.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(string $s): void {
    if ($s === null) {}
//      ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'string' and 'null' is always false — these types can never be identical
//      ^^^^^^^^^^^ RedundantCondition: Condition is always true/false for type 'bool'
}
===expect===
