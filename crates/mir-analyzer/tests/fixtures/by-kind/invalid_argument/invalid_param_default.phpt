===description===
Invalid param default
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(int $p = false) {}
//                  ^^^^^ CompileError: Cannot use bool as default value for parameter $p of type int [default-value-type]
