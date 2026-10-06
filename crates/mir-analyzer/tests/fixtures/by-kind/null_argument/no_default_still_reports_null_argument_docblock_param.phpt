===description===
Without a default, a docblock non-null param still rejects null.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param string $f */
function plain($f): void {}
plain(null);
//    ^^^^ NullArgument: Argument $f of plain() cannot be null
