===description===
Too few arguments
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function fooFoo(int $a): void {}
fooFoo();
//<^^^^^^^^ TooFewArguments: Too few arguments for fooFoo(): expected 1, got 0
