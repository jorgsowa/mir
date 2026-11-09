===description===
A condition on an array element with a literal key guards definitions like one on a
variable; a write to the array drops the proof.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array<string, mixed> $a */
function complementaryEmptyChecks(array $a): void {
    if (!empty($a['url'])) { $title = 'link'; }
    if (empty($a['url'])) { $title = 'text'; }
    echo $title;
}

/** @param array<string, mixed> $a */
function sameElementCondition(array $a): void {
    if (isset($a['k'])) { $x = 1; }
    if (isset($a['k'])) { echo $x; }
}

/** @param array<string, mixed> $a */
function arrayWrittenInBetween(array $a): void {
    if (isset($a['k'])) { $x = 1; }
    $a['k'] = 0;
    if (isset($a['k'])) { echo $x; }
//                             ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}

/** @param array<string, mixed> $a */
function otherKey(array $a): void {
    if (isset($a['k'])) { $x = 1; }
    if (isset($a['j'])) { echo $x; }
//                             ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}

/** @param array<string, mixed> $a */
function elementUnsetInBetween(array $a): void {
    if (empty($a['k'])) { $x = 1; }
    unset($a['k']);
    if (empty($a['k'])) { echo $x; }
//                             ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}
