===description===
Duplicate param
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @return void
 */
function f($p, $p) {}
//             ^^ CompileError: Redefinition of parameter $p [duplicate-parameter]
