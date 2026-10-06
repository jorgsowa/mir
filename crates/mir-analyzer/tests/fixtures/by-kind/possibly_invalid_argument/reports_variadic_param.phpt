===description===
reports variadic param
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takesInts(int ...$ns): void { var_dump($ns); }
/** @return int|false */
function getResult(): int|false { return 1; }
function test(): void {
    takesInts(getResult(), getResult());
//            ^^^^^^^^^^^ PossiblyInvalidArgument: Argument $ns of takesInts() expects 'int', possibly different type 'int|false' provided
//                         ^^^^^^^^^^^ PossiblyInvalidArgument: Argument $ns of takesInts() expects 'int', possibly different type 'int|false' provided
}
