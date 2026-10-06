===description===
Add array to number
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = [1] + 1;
//   ^^^^^^^ InvalidOperand: Operator '+' not supported between 'array{0: 1}' and '1'
