===description===
Verify UnusedVariable is reported at the correct line and column.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function example() {
    $unused = 42;
//  ^^^^^^^ UnusedVariable: Variable $unused is never read
    return 10;
}
===expect===
