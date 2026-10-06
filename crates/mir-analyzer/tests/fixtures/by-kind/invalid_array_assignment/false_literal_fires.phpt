===description===
InvalidArrayAssignment fires for literal false.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = false;
$a[0] = 5;
//<^^^^^^^^^ InvalidArrayAssignment: Cannot use [] assignment on non-array type 'false'
