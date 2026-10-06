===description===
A flag does not prove a definition when the flag can hold the value from
elsewhere, when it is rewritten, or when the definition is not unconditional
in the branch that sets it.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function flagSetWithoutDefinition(): void {
    $mode = null;
    if (rand(0, 1) === 1) { $file = 'f'; $mode = 'x'; }
    if (rand(0, 1) === 1) { $mode = 'x'; }
    if ($mode === 'x') { echo $file; }
}

function flagAlreadyHeldValue(): void {
    $mode = 'x';
    if (rand(0, 1) === 1) { $file = 'f'; $mode = 'x'; }
    if ($mode === 'x') { echo $file; }
}

function flagOfUnknownType(string $mode): void {
    if (rand(0, 1) === 1) { $file = 'f'; $mode = 'x'; }
    if ($mode === 'x') { echo $file; }
}

function flagReassignedAfter(): void {
    $mode = null;
    if (rand(0, 1) === 1) { $file = 'f'; $mode = 'x'; }
    $mode = 'x';
    if ($mode === 'x') { echo $file; }
}

function definitionNotUnconditional(): void {
    $mode = null;
    if (rand(0, 1) === 1) {
        if (rand(0, 1) === 1) { $file = 'f'; }
        $mode = 'x';
    }
    if ($mode === 'x') { echo $file; }
}

function elseSetsFlagToOtherValue(): void {
    $mode = null;
    if (rand(0, 1) === 1) { $file = 'f'; $mode = 'x'; } else { $mode = 'y'; }
    if ($mode !== null) { echo $file; }
}

/** @param list<string> $names */
function otherBranchSetsFlag(array $names): void {
    $mode = null;
    foreach ($names as $n) {
        if (str_ends_with($n, '.x')) { $file = $n; $mode = 'x'; }
        elseif (str_ends_with($n, '.y')) { $mode = 'x'; }
    }
    if ($mode === 'x') { echo $file; }
}
===expect===
PossiblyUndefinedVariable@6:30-6:35: Variable $file might not be defined
PossiblyUndefinedVariable@12:30-12:35: Variable $file might not be defined
PossiblyUndefinedVariable@17:30-17:35: Variable $file might not be defined
PossiblyUndefinedVariable@24:30-24:35: Variable $file might not be defined
PossiblyUndefinedVariable@33:30-33:35: Variable $file might not be defined
ImpossibleIdenticalComparison@39:8-39:22: '!==' between '"x"|"y"' and 'null' is always true — these types can never be identical
PossiblyUndefinedVariable@39:31-39:36: Variable $file might not be defined
PossiblyUndefinedVariable@49:30-49:35: Variable $file might not be defined
