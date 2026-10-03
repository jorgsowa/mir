===description===
Invalid scalar argument
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function fooFoo(int $a): void {}
fooFoo("string");
//     ^^^^^^^^ InvalidArgument: Argument $a of fooFoo() expects 'int', got '"string"'
===expect===
