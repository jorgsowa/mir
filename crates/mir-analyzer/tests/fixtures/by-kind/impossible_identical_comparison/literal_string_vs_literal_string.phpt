===description===
Two specific inferred string literals that differ can never be ===.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(): void {
    $a = "foo";
    $b = "bar";
    if ($a === $b) {}
//      ^^^^^^^^^ ImpossibleIdenticalComparison: '===' between '"foo"' and '"bar"' is always false — these types can never be identical
}
===expect===
