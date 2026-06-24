===description===
true and false are specific bool literals that can never be identical to each other.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test_true(true $x): void {
    if ($x === false) {}
}

function test_false(false $x): void {
    if ($x === true) {}
}
===expect===
ImpossibleIdenticalComparison@3:8-3:20: '===' between 'true' and 'false' is always false — these types can never be identical
ImpossibleIdenticalComparison@7:8-7:19: '===' between 'false' and 'true' is always false — these types can never be identical
