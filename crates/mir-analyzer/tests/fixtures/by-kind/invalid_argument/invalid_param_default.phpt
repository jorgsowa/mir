===description===
Invalid param default
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(int $p = false) {}
===expect===
