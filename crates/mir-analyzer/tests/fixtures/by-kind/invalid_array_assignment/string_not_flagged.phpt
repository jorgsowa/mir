===description===
InvalidArrayAssignment does NOT fire for string — PHP allows single-character string subscript writes.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = "hello";
$a[0] = 'x';
===expect===
