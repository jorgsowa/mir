===description===
does not report single false
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takesInt(int $n): void { var_dump($n); }
function test(): void {
    takesInt(false);
//           ^^^^^ InvalidArgument: Argument $n of takesInt() expects 'int', got 'false'
}
