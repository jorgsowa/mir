===description===
reports string or false to string param
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takesString(string $s): void { var_dump($s); }
/** @return string|false */
function getResult(): string|false { return 'x'; }
function test(): void {
    takesString(getResult());
//              ^^^^^^^^^^^ PossiblyInvalidArgument: Argument $s of takesString() expects 'string', possibly different type 'string|false' provided
}
