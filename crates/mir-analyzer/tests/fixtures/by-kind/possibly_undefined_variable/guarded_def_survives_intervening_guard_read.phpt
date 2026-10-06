===description===
A definition made under `if ($guard)` stays defined under a later `if ($guard)`
even when other `if`s test the guard in between; a real write to the guard
(assignment, by-reference argument, unset, foreach target) still drops it.
===file===
<?php
function fill(bool &$out): void { $out = true; }

function intervening_read(array $in): void {
    $enabled = count($in) > 0;
    if ($enabled) { $items = $in; }
    if ($enabled) { echo "log"; }
    if ($enabled) {
        /** @mir-check $items is array<mixed> */
        echo count($items);
    }
}

function intervening_compound_condition(array $in, bool $other): void {
    $enabled = count($in) > 0;
    if ($enabled) { $items = $in; }
    if ($enabled && $other) { echo "log"; }
    if (!$enabled) { echo "off"; }
    if ($enabled) { echo count($items); }
}

function intervening_assignment(array $in): void {
    $enabled = count($in) > 0;
    if ($enabled) { $items = $in; }
    if ($in) { $enabled = false; }
    if ($enabled) { echo count($items); }
//                             ^^^^^^ PossiblyUndefinedVariable: Variable $items might not be defined
}

function intervening_unset(array $in): void {
    $enabled = count($in) > 0;
    if ($enabled) { $items = $in; }
    if ($in) { unset($enabled); }
    if ($enabled) { echo count($items); }
//                             ^^^^^^ PossiblyUndefinedVariable: Variable $items might not be defined
}

function intervening_by_ref(array $in): void {
    $enabled = count($in) > 0;
    if ($enabled) { $items = $in; }
    if ($in) { fill($enabled); }
    if ($enabled) { echo count($items); }
//                             ^^^^^^ PossiblyUndefinedVariable: Variable $items might not be defined
}

/** @param list<bool> $in */
function intervening_foreach_target(array $in): void {
    $enabled = count($in) > 0;
    if ($enabled) { $items = $in; }
    foreach ($in as $enabled) { echo "x"; }
    if ($enabled) { echo count($items); }
//                             ^^^^^^ PossiblyUndefinedVariable: Variable $items might not be defined
}
===expect===
