===description===
Two specific inferred integer literals that differ can never be ===.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(): void {
    $a = 5;
    $b = 6;
    if ($a === $b) {}
}
===expect===
ImpossibleIdenticalComparison@5:8-5:17: '===' between '5' and '6' is always false — these types can never be identical
