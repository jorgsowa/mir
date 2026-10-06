===description===
InvalidArrayAssignment fires for float-typed variables.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(float $a): void {
    $a[0] = 5;
//  ^^^^^^^^^ InvalidArrayAssignment: Cannot use [] assignment on non-array type 'float'
}
