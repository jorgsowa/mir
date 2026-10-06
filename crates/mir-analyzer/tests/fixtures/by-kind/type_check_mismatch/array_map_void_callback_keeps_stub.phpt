===description===
array_map with a void/never callback is degenerate — the generic stub array is kept
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param list<int> $nums */
function test(array $nums): void {
    $r = array_map(function (int $i): void { echo $i; }, $nums);
    /** @mir-check $r is array */
    $_ = $r;
}
