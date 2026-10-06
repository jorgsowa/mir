===description===
Sibling of deprecated_enum_as_param: a plain enum stays silent.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
enum Status { case A; case B; }

function foo(Status $s): void {}
