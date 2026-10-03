===description===
Undefined callable function
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foo(callable $c): void {}

foo("trime");
//  ^^^^^^^ UndefinedFunction: Function trime() is not defined
===expect===
