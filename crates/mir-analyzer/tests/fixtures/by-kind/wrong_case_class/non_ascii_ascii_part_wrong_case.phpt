===description===
A class name containing non-ASCII characters is still subject to ASCII case
checks: only the ASCII letters must match the declaration's casing.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class GrüBar {}
$x = new grübar();
//       ^^^^^^ WrongCaseClass: Class name 'grübar' has incorrect casing; use 'GrüBar'
