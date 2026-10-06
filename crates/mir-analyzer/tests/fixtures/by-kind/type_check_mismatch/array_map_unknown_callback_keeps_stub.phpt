===description===
array_map with a bare callable param falls back to the generic stub array (no fabricated element type)
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param callable $cb
 * @param list<int> $nums
 */
function test(callable $cb, array $nums): void {
    $r = array_map($cb, $nums);
    /** @mir-check $r is array */
    $_ = $r;
}
