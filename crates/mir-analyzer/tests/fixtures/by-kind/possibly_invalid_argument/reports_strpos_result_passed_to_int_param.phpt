===description===
reports strpos result passed to int param
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takesInt(int $n): void { var_dump($n); }
function test(string $haystack, string $needle): void {
    takesInt(strpos($haystack, $needle));
//           ^^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyInvalidArgument: Argument $n of takesInt() expects 'int', possibly different type 'int<0, max>|false' provided
}
===expect===
