===description===
No int to float enum
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param 0.3|0.5 $p */
function f($p): void {}
f(1);
//^ InvalidArgument: Argument $p of f() expects '0.3|0.5', got '1'
===expect===
