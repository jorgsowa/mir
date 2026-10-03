===description===
True increment
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = true;
$a++;
//<^^ InvalidOperand: Operator '++' not supported for operand of type 'true'
===expect===
