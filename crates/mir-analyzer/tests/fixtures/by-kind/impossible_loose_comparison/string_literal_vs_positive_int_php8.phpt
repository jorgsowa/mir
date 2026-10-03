===description===
PHP 8: non-numeric literal string vs positive-int is always false.
No positive integer can ever loosely equal a non-numeric string in PHP 8+.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.0</phpVersion>
</mir>
===file===
<?php
/** @param positive-int $n */
function test(int $n): void {
    $s = "foo";
    if ($s == $n) {}
//      ^^^^^^^^ ImpossibleLooseComparison: '==' between '"foo"' and 'positive-int' is always false — these types can never be loosely equal
}
===expect===
