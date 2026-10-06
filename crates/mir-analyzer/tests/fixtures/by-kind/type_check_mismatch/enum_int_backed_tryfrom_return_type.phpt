===description===
Int-backed enum ::tryFrom() returns enum|null, not mixed.
Expected: no issue.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.1</phpVersion>
</mir>
===file===
<?php
enum Priority: int {
    case Low = 1;
    case High = 2;
}

$p = Priority::tryFrom(99);
/** @mir-check $p is Priority|null */
if ($p !== null) {
    echo $p->value;
}
