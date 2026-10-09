===description===
Array filter callback param is seeded as int, so the method call is not reported as mixed
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = array_filter([1, 2, 3, 4], function ($i) { return $i->foo(); });
