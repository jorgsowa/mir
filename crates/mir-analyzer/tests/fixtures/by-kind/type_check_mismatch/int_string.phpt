===description===
mir-check detects int vs string type mismatch
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = 5;
/** @mir-check $x is int */
$x = "hello";
/** @mir-check $x is string */
echo $x;
