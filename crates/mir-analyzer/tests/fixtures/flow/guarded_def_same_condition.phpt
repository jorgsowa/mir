===description===
A variable assigned under `if ($flag)` is defined wherever `$flag` is truthy
again; the guard being rewritten or a different guard keeps it possibly undefined.
===file===
<?php
function compute(): int { return 1; }

function andChain(bool $flag): void {
    if ($flag) { $x = compute(); }
    if ($flag && $x > 0) {
        /** @mir-check $x is int */
        echo $x;
    }
}

function secondIf(bool $flag): void {
    if ($flag) { $x = compute(); }
    if ($flag) {
        /** @mir-check $x is int */
        echo $x;
    }
}

function parenthesizedGuard(bool $flag): void {
    if (($flag)) { $x = compute(); }
    if ($flag && $x > 0) { echo $x; }
}

function guardReassigned(bool $flag): void {
    if ($flag) { $x = compute(); }
    $flag = (bool) rand(0, 1);
    if ($flag && $x > 0) { echo $x; }
//               ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}

function guardReassignedInBranch(bool $flag): void {
    if ($flag) {
        $flag = (bool) rand(0, 1);
        $x = compute();
    }
    if ($flag && $x > 0) { echo $x; }
//               ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}

function otherGuard(bool $flag, bool $other): void {
    if ($flag) { $x = compute(); }
    if ($other && $x > 0) { echo $x; }
//                ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}

function variableReassigned(bool $flag): void {
    if ($flag) { $x = compute(); }
    unset($x);
    if ($flag) { echo $x; }
//                    ^^ UndefinedVariable: Variable $x is not defined
}

function withElseBranch(bool $flag): void {
    if ($flag) { $x = compute(); } else { echo 'no'; }
    if ($flag && $x > 0) { echo $x; }
//               ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}

function negatedGuard(bool $flag): void {
    if ($flag) { $x = compute(); }
    if (!$flag && $x > 0) { echo $x; }
//                ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}
===expect===
