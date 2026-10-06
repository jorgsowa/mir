===description===
scalar (int|float|string|bool) is open — no false positive.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param scalar $x */
function test(mixed $x): void {
    if ($x === "foo") {}
    if ($x === 42) {}
}
