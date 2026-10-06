===description===
reports string passed as int
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(int $x): void { var_dump($x); }
function test(): void { f('hello'); }
//                        ^^^^^^^ InvalidArgument: Argument $x of f() expects 'int', got '"hello"'
