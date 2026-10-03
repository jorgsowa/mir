===description===
reports both null and false independently
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takesInt(int $n): void { var_dump($n); }
/** @return int|null|false */
function getResult(): int|null|false { return 1; }
function test(): void {
    takesInt(getResult());
//           ^^^^^^^^^^^ PossiblyInvalidArgument: Argument $n of takesInt() expects 'int', possibly different type 'int|null|false' provided
//           ^^^^^^^^^^^ PossiblyNullArgument: Argument $n of takesInt() might be null
}
===expect===
