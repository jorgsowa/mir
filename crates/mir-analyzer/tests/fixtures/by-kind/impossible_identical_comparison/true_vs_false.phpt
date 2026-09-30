===description===
true and false are specific bool literals that can never be identical to each other.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test_true(true $x): void {
    if ($x === false) {}
//      ^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'true' and 'false' is always false — these types can never be identical
}

function test_false(false $x): void {
    if ($x === true) {}
//      ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'false' and 'true' is always false — these types can never be identical
}
===expect===
