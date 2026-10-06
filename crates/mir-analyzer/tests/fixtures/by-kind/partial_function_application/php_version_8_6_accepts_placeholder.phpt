===description===
Targeting PHP 8.6 lets the parser accept partial-application placeholders, so
no version-gate ParseError is raised and the placeholder argument is not
type-checked.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.6</phpVersion>
</mir>
===file===
<?php

function add(int $a, int $b): int {
    return $a + $b;
}

$partial = add(?, 5);
