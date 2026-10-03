===description===
static call with string variable should not error
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(string $className) {
    $className::method();
}
===expect===
