===description===
MissingClosureReturnType does NOT fire when the closure has a return type annotation.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = function(): string {
    return "foo";
};
