===description===
PHP === is type-and-value strict: int and float are different types.
$x === 1.5 is always false when $x is int.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(int $x): void {
    if ($x === 1.5) {}
}
===expect===
ImpossibleIdenticalComparison@3:8-3:18: '===' between 'int' and '1.5' is always false — these types can never be identical
