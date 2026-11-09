===description===
A flag the then branch sets to `true` and the else branch may leave `false` proves the
variables the else branch defines wherever the flag is falsy.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param ?list<string> $q */
function elseLeavesFlagFalse(?array $q): void {
    $required = false;
    if ($q === null) { $required = true; }
    else {
        $words = $q;
        if (!$words) { $required = true; }
    }
    if ($required) { return; }
    foreach ($words as $w) { echo $w; }
}

/** @param ?list<string> $q */
function thenMayLeaveFlagFalse(?array $q, bool $c): void {
    $required = false;
    if ($q === null) { $required = $c; }
    else { $words = $q; }
    if ($required) { return; }
    foreach ($words as $w) { echo $w; }
//           ^^^^^^ PossiblyUndefinedVariable: Variable $words might not be defined
}
