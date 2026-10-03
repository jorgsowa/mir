===description===
Invalid g m p operation
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = gmp_init(2);
$b = "a" + $a;
//   ^^^^^^^^ InvalidOperand: Operator '+' not supported between '"a"' and 'mixed'
===expect===
