===description===
non-empty-array is always truthy; truthy-check on it is a RedundantCondition
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param non-empty-array<string, int> $arr */
function test(array $arr): void {
    if ($arr) {
//      ^^^^ RedundantCondition: Condition is always true, so the check is redundant
        $_ = $arr;
    }
}
===expect===
