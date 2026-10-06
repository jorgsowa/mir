===description===
unknown param type hint
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(UnknownClass $x): void {}
//         ^^^^^^^^^^^^ UndefinedClass: Class UnknownClass does not exist
