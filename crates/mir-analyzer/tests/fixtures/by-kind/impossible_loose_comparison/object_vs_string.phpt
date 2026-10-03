===description===
Objects can never be loosely equal to strings in PHP.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(\stdClass $obj, string $s): void {
    if ($obj == $s) {}
//      ^^^^^^^^^^ ImpossibleLooseComparison: '==' between 'stdClass' and 'string' is always false — these types can never be loosely equal
}
===expect===
