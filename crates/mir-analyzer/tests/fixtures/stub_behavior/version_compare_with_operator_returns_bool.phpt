===description===
version_compare() returns int without an operator and bool with one
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(string $v): array {
    $cmp = version_compare($v, '2.0');
    $gt = version_compare($v, '2.0', '>');
    /** @mir-check $cmp is int */
    /** @mir-check $gt is bool */
    return [$cmp, $gt];
}
