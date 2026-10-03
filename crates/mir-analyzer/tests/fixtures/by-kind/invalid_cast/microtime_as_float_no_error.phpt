===description===
microtime(true) returns float, not string|float — casting to int must not emit InvalidCast

===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$t = microtime(true);
$ms = (int)($t * 1000);

===expect===
