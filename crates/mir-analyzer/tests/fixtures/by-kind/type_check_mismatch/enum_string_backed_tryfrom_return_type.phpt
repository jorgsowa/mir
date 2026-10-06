===description===
String-backed enum ::tryFrom() returns enum|null, not mixed.
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
enum Status: string {
    case Active = 'active';
    case Inactive = 'inactive';
}

$s = Status::tryFrom('active');
/** @mir-check $s is Status|null */
if ($s !== null) {
    echo $s->value;
}
