===description===
Writing a guarded variable never makes it less defined, so it stays defined
wherever the guard holds again; unset or reassigning the guard drops the proof.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array<string, int> $in */
function keyWrite(array $in): void {
    $on = count($in) > 0;
    if ($on) { $items = $in; }
    if ($on) { $items['k'] = 1; }
    if ($on) {
        /** @mir-check $items is array<string, int> */
        echo count($items);
    }
}

/** @param array<int, int> $in */
function appendWrite(array $in): void {
    $on = count($in) > 0;
    if ($on) { $items = $in; }
    if ($on) { $items[] = 2; }
    if ($on) { echo count($items); }
}

function fullReassignment(): void {
    $on = rand(0, 1) === 1;
    if ($on) { $items = [1]; }
    if ($on) { $items = [2]; }
    if ($on) {
        /** @mir-check $items is array{0: 2}|array{0: 1} */
        echo count($items);
    }
}

function writeInNestedBlock(bool $other): void {
    $on = rand(0, 1) === 1;
    if ($on) { $items = [1]; }
    if ($on) {
        if ($other) { $items = [2]; }
    }
    if ($on) { echo count($items); }
}

function strongerGuardKeepsProof(?string $g): void {
    if ($g !== null) { $x = 1; }
    if (!empty($g)) { $x = 2; }
    if ($g !== null) { echo $x; }
}

function weakerGuardWriteKeepsProof(?string $g): void {
    if (!empty($g)) { $x = 1; }
    if ($g !== null) { $x = 2; }
    if (!empty($g)) { echo $x; }
}

function unsetUnderGuard(): void {
    $on = rand(0, 1) === 1;
    if ($on) { $items = [1]; }
    if ($on) { unset($items); }
    if ($on) { echo count($items); }
//                        ^^^^^^ PossiblyUndefinedVariable: Variable $items might not be defined
}

function guardReassignedWhileWriting(): void {
    $on = rand(0, 1) === 1;
    if ($on) { $items = [1]; }
    if ($on) { $on = rand(0, 1) === 1; $items = [2]; }
    if ($on) { echo count($items); }
//                        ^^^^^^ PossiblyUndefinedVariable: Variable $items might not be defined
}

function writeOutsideGuard(): void {
    $on = rand(0, 1) === 1;
    if ($on) { $items = [1]; }
    if (!$on) { $items = [2]; }
    if ($on) { echo count($items); }
}

function writeInElse(): void {
    $on = rand(0, 1) === 1;
    if ($on) { $items = [1]; }
    if ($on) { echo 'a'; } else { $items = [2]; }
    if ($on) { echo count($items); }
}
===expect===
