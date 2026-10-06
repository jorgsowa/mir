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
//                            ^^^^^ PossiblyUndefinedVariable: Variable $file might not be defined
}

function flagAlreadyHeldValue(): void {
    $mode = 'x';
    if (rand(0, 1) === 1) { $file = 'f'; $mode = 'x'; }
    if ($mode === 'x') { echo $file; }
//                            ^^^^^ PossiblyUndefinedVariable: Variable $file might not be defined
}

function flagOfUnknownType(string $mode): void {
    if (rand(0, 1) === 1) { $file = 'f'; $mode = 'x'; }
    if ($mode === 'x') { echo $file; }
//                            ^^^^^ PossiblyUndefinedVariable: Variable $file might not be defined
}

function flagReassignedAfter(): void {
    $mode = null;
    if (rand(0, 1) === 1) { $file = 'f'; $mode = 'x'; }
    $mode = 'x';
    if ($mode === 'x') { echo $file; }
//                            ^^^^^ PossiblyUndefinedVariable: Variable $file might not be defined
}

function definitionNotUnconditional(): void {
    $mode = null;
    if (rand(0, 1) === 1) {
        if (rand(0, 1) === 1) { $file = 'f'; }
        $mode = 'x';
    }
    if ($mode === 'x') { echo $file; }
//                            ^^^^^ PossiblyUndefinedVariable: Variable $file might not be defined
}

function elseSetsFlagToOtherValue(): void {
    $mode = null;
    if (rand(0, 1) === 1) { $file = 'f'; $mode = 'x'; } else { $mode = 'y'; }
    if ($mode !== null) { echo $file; }
//      ^^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '!==' between '"x"|"y"' and 'null' is always true — these types can never be identical
//                             ^^^^^ PossiblyUndefinedVariable: Variable $file might not be defined
}

/** @param list<string> $names */
function otherBranchSetsFlag(array $names): void {
    $mode = null;
    foreach ($names as $n) {
        if (str_ends_with($n, '.x')) { $file = $n; $mode = 'x'; }
        elseif (str_ends_with($n, '.y')) { $mode = 'x'; }
    }
    if ($mode === 'x') { echo $file; }
//                            ^^^^^ PossiblyUndefinedVariable: Variable $file might not be defined
}
