===description===
Narrowing a guarded variable by count(), an empty-array comparison, in_array()
or array_key_exists() refines its type without dropping the guarded definition.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array<string, int> $in */
function countGreater(array $in): void {
    $on = count($in) > 0;
    if ($on) { $items = $in; }
    if ($on && count($items) > 0) { echo 'x'; }
    if ($on) { echo count($items); }
}

/** @param array<string, int> $in */
function countTruthy(array $in): void {
    $on = count($in) > 0;
    if ($on) { $items = $in; }
    if ($on && count($items)) { echo 'x'; }
    if ($on) { echo count($items); }
}

/** @param array<string, int> $in */
function countIdentical(array $in): void {
    $on = count($in) > 0;
    if ($on) { $items = $in; }
    if ($on && count($items) === 1) { echo 'x'; }
    if ($on) { echo count($items); }
}

/** @param array<string, int> $in */
function emptyArrayComparison(array $in): void {
    $on = count($in) > 0;
    if ($on) { $items = $in; }
    if ($on && $items !== []) { echo 'x'; }
    if ($on) { echo count($items); }
}

/** @param array<string, int> $in */
function emptyArrayComparisonLeft(array $in): void {
    $on = count($in) > 0;
    if ($on) { $items = $in; }
    if ($on && [] === $items) { echo 'x'; }
    if ($on) { echo count($items); }
}

/** @param array<string, int> $in */
function inArrayHaystack(array $in): void {
    $on = count($in) > 0;
    if ($on) { $items = $in; }
    if ($on && in_array(1, $items, true)) { echo 'x'; }
    if ($on) { echo count($items); }
}

function inArrayNeedle(): void {
    $on = rand(0, 1) === 1;
    if ($on) { $mode = rand(0, 1) ? 'a' : 'b'; }
    if ($on && in_array($mode, ['a'], true)) { echo 'x'; }
    if ($on) { echo $mode; }
}

/** @param array<string, int> $in */
function keyExists(array $in): void {
    $on = count($in) > 0;
    if ($on) { $items = $in; }
    if ($on && array_key_exists('k', $items)) { echo 'x'; }
    if ($on) { echo count($items); }
}

/** @param array<string, int> $in */
function negatedFormsStayClean(array $in): void {
    $on = count($in) > 0;
    if ($on) { $items = $in; }
    if (!$on || count($items) === 0) { return; }
    echo count($items);
}

/** @param array<string, int> $in */
function unguardedReadStillReported(array $in): void {
    $on = count($in) > 0;
    if ($on) { $items = $in; }
    if ($on && count($items) > 0) { echo 'x'; }
    echo count($items);
}
===expect===
PossiblyUndefinedVariable@78:15-78:21: Variable $items might not be defined
