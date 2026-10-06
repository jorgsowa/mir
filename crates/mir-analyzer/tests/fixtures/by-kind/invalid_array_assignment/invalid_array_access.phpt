===description===
Invalid array access
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = 5;
$a[0] = 5;
//<^^^^^^^^^ InvalidArrayAssignment: Cannot use [] assignment on non-array type '5'
