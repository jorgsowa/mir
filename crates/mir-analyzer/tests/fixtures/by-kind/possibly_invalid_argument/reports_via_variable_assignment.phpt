===description===
reports via variable assignment
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takesInt(int $n): void { var_dump($n); }
function test(string $s): void {
    $pos = strpos($s, 'x');
    /** @mir-check $pos is int|false */
    takesInt($pos);
//           ^^^^ PossiblyInvalidArgument: Argument $n of takesInt() expects 'int', possibly different type 'int<0, max>|false' provided
}
