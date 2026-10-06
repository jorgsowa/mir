===description===
FN: unary `-`/`+` never checked for a non-numeric operand, unlike binary
arithmetic and unary `~`.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = [1, 2];
$b = -$a;
//    ^^ InvalidOperand: Operator '-' not supported for operand of type 'array{0: 1, 1: 2}'
