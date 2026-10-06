===description===
Resource cannot be coerced to string
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @mutation-free */
function takesString(string $s) : void {}
$a = fopen("php://memory", "r");
takesString($a);
