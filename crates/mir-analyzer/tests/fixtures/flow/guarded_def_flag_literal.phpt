===description===
A flag set to a literal in the same branch that defines a variable proves the
variable defined wherever the flag holds that value again, including across
loops, as long as only that branch can give the flag a non-initial value.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param list<string> $names */
function identicalAcrossLoops(array $names): void {
    $mode = null;
    foreach ($names as $n) {
        if (str_ends_with($n, '.x')) { $file = $n; $mode = 'x'; }
    }
    foreach ($names as $n) {
        if ($mode === 'x' && $n === $file) { echo "hit"; }
    }
}

function identicalStraightLine(): void {
    $mode = null;
    if (rand(0, 1) === 1) { $file = 'f'; $mode = 'x'; }
    if ($mode === 'x') {
        /** @mir-check $file is 'f' */
        echo $file;
    }
}

function negatedForm(): void {
    $mode = null;
    if (rand(0, 1) === 1) { $file = 'f'; $mode = 'x'; }
    if ($mode !== 'x') { return; }
    echo $file;
}

/** @param list<string> $names */
function truthyFlag(array $names): void {
    $found = false;
    foreach ($names as $n) {
        if (str_ends_with($n, '.x')) { $file = $n; $found = true; }
    }
    if ($found) { echo $file; }
}

/** @param list<string> $names */
function notNullFlag(array $names): void {
    $mode = null;
    foreach ($names as $n) {
        if (str_ends_with($n, '.x')) { $file = $n; $mode = 'x'; }
    }
    if ($mode !== null) { echo $file; }
    if (isset($mode)) { echo $file; }
}

function intFlag(): void {
    $kind = null;
    if (rand(0, 1) === 1) { $file = 'f'; $kind = 2; }
    if ($kind === 2) { echo $file; }
}

function sameValueWrittenAgainKeepsProof(): void {
    $mode = null;
    if (rand(0, 1) === 1) { $file = 'f'; $mode = 'x'; }
    if (rand(0, 1) === 1) { $mode = 'x'; $file = 'g'; }
    if ($mode === 'x') { echo $file; }
}

/** @param list<string> $names */
function severalFlagsAndBranches(array $names): void {
    $detected = false;
    $mode = null;
    foreach ($names as $n) {
        if ($n === 'a.xml') { $file = $n; $mode = 'xml'; $detected = true; }
        if ($n === 'b.css') { $file = $n; $mode = 'css'; $detected = true; }
    }
    if ($mode === 'xml') { echo $file; }
    if ($mode === 'css') { echo $file; }
    if ($detected) { echo $file; }
}
===expect===
