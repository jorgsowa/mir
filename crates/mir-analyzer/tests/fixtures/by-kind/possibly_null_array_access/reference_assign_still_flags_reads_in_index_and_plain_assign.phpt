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
    return $v;
}

function inIndex(array $a, ?array $b): void {
    $r = &$a[$b['k']];
    $r = 1;
}

function readAfterRef(?array $a): mixed {
    $r = &$a['k'];
    $r = 1;
    return $a['other'];
}
===expect===
PossiblyNullArrayAccess@3:9-3:16: Cannot access array on possibly null value
PossiblyNullArrayAccess@8:13-8:20: Cannot access array on possibly null value
