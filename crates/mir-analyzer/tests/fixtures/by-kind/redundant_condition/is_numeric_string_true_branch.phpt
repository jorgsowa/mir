===description===
is_numeric($s) on a string type should not mark true branch unreachable
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(string $s): void {
    if (is_numeric($s)) {
        // true branch - s is a numeric string
        $_ = $s;
    }
    // should not emit RedundantCondition
}
===expect===
