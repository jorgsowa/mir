===description===
Two variables with disjoint types compared with === is always false.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(int $a, string $b): void {
    if ($a === $b) {}
}

function test_array_vs_int(array $arr, int $n): void {
    if ($arr === $n) {}
}
===expect===
ImpossibleIdenticalComparison@3:8-3:17: '===' between 'int' and 'string' is always false — these types can never be identical
ImpossibleIdenticalComparison@7:8-7:19: '===' between 'array<mixed, mixed>' and 'int' is always false — these types can never be identical
