===description===
Trace emits the inferred type of a variable via @trace in a docblock.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = 42;
/** @trace $x */
$y = $x + 1;
//<^^^^^^^^^^^^ Trace: Type of $x is 42
