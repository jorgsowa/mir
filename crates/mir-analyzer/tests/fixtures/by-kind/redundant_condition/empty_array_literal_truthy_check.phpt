===description===
Empty array literal is always falsy - truthy check should fire RedundantCondition
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    $a = [];
    if ($a) {
//      ^^ RedundantCondition: Condition is always false, so the then branch is never reached
        $_ = $a;
    }
}
