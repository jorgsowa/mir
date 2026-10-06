===description===
reports incompatible union arg
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function g(): int|string { return 1; }
function f(int $x): void { var_dump($x); }
function test(): void { f(g()); }
//                        ^^^ PossiblyInvalidArgument: Argument $x of f() expects 'int', possibly different type 'int|string' provided
