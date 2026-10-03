===description===
variable accessed through variable-variable with known name should not be reported as unused

===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test() {
    $varName = 'foo';
    $foo = 'bar';
    return $$varName;
}
===expect===
