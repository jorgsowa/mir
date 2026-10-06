===description===
Expectations written as `^^^` annotations under the source line.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(NotARealClass $x): void {}
//         ^^^^^^^^^^^^^ UndefinedClass: Class NotARealClass does not exist
