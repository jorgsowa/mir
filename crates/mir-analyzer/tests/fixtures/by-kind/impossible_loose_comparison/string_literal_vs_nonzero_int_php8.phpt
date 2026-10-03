===description===
A non-numeric literal string vs a non-zero integer is always false in PHP 8:
the int is converted to string and compared, so "bar" != "5".
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
    $s = "bar";
    if ($s == 5) {}
//      ^^^^^^^ ImpossibleLooseComparison: '==' between '"bar"' and '5' is always false — these types can never be loosely equal
}
===expect===
