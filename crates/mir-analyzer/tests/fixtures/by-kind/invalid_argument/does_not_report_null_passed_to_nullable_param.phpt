===description===
does not report null passed to nullable param
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(?string $x): void {}
function test(): void { f(null); }
===expect===
