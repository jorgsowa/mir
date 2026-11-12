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
//      ^^^^^^^^ DocblockTypeContradiction: Type 'positive-int' makes '$s == $n' impossible — this can never hold
}
