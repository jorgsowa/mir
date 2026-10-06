===description===
array_map(null, ...) zip mode is not modeled — the generic stub array is kept
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
 * @param list<int> $a
 * @param list<int> $b
 */
function test(array $a, array $b): void {
    $r = array_map(null, $a, $b);
    /** @mir-check $r is array */
    $_ = $r;
}
