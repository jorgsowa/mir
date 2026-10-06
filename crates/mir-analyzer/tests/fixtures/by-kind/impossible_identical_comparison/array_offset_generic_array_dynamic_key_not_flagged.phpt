===description===
Null comparisons on generic array reads with dynamic keys are allowed.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array<string, string> $map */
function test(array $map, string $k): void {
    $v = $map[$k];
    if ($v === null) {}
}
