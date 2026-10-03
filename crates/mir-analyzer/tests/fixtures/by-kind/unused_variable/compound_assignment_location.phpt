===description===
Verify location tracking for compound assignment operators.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test() {
    $x = 1;
    $x += 2;
//  ^^ UnusedVariable: Variable $x is never read

    $y = "hello";
    $y .= "world";
//  ^^ UnusedVariable: Variable $y is never read

    $z = 10;
    ++$z;
//    ^^ UnusedVariable: Variable $z is never read

    $a = 1;
    $a++;
//  ^^ UnusedVariable: Variable $a is never read
}
===expect===
