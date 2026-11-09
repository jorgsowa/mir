===description===
A variable assigned under `!$g`, `empty($g)` or `$g = f()` is defined wherever the
same condition holds again; the opposite condition keeps it possibly undefined.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param list<string> $all */
function negatedFlag(bool $global, array $all): void {
    if (!$global) { $required = array_slice($all, 1); }
    foreach ($all as $p) {
        if (!$global && !in_array($p, $required, true)) { echo $p; }
    }
}

function emptyGuard(?string $g): void {
    if (empty($g)) { $x = 1; }
    if (empty($g)) { echo $x; }
}

function assignedInCondition(): void {
    if ($e = rand(0, 1)) { $x = 1; }
    if ($e) { echo $x; }
}

function oppositePolarity(bool $g): void {
    if (!$g) { $x = 1; }
    if ($g) { echo $x; }
//                 ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}

function notIdenticalLiteral(string $m): void {
    if ($m !== 'default') { $x = 1; }
    if ($m !== 'default') { echo $x; }
    if ($m === 'other') { echo $x; }
    if ($m !== 'other') { echo $x; }
//                             ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}
