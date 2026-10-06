===description===
Bad addition
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = "b" + 5;
//   ^^^^^^^ InvalidOperand: Operator '+' not supported between '"b"' and '5'
