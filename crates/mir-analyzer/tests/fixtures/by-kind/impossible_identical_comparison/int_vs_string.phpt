===description===
An integer-typed variable can never be === to a string literal.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(int $x): void {
    if ($x === "hello") {}
//      ^^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'int' and '"hello"' is always false — these types can never be identical
    if ($x !== "world") {}
//      ^^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '!==' between 'int' and '"world"' is always true — these types can never be identical
}
===expect===
