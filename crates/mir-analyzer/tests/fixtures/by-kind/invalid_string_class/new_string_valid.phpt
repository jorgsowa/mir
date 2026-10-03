===description===
new with string variable should not error
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(string $className) {
    new $className();
}
===expect===
