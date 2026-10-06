===description===
new with bool variable should error
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(bool $flag) {
    new $flag();
//      ^^^^^ InvalidStringClass: Dynamic class instantiation requires string or class-string type, got 'bool'
}
