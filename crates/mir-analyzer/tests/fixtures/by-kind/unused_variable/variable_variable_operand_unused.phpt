===description===
variable used only in variable-variable operand

===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test() {
    $foo = 'bar';
    $$foo = 42;
//  ^^^^^ UnusedVariable: Variable $bar is never read
}
===expect===
