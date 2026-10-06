===description===
new with union type containing non-string should error
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(int|bool $value) {
    new $value();
//      ^^^^^^ InvalidStringClass: Dynamic class instantiation requires string or class-string type, got 'int|bool'
}
