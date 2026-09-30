===description===
Two variables with disjoint types compared with === is always false.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(int $a, string $b): void {
    if ($a === $b) {}
//      ^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'int' and 'string' is always false — these types can never be identical
}

function test_array_vs_int(array $arr, int $n): void {
    if ($arr === $n) {}
//      ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'array' and 'int' is always false — these types can never be identical
}
===expect===
