===description===
PHP 8: "foo" != 0 is always true (the != operator).
The != operator is the inverse of ==; the same impossibility applies.
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
    if ($s != 0) {}
//      ^^^^^^^ ImpossibleLooseComparison: '!=' between '"foo"' and '0' is always true — these types can never be loosely equal
}
===expect===
