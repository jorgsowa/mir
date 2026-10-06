===description===
+= and -= on ranged int types preserve bounds
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param positive-int $n */
function test_plus_eq(int $n): void {
    $n += 5;
    /** @mir-check $n is int<6, max> */
    $_ = $n;
}

/** @param int<0, 10> $n */
function test_range_minus_eq(int $n): void {
    $n -= 3;
    /** @mir-check $n is int<-3, 7> */
    $_ = $n;
}
