===description===
abs(int) should not produce TypeMismatch when passed to takesInt()

===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function takesInt(int $x): void {}

function test(int $n): void {
    takesInt(abs($n)); // mir reports: Argument 1 expects int, got float|int (FALSE POSITIVE)
}
?>
===expect===
