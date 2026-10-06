===description===
new with array variable should error
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(array $config) {
    new $config();
//      ^^^^^^^ InvalidStringClass: Dynamic class instantiation requires string or class-string type, got 'array'
}
