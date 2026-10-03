===description===
A valid member of a literal-string union containing '@' is not flagged.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param 'admin@example.com'|'guest@example.com' $email */
function f($email): void {}

f('admin@example.com');
===expect===
