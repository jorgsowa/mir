===description===
A non-empty array is always truthy, so it can never be loosely equal to null
either — null converts to an empty array for the comparison.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param non-empty-array<string> $arr */
function test(array $arr): void {
    if ($arr == null) {}
//      ^^^^^^^^^^^^ ImpossibleLooseComparison: '==' between 'non-empty-array<int|string, string>' and 'null' is always false — these types can never be loosely equal
}
