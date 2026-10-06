===description===
does not report extra variadic arguments
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function many(int $first, int ...$rest): void {}
many(1, 2, 3);
