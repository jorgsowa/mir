===description===
@mir-check is lenient: a range-typed value still satisfies a plain `int` assertion
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array<int> $arr */
function test(array $arr): void {
    $n = count($arr);
    /** @mir-check $n is int */
    $_ = $n;
}
