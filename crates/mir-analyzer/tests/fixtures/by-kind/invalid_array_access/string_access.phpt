===description===
InvalidArrayAccess fires when attempting array access on a string literal.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$s = "hello";
$c = $s[0];
