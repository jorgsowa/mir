===description===
does not report correct int arg
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(int $x): void { var_dump($x); }
function test(): void { f(42); }
