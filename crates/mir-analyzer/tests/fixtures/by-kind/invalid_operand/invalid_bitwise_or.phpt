===description===
Invalid bitwise or
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = "x" | new stdClass;
//   ^^^^^^^^^^^^^^^^^^ InvalidOperand: Operator '|' not supported between '"x"' and 'stdClass'
