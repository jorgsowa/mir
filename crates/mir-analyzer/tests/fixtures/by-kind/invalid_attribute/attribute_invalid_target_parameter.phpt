===description===
Attribute invalid target parameter
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foo(#[Attribute] string $_bar): void {}
//             ^^^^^^^^^ CompileError: Attribute "Attribute" cannot target parameter (allowed targets: class) [attribute-target-mismatch]
