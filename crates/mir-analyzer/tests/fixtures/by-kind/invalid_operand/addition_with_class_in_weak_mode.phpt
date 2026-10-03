===description===
Addition with class in weak mode
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = "hi" + (new stdClass);
//   ^^^^^^^^^^^^^^^^^^^^^ InvalidOperand: Operator '+' not supported between '"hi"' and 'stdClass'
===expect===
