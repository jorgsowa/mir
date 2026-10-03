===description===
Invalid bitwise not
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = ~new stdClass;
//    ^^^^^^^^^^^^ InvalidOperand: Operator '~' not supported for operand of type 'stdClass'
===expect===
