===description===
A non-null default does not make the docblock-typed param nullable.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param string $f */
function plain($f = 'x'): void {}
plain(null);
//    ^^^^ NullArgument: Argument $f of plain() cannot be null
===expect===
