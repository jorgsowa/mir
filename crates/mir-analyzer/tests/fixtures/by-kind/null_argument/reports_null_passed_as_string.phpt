===description===
reports null passed as string
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(string $x): void { var_dump($x); }
function test(): void { f(null); }
//                        ^^^^ NullArgument: Argument $x of f() cannot be null
===expect===
