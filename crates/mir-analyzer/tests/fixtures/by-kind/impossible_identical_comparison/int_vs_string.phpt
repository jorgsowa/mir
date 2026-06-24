===description===
An integer-typed variable can never be === to a string literal.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(int $x): void {
    if ($x === "hello") {}
    if ($x !== "world") {}
}
===expect===
ImpossibleIdenticalComparison@3:8-3:22: '===' between 'int' and '"hello"' is always false — these types can never be identical
ImpossibleIdenticalComparison@4:8-4:22: '!==' between 'int' and '"world"' is always true — these types can never be identical
