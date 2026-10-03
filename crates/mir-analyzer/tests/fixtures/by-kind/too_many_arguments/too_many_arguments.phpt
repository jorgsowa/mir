===description===
Too many arguments
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function fooFoo(int $a): void {}
fooFoo(5, "dfd");
//        ^^^^^ TooManyArguments: Too many arguments for fooFoo(): expected 1, got 2
===expect===
