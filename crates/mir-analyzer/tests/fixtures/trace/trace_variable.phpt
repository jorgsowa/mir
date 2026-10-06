===description===
Trace variable
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @trace $a */
$a = getmypid();
//<^^^^^^^^^^^^^^^^ Trace: Type of $a is mixed
