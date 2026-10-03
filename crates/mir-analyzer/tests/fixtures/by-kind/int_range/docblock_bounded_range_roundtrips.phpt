===description===
a declared int<2, 10> range parses its bounds and round-trips through an exact @mir-check
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param int<2, 10> $x */
function test(int $x): void {
    /** @mir-check $x is int<2, 10> */
    $_ = $x;
}
===expect===
