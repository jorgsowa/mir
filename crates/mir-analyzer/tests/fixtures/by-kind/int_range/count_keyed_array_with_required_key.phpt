===description===
count() over a shape with a required key is int<1, max>
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array{id: int, name?: string} $row */
function test(array $row): void {
    $n = count($row);
    /** @mir-check $n is int<1, max> */
    $_ = $n;
}
