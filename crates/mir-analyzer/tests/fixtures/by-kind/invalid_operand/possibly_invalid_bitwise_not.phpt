===description===
Possibly invalid bitwise not
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = ~(rand(0, 1) ? 2 : null);
//    ^^^^^^^^^^^^^^^^^^^^^^^ PossiblyNullOperand: Operator '~' operand '2|null' might be null
