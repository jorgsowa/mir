===description===
Unused param
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foo(int $i) {}
//           ^^^^^^ UnusedParam: Parameter $i is never used

foo(4);
===expect===
