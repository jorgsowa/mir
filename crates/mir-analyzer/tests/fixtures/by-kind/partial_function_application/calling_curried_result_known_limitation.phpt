===description===
The placeholder call is not modeled as a new Closure, so calling its result yields `mixed`.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php

function add(int $a, int $b): int {
    return $a + $b;
}

$addFive = add(?, 5);
//             ^ ParseError: Parse error: 'partial function application' requires PHP 8.6 or higher [version-too-low]
$result = $addFive(10);
//<^^^^^^^^^^^^^^^^^^^^^^ MixedAssignment: Variable $result is assigned a mixed type
