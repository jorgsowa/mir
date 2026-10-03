===description===
does not report mixed arg
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(int $x): void { var_dump($x); }
function test(mixed $v): void { f($v); }
===expect===
