===description===
A numeric string is a valid unary `-`/`+` operand and must not be flagged.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = -"5";
$b = +"5";
