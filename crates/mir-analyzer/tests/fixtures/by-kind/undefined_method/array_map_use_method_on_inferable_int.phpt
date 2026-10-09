===description===
Array map callback param is seeded as int, so the method call is not reported as mixed
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = array_map(function ($i) { return $i->foo(); }, [1, 2, 3, 4]);
