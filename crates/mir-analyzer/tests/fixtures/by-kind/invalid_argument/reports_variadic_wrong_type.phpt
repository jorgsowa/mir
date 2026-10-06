===description===
reports variadic wrong type
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(int ...$xs): void { var_dump($xs); }
function test(): void { f('a'); }
//                        ^^^ InvalidArgument: Argument $xs of f() expects 'int', got '"a"'
