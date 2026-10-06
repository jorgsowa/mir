===description===
emits invalid argument not possibly for fully wrong union
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takesInt(int $n): void { var_dump($n); }
/** @return string|false */
function getResult(): string|false { return 'x'; }
function test(): void {
    takesInt(getResult());
//           ^^^^^^^^^^^ InvalidArgument: Argument $n of takesInt() expects 'int', got 'string|false'
}
