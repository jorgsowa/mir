===description===
PHP 8: non-numeric literal string vs float is always false.
The float is converted to a string ("1.5") and compared — "hello" can never equal "1.5".
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
    $s = "hello";
    if ($s == 1.5) {}
//      ^^^^^^^^^ ImpossibleLooseComparison: '==' between '"hello"' and '1.5' is always false — these types can never be loosely equal
}
