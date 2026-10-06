===description===
is_numeric($n) on int is always true - fires RedundantCondition
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(int $n): void {
    if (is_numeric($n)) {
//      ^^^^^^^^^^^^^^ RedundantCondition: Condition is always true, so the check is redundant
        $_ = $n;
    }
}
