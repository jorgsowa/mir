===description===
Null argument
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function fooFoo(int $a): void {}
fooFoo(null);
//     ^^^^ NullArgument: Argument $a of fooFoo() cannot be null
