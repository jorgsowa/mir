===description===
Undefined variable
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = function() use ($i) {};
//                   ^^ UndefinedVariable: Variable $i is not defined
===expect===
