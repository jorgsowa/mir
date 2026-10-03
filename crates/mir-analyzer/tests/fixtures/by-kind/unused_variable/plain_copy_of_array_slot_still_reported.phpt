===description===
A plain (non-reference) copy of an array slot that is only written to is still reported.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param list<array{g: int, id: int}> $rows */
function group(array $rows): void {
    $groups = [];
    foreach ($rows as $r) {
        $bucket = $groups[$r['g']];
//      ^^^^^^^ UnusedVariable: Variable $bucket is never read
    }
}
===expect===
