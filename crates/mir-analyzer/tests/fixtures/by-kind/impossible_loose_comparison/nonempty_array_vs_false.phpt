===description===
A non-empty array is always truthy, so it can never be loosely equal to false.
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
    if ($arr == false) {}
//      ^^^^^^^^^^^^^ DocblockTypeContradiction: Type 'non-empty-array<int|string, string>' makes '$arr == false' impossible — this can never hold
}
