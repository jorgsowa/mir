===description===
An integer-typed variable can never be === to a boolean.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(int $x): void {
    if ($x === true) {}
    if ($x === false) {}
}
===expect===
ImpossibleIdenticalComparison@3:8-3:19: '===' between 'int' and 'true' is always false — these types can never be identical
ImpossibleIdenticalComparison@4:8-4:20: '===' between 'int' and 'false' is always false — these types can never be identical
