===description===
`= NULL` counts as a null default.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param string $f */
function plain($f = NULL): void {}
plain(null);
===expect===
