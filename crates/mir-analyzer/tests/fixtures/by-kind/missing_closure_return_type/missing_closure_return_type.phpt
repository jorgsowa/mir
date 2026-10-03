===description===
Missing closure return type
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = function() {
//   ^ +2:1 MissingClosureReturnType: Closure has no return type annotation
    return "foo";
};
===expect===
