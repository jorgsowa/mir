===description===
does not report null passed to mixed param
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(mixed $x): void {}
function test(): void { f(null); }
===expect===
