===description===
An array-typed variable can never be === to a string.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(array $arr): void {
    if ($arr === "foo") {}
//      ^^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'array' and '"foo"' is always false — these types can never be identical
}
===expect===
