===description===
InvalidArrayAssignment does NOT fire when assigning to an actual array.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = [];
$a[0] = 5;
===expect===
