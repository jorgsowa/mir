===description===
PHP 8 changed string-vs-int comparison: when the string is non-numeric, the int
is converted to string rather than the string to int.  "foo" == 0 was always true
in PHP 7 (non-numeric string -> int(0)), but is always false in PHP 8+.
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
    $s = "foo";
    if ($s == 0) {}
//      ^^^^^^^ ImpossibleLooseComparison: '==' between '"foo"' and '0' is always false — these types can never be loosely equal
}
===expect===
