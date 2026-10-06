===description===
positive-int >= 1 is always true - should fire RedundantCondition
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
function test(int $n): void {
    if ($n >= 1) {
//      ^^^^^^^ RedundantCondition: Condition is always true, so the check is redundant
        echo "always";
    }
}
