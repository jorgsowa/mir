===description===
gettimeofday(true) returns float, not array|float — casting to int must not emit InvalidCast

===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$t = gettimeofday(true);
$ms = (int)($t * 1000);

===expect===
