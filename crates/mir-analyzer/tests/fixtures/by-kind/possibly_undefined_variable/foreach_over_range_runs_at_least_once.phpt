===description===
`range()` always contains `$start`, so a `foreach` over it runs at least once.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(int $n): int {
    $r = range(1, $n);
    /** @mir-check $r is non-empty-list<mixed> */
    foreach ($r as $_) {
        $last = 1;
    }
    return $last;
}
