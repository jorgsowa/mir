===description===
PHP 8: non-numeric literal string vs int<1, 100> is always false.
No integer in the range [1, 100] can loosely equal a non-numeric string in PHP 8+.
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
/** @param int<1, 100> $n */
function test(int $n): void {
    $s = "baz";
    if ($s == $n) {}
//      ^^^^^^^^ ImpossibleLooseComparison: '==' between '"baz"' and 'int<1, 100>' is always false — these types can never be loosely equal
}
