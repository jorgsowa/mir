===description===
Misplaced required param
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foo(string $bar = null, int $bat): void {}
foo();
//<^^^^^ TooFewArguments: Too few arguments for foo(): expected 1, got 0
