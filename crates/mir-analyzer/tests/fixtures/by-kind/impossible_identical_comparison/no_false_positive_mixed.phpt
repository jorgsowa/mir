===description===
mixed is open — no ImpossibleIdenticalComparison should fire.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(mixed $x): void {
    if ($x === "foo") {}
    if ($x === 42) {}
    if ($x === null) {}
    if ($x === false) {}
}
