===description===
Missing attribute on param
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foo(#[Pure] string $str) : void {}
//             ^^^^ UndefinedAttributeClass: Attribute class Pure does not exist
===expect===
