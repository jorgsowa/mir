===description===
FN: prefix -- never checked its operand, unlike postfix --.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = "hello";
--$a;
//^^ InvalidOperand: Operator '--' not supported for operand of type '"hello"'
===expect===
