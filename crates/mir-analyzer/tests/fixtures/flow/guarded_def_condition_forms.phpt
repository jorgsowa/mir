===description===
A variable assigned under `!empty($g)`, `isset($g)` or `$g !== null` is defined
wherever the same condition holds again; weaker or unrelated conditions keep it
possibly undefined.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function compute(): int { return 1; }

function notEmptyGuard(?string $g): void {
    if (!empty($g)) { $x = compute(); }
    if (!empty($g)) {
        /** @mir-check $x is int */
        echo $x;
    }
}

function issetGuard(?string $g): void {
    if (isset($g)) { $x = compute(); }
    if (isset($g)) {
        /** @mir-check $x is int */
        echo $x;
    }
}

function notNullGuard(?string $g): void {
    if ($g !== null) { $x = compute(); }
    if ($g !== null) {
        /** @mir-check $x is int */
        echo $x;
    }
}

function nullOnLeftAndParens(?string $g): void {
    if ((null !== $g)) { $x = compute(); }
    if ($g !== null && $x > 0) {
        /** @mir-check $x is int */
        echo $x;
    }
}

function doubleNegation(?string $g): void {
    if (!empty($g)) { $x = compute(); }
    if (!(empty($g))) {
        /** @mir-check $x is int */
        echo $x;
    }
}

function mixedForms(?string $g): void {
    if ($g !== null) { $x = compute(); }
    if (isset($g)) {
        /** @mir-check $x is int */
        echo $x;
    }
}

function truthyImpliesNotNull(?string $g): void {
    if ($g !== null) { $x = compute(); }
    if (!empty($g)) {
        /** @mir-check $x is int */
        echo $x;
    }
}

function notNullDoesNotImplyTruthy(?string $g): void {
    if (!empty($g)) { $x = compute(); }
    if ($g !== null) { echo $x; }
//                          ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}

function oppositePolarity(?string $g): void {
    if (!empty($g)) { $x = compute(); }
    if (empty($g)) { echo $x; }
//                        ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}

function identicalNullBranch(?string $g): void {
    if ($g !== null) { $x = compute(); }
    if ($g === null) { echo $x; }
//                          ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}

function otherVariable(?string $g, ?string $h): void {
    if (!empty($g)) { $x = compute(); }
    if (!empty($h)) { echo $x; }
//                         ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}

function guardReassigned(?string $g): void {
    if (!empty($g)) { $x = compute(); }
    $g = rand(0, 1) ? 'a' : null;
    if (!empty($g)) { echo $x; }
//                         ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}

function guardReassignedInBranch(?string $g): void {
    if (isset($g)) {
        $g = rand(0, 1) ? 'a' : null;
        $x = compute();
    }
    if (isset($g)) { echo $x; }
//                        ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}

function withElseBranch(?string $g): void {
    if (!empty($g)) { $x = compute(); } else { echo 'no'; }
    if (!empty($g)) { echo $x; }
//                         ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}

function assignmentInCondition(): void {
    if (($g = rand(0, 1) ? 'a' : null) !== null) { $x = compute(); }
    if (($g = rand(0, 1) ? 'a' : null) !== null) { echo $x; }
//                                                      ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}

function multiIssetIsNotAGuard(?string $g, ?string $h): void {
    if (isset($g, $h)) { $x = compute(); }
    if (isset($g, $h)) { echo $x; }
//                            ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}
===expect===
