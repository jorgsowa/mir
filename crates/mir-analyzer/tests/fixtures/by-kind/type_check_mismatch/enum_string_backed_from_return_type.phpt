===description===
String-backed enum ::from() returns the enum type, not mixed.
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

$s = Status::from('active');
/** @mir-check $s is Status */
echo $s->value;
