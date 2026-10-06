===description===
Arrays can never be loosely equal to strings in PHP.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(array $arr, string $s): void {
    if ($arr == $s) {}
//      ^^^^^^^^^^ ImpossibleLooseComparison: '==' between 'array' and 'string' is always false — these types can never be loosely equal
}
