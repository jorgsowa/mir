===description===
array_values on a non-empty array yields a non-empty-list
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param non-empty-array<string, int> $map */
function test(array $map): void {
    $vals = array_values($map);
    /** @mir-check $vals is non-empty-list<int> */
    $_ = $vals;
}
===expect===
