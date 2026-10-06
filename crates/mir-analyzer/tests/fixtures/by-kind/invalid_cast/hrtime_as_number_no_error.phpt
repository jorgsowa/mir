===description===
hrtime(true) returns int, not int|false — casting to string must not emit InvalidCast

===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$ns = hrtime(true);
$str = (string)$ns;
