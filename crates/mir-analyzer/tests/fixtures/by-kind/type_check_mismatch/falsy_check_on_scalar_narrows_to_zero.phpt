===description===
Falsy check on int narrows to 0; falsy check on float narrows to 0 (literal float 0.0 displays as "0").
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param int $x */
function test_int(int $x): void {
    if (!$x) {
        /** @mir-check $x is 0 */
        $_ = $x;
    }
}

/** @param float $f */
function test_float(float $f): void {
    if (!$f) {
        /** @mir-check $f is 0 */
        $_ = $f;
    }
}
===expect===
