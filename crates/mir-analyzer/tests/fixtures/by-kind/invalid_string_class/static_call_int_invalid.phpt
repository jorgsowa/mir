===description===
static call with int variable should error
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(int $value) {
    $value::method();
//  ^^^^^^ InvalidStringClass: Dynamic class instantiation requires string or class-string type, got 'int'
}
===expect===
