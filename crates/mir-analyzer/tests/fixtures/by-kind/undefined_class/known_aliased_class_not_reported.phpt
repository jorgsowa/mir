===description===
known aliased class not reported
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Bar {}
use Bar as Baz;
function f(Baz $x): void {}
