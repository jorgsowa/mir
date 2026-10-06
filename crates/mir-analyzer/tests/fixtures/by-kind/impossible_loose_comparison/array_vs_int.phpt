===description===
Arrays can never be loosely equal to integers in PHP.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(array $arr, int $n): void {
    if ($arr == $n) {}
//      ^^^^^^^^^^ ImpossibleLooseComparison: '==' between 'array' and 'int' is always false — these types can never be loosely equal
}
