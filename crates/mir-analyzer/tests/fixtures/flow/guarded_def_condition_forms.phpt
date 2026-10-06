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
}

function oppositePolarity(?string $g): void {
    if (!empty($g)) { $x = compute(); }
    if (empty($g)) { echo $x; }
}

function identicalNullBranch(?string $g): void {
    if ($g !== null) { $x = compute(); }
    if ($g === null) { echo $x; }
}

function otherVariable(?string $g, ?string $h): void {
    if (!empty($g)) { $x = compute(); }
    if (!empty($h)) { echo $x; }
}

function guardReassigned(?string $g): void {
    if (!empty($g)) { $x = compute(); }
    $g = rand(0, 1) ? 'a' : null;
    if (!empty($g)) { echo $x; }
}

function guardReassignedInBranch(?string $g): void {
    if (isset($g)) {
        $g = rand(0, 1) ? 'a' : null;
        $x = compute();
    }
    if (isset($g)) { echo $x; }
}

function withElseBranch(?string $g): void {
    if (!empty($g)) { $x = compute(); } else { echo 'no'; }
    if (!empty($g)) { echo $x; }
}

function assignmentInCondition(): void {
    if (($g = rand(0, 1) ? 'a' : null) !== null) { $x = compute(); }
    if (($g = rand(0, 1) ? 'a' : null) !== null) { echo $x; }
}

function multiIssetIsNotAGuard(?string $g, ?string $h): void {
    if (isset($g, $h)) { $x = compute(); }
    if (isset($g, $h)) { echo $x; }
}
===expect===
PossiblyUndefinedVariable@62:28-62:30: Variable $x might not be defined
PossiblyUndefinedVariable@67:26-67:28: Variable $x might not be defined
PossiblyUndefinedVariable@72:28-72:30: Variable $x might not be defined
PossiblyUndefinedVariable@77:27-77:29: Variable $x might not be defined
PossiblyUndefinedVariable@83:27-83:29: Variable $x might not be defined
PossiblyUndefinedVariable@91:26-91:28: Variable $x might not be defined
PossiblyUndefinedVariable@96:27-96:29: Variable $x might not be defined
PossiblyUndefinedVariable@101:56-101:58: Variable $x might not be defined
PossiblyUndefinedVariable@106:30-106:32: Variable $x might not be defined
