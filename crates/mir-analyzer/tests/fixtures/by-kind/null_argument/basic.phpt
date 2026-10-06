===description===
Basic
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takes_string(string $s): void { var_dump($s); }

takes_string(null);
//           ^^^^ NullArgument: Argument $s of takes_string() cannot be null
