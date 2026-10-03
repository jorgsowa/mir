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
//             ^^^^^^^^^ InvalidAttribute: #[Attribute] can only be applied to classes, not parameters

===expect===
