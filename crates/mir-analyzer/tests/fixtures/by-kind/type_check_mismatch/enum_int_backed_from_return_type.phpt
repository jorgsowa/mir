===description===
Int-backed enum ::from() returns the enum type, not mixed.
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

$p = Priority::from(1);
/** @mir-check $p is Priority */
echo $p->value;
===expect===
