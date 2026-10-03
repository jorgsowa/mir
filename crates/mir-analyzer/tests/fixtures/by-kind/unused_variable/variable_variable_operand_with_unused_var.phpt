===description===
variable-variable should mark operand as read, but other vars should still be reported as unused

===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test() {
    $unused = 'never_used';
//  ^^^^^^^ UnusedVariable: Variable $unused is never read
    $key = 'value';
    echo $$key;
}
===expect===
