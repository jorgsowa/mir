===description===
A union that includes the literal's family does not fire.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(string|int $x): void {
    if ($x === "foo") {}
    if ($x === 42) {}
}
