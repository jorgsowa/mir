===description===
Trace variables
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @trace $a $b */
$a = getmypid();
//<^^^^^^^^^^^^^^^^ Trace: Type of $a is mixed
$b = getmypid();
