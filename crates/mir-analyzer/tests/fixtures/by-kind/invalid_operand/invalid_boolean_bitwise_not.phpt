===description===
Bitwise NOT on an array is invalid; bool is valid (PHP coerces bool→int, so ~true/-2/ is fine)
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = ~[1, 2, 3];
//    ^^^^^^^^^ InvalidOperand: Operator '~' not supported for operand of type 'array{0: 1, 1: 2, 2: 3}'
