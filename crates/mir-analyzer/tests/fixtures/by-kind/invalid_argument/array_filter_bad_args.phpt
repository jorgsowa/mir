===description===
Array filter bad args
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foo(int $i) : bool {
  return true;
}

array_filter(["hello"], "foo");
