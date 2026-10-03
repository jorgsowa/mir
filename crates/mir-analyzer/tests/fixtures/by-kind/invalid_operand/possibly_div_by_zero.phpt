===description===
Possibly div by zero
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = 5 / (rand(0, 1) ? 2 : null);
//   ^^^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyNullOperand: Operator '/' operand '2|null' might be null
===expect===
