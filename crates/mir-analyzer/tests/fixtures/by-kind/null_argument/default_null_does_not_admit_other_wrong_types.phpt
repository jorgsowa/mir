===description===
Implicit nullability adds only null: other wrong argument types are still reported.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param string $f */
function plain($f = null): void {}
plain([]);
//    ^^ InvalidArgument: Argument $f of plain() expects 'string|null', got 'array{}'
