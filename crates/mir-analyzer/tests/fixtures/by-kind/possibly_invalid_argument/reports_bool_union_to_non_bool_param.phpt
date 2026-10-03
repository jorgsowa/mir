===description===
reports bool union to non bool param
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takesInt(int $n): void { var_dump($n); }
/** @return int|bool */
function getResult(): int|bool { return 1; }
function test(): void {
    takesInt(getResult());
//           ^^^^^^^^^^^ PossiblyInvalidArgument: Argument $n of takesInt() expects 'int', possibly different type 'int|bool' provided
}
===expect===
