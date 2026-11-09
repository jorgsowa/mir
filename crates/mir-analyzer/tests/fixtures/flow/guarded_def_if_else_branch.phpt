===description===
A variable only one branch of an if/else assigns is defined wherever that branch's condition holds again; the opposite condition or a write to the guard in the else branch keeps it possibly undefined.
===file===
<?php
function thenBranch(?int $x): void {
    if ($x !== null) { $y = 1; } else { echo 0; }
    if ($x !== null) { echo $y; }
}

function elseBranch(?int $x): void {
    if ($x === null) { echo 0; } else { $y = 1; }
    if ($x !== null) { echo $y; }
}

function oppositeCondition(?int $x): void {
    if ($x !== null) { $y = 1; } else { echo 0; }
    if ($x === null) { echo $y; }
//                          ^^ PossiblyUndefinedVariable: Variable $y might not be defined
}

function guardWrittenInElse(?int $x): void {
    if ($x !== null) { $y = 1; } else { $x = null; }
    if ($x !== null) { echo $y; }
//                          ^^ PossiblyUndefinedVariable: Variable $y might not be defined
}
