===description===
Conservative: numeric literal strings ("123", "3.14") can loosely equal integers/floats.
PHP 8 compares them numerically when the string is numeric — no warning.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.0</phpVersion>
</mir>
===file===
<?php
function test(): void {
    $s = "123";
    if ($s == 123) {}
    $t = "3.14";
    if ($t == 3) {}
    $u = "0";
    if ($u == 0) {}
}
