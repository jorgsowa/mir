===description===
String increment
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = "hello";
$a++;
//<^^ InvalidOperand: Operator '++' not supported for operand of type '"hello"'
