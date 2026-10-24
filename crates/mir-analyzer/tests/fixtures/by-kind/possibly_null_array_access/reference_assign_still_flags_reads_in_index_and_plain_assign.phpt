===description===
Only the base of the `=&` array access auto-vivifies; a nullable array read
in the index expression, or a plain (non-reference) read, is still reported.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArrayOffset errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function plain(?array $a): mixed {
    $v = $a['q'];
//       ^^^^^^^ PossiblyNullArrayAccess: Cannot access array on possibly null value
    return $v;
}

function inIndex(array $a, ?array $b): void {
    $r = &$a[$b['k']];
//           ^^^^^^^ PossiblyNullArrayAccess: Cannot access array on possibly null value
    $r = 1;
}

function readAfterRef(?array $a): mixed {
    $r = &$a['k'];
    $r = 1;
    return $a['other'];
}
