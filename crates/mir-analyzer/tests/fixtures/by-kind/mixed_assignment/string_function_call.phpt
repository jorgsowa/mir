===description===
String function call
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$bad_one = "hello";
$a = $bad_one(1);
//<^^^^^^^^^^^^^^^^ MixedAssignment: Variable $a is assigned a mixed type
===expect===
